use crate::event::UnrecognizedInputEvent;
use crate::event::mouse::parse_mouse_event_bytes;
use crate::event::terminal::{BracketedPasteEnd, BracketedPasteStart};
use better_any::TidExt;
use vt_push_parser::{
    VT_PARSER_INTEREST_ALL,
    capture::{VTCaptureEvent, VTCapturePushParser, VTInputCapture},
    event::{DCSOwned, VTEvent},
};
use vtansi::registry::{AnsiEventData, Answer};
use vtansi::{StaticAnsiEncode, format_csi};

use super::common;

const MAX_UTF8_CHAR_BYTES: usize = 4;

/// Tracks the type of capture mode currently active.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum CaptureMode {
    #[default]
    None,
    /// Capturing bracketed paste content.
    BracketedPaste,
    /// Capturing mouse event bytes (3 UTF-8 chars: btn, col, row).
    /// Supports both default format (single bytes) and multibyte UTF-8 format (p1005).
    MouseEvent,
    /// Capturing DCS data.
    DcsData,
}

#[derive(Debug, Default)]
struct ParserState {
    // Buffer for incomplete UTF-8 sequences from previous feed (max 4 bytes)
    utf8_buffer: [u8; MAX_UTF8_CHAR_BYTES],
    utf8_buffer_len: usize,
    accum_buffer: Vec<u8>,
    capture_mode: CaptureMode,
    dcs_header: Option<DCSOwned>,
}

impl ParserState {
    const fn new() -> Self {
        Self {
            utf8_buffer: [0; MAX_UTF8_CHAR_BYTES],
            utf8_buffer_len: 0,
            accum_buffer: Vec::new(),
            capture_mode: CaptureMode::None,
            dcs_header: None,
        }
    }

    fn store_dcs_header(&mut self, dcs: &vt_push_parser::event::DCS) {
        self.dcs_header = Some(DCSOwned {
            private: dcs.private,
            params: dcs.params.to_owned(),
            intermediates: dcs.intermediates,
            final_byte: dcs.final_byte,
        });
    }

    fn clear_dcs_header(&mut self) {
        self.dcs_header = None;
    }
}

pub struct TerminalInputParser {
    seq_parser: VTCapturePushParser<VT_PARSER_INTEREST_ALL>,
    state: ParserState,
}

impl Default for TerminalInputParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalInputParser {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            seq_parser: VTCapturePushParser::new_with_interest::<
                VT_PARSER_INTEREST_ALL,
            >(),
            state: ParserState::new(),
        }
    }

    /// Decode a buffer of bytes into a series of events.
    pub fn decode_buffer<F>(input: &[u8], cb: &mut F)
    where
        for<'a> F: FnMut(&dyn vtansi::AnsiEvent),
    {
        let mut parser = TerminalInputParser::new();
        parser.feed_with(input, cb);
    }

    // =====================
    // Callback-driven API
    // =====================

    /// Feed bytes into the parser. This is the main entry point for the parser.
    /// It will call the callback with events as they are emitted.
    ///
    /// The callback must be valid for the lifetime of the `feed_with` call.
    ///
    /// The callback may emit any number of events (including zero), depending
    /// on the state of the internal parser.
    #[inline]
    pub fn feed_with<F>(&mut self, input: &[u8], cb: &mut F)
    where
        F: FnMut(&dyn vtansi::AnsiEvent),
    {
        self.seq_parser
            .feed_with(input, |vt_event: VTCaptureEvent| {
                Self::process_vt_event(&vt_event, &mut self.state, cb)
            });
    }

    #[allow(clippy::too_many_lines)]
    fn process_vt_event<F>(
        vt_event: &VTCaptureEvent,
        state: &mut ParserState,
        cb: &mut F,
    ) -> VTInputCapture
    where
        F: FnMut(&dyn vtansi::AnsiEvent),
    {
        match vt_event {
            VTCaptureEvent::VTEvent(VTEvent::Raw(bytes)) => {
                if state.utf8_buffer_len == 0 {
                    state.utf8_buffer_len =
                        bytes_to_events(bytes, &mut state.utf8_buffer, cb);
                } else {
                    let buf_len = state.utf8_buffer_len;

                    // Combine buffered bytes with just enough new bytes to
                    // complete a UTF-8 char (max 4 bytes total).
                    let mut temp_buf = [0u8; MAX_UTF8_CHAR_BYTES];
                    temp_buf[..buf_len]
                        .copy_from_slice(&state.utf8_buffer[..buf_len]);

                    let take = bytes.len().min(MAX_UTF8_CHAR_BYTES - buf_len);
                    temp_buf[buf_len..buf_len + take]
                        .copy_from_slice(&bytes[..take]);

                    let incomplete_len = bytes_to_events(
                        &temp_buf[..buf_len + take],
                        &mut state.utf8_buffer,
                        cb,
                    );

                    state.utf8_buffer_len =
                        if take < bytes.len() && incomplete_len <= take {
                            // Process from start of incomplete sequence (or from take if none)
                            bytes_to_events(
                                &bytes[take - incomplete_len..],
                                &mut state.utf8_buffer,
                                cb,
                            )
                        } else {
                            // No more input, or incomplete bytes span buffer boundary
                            incomplete_len
                        };
                }
                VTInputCapture::None
            }
            VTCaptureEvent::VTEvent(vt_event @ VTEvent::Csi(csi)) => {
                parse_csi(vt_event, csi, state, cb)
            }
            VTCaptureEvent::VTEvent(vt_event @ VTEvent::C0(byte)) => {
                parse_c0(vt_event, *byte, state, cb)
            }
            VTCaptureEvent::VTEvent(vt_event @ VTEvent::Esc(esc)) => {
                parse_esc(vt_event, *esc, state, cb)
            }
            VTCaptureEvent::VTEvent(vt_event @ VTEvent::EscInvalid(esc)) => {
                parse_esc_invalid(vt_event, *esc, state, cb)
            }
            VTCaptureEvent::VTEvent(vt_event @ VTEvent::Ss3(ss3)) => {
                parse_ss3(vt_event, *ss3, state, cb)
            }
            VTCaptureEvent::VTEvent(VTEvent::OscStart | VTEvent::OscCancel) => {
                state.accum_buffer.clear();
                VTInputCapture::None
            }
            VTCaptureEvent::VTEvent(VTEvent::OscData(data)) => {
                state.accum_buffer.extend_from_slice(data);
                VTInputCapture::None
            }
            VTCaptureEvent::VTEvent(
                vt_event @ VTEvent::OscEnd { data, .. },
            ) => {
                state.accum_buffer.extend_from_slice(data);
                let osc_data = std::mem::take(&mut state.accum_buffer);
                parse_osc(vt_event, &osc_data, cb);
                VTInputCapture::None
            }
            VTCaptureEvent::VTEvent(VTEvent::DcsStart(dcs)) => {
                state.store_dcs_header(dcs);
                state.accum_buffer.clear();
                state.capture_mode = CaptureMode::DcsData;
                VTInputCapture::None
            }
            VTCaptureEvent::VTEvent(VTEvent::DcsCancel) => {
                state.clear_dcs_header();
                state.accum_buffer.clear();
                state.capture_mode = CaptureMode::None;
                VTInputCapture::None
            }
            VTCaptureEvent::VTEvent(VTEvent::DcsData(data)) => {
                if state.capture_mode == CaptureMode::DcsData {
                    state.accum_buffer.extend_from_slice(data);
                }
                VTInputCapture::None
            }
            VTCaptureEvent::VTEvent(vt_event @ VTEvent::DcsEnd(data)) => {
                state.accum_buffer.extend_from_slice(data);
                let dcs_data = std::mem::take(&mut state.accum_buffer);
                if let Some(dcs_header) = state.dcs_header.take() {
                    parse_dcs(vt_event, &dcs_header, &dcs_data, cb);
                    state.clear_dcs_header();
                } else {
                    cb(&crate::event::UnrecognizedInputEvent(vt_event));
                }
                state.accum_buffer.clear();
                state.capture_mode = CaptureMode::None;
                VTInputCapture::None
            }
            VTCaptureEvent::Capture(data) => {
                match state.capture_mode {
                    CaptureMode::BracketedPaste | CaptureMode::DcsData => {
                        state.accum_buffer.extend_from_slice(data);
                    }
                    CaptureMode::MouseEvent => {
                        // Store the captured bytes for mouse event (3 UTF-8 chars)
                        state.accum_buffer.extend_from_slice(data);
                    }
                    CaptureMode::None => {
                        // Unexpected capture data, ignore
                    }
                }
                VTInputCapture::None
            }
            VTCaptureEvent::CaptureEnd => {
                match state.capture_mode {
                    CaptureMode::BracketedPaste => {
                        use crate::event::terminal::BracketedPaste;

                        let paste_data = state.accum_buffer.clone();
                        state.accum_buffer.clear();
                        state.capture_mode = CaptureMode::None;
                        cb(&BracketedPaste(&paste_data));
                    }
                    CaptureMode::MouseEvent => {
                        // Parse the captured bytes as mouse event (3 UTF-8 chars)
                        // Supports both default format (3 single bytes) and multibyte
                        // UTF-8 format (p1005) where values >= 96 are encoded as UTF-8.
                        if let Ok(event) =
                            parse_mouse_event_bytes(&state.accum_buffer)
                        {
                            cb(&event);
                        } else {
                            cb(&crate::event::UnrecognizedInputEvent(
                                &VTEvent::Raw(
                                    &[
                                        format_csi!("M").as_bytes(),
                                        &state.accum_buffer,
                                    ]
                                    .concat(),
                                ),
                            ));
                        }
                        state.accum_buffer.clear();
                        state.capture_mode = CaptureMode::None;
                    }
                    CaptureMode::DcsData | CaptureMode::None => {
                        // DCS data is handled by DcsEnd event, not CaptureEnd
                        // Unexpected capture end, ignore
                    }
                }
                VTInputCapture::None
            }
            VTCaptureEvent::VTEvent(vt_event) => {
                cb(&crate::event::UnrecognizedInputEvent(vt_event));
                VTInputCapture::None
            }
        }
    }

    /// Handle idle state by flushing any incomplete escape sequences.
    ///
    /// This method should be called when input has stopped but the parser
    /// may be in the middle of processing an escape sequence. It will emit
    /// any appropriate events for incomplete sequences and reset the parser
    /// to ground state.
    ///
    /// Return `true` if any events were emitted, `false` otherwise.
    pub fn idle<F>(&mut self, cb: &mut F) -> bool
    where
        F: FnMut(&dyn vtansi::AnsiEvent),
    {
        if let Some(vt_event) = self.seq_parser.idle() {
            Self::process_vt_event(&vt_event, &mut self.state, cb);
            true
        } else {
            false
        }
    }
}

fn parse_c0<F>(
    vt_event: &VTEvent,
    c0_byte: u8,
    _state: &mut ParserState,
    cb: &mut F,
) -> VTInputCapture
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    if !common::parse_c0(
        c0_byte,
        vtansi::registry::ansi_input_c0_trie_cursor,
        cb,
    ) {
        cb(&UnrecognizedInputEvent(vt_event));
    }
    VTInputCapture::None
}

fn parse_ss3<F>(
    vt_event: &VTEvent,
    ss3: vt_push_parser::event::SS3,
    _state: &mut ParserState,
    cb: &mut F,
) -> VTInputCapture
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    let mut cursor = vtansi::registry::ansi_input_ss3_trie_cursor();

    // First try to match with the actual data byte (for specific handlers)
    if common::maybe_handle_byte(&mut cursor, ss3.char, cb) {
        return VTInputCapture::None;
    }

    // Then try matching with \0 placeholder (for generic handlers like Ss3KeySeq)
    let mut cursor = vtansi::registry::ansi_input_ss3_trie_cursor();
    cursor.advance(0); // Advance with \0 placeholder
    if common::maybe_handle_data(
        &cursor,
        &AnsiEventData::new_with_data(std::slice::from_ref(&ss3.char)),
        cb,
    ) {
        return VTInputCapture::None;
    }

    cb(&UnrecognizedInputEvent(vt_event));
    VTInputCapture::None
}

/// Maximum number of params we support for CSI sequences.
/// This matches the `SmallVec` capacity used in vt-push-parser.
const MAX_CSI_PARAMS: usize = 16;

#[allow(clippy::too_many_lines)]
fn parse_csi<F>(
    vt_event: &VTEvent,
    seq: &vt_push_parser::event::CSI,
    state: &mut ParserState,
    cb: &mut F,
) -> VTInputCapture
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    let mut capture = VTInputCapture::None;

    // Check for mouse reporting format: CSI M (no private marker, no params)
    // Both default and multibyte (p1005) formats use: ESC [ M btn col row
    // - Default format: each value is a single byte = 32 + actual_value
    // - Multibyte format (p1005): values < 96 are single bytes, values >= 96
    //   are encoded as UTF-8 (codepoint = value + 32)
    // Using CountUtf8(3) handles both formats correctly.
    if seq.final_byte == b'M' && seq.private.is_none() && seq.params.is_empty()
    {
        state.accum_buffer.clear();
        state.capture_mode = CaptureMode::MouseEvent;
        return VTInputCapture::CountUtf8(3);
    }

    // Wrapper callback that intercepts BracketedPasteStart to set up capture mode
    let mut intercept_cb = |event: &dyn vtansi::AnsiEvent| {
        if event.downcast_ref::<BracketedPasteStart>().is_some() {
            state.accum_buffer.clear();
            state.capture_mode = CaptureMode::BracketedPaste;
            capture = VTInputCapture::Terminator(BracketedPasteEnd::BYTES);
        } else {
            cb(event);
        }
    };

    // Collect params into a stack-allocated array to avoid heap allocation.
    let mut params_storage: [&[u8]; MAX_CSI_PARAMS] = [&[]; MAX_CSI_PARAMS];
    let mut param_count = 0;
    for param in &seq.params {
        if param_count < MAX_CSI_PARAMS {
            params_storage[param_count] = param;
            param_count += 1;
        }
    }
    let all_params = &params_storage[..param_count];

    let finalbyte_slice = std::slice::from_ref(&seq.final_byte);
    let data = AnsiEventData::new_with_finalbyte(finalbyte_slice);

    let mut cursor = vtansi::registry::ansi_input_csi_trie_cursor();

    // Advance with private marker if present
    if let Some(private) = seq.private
        && !cursor.advance(private).is_prefix()
    {
        cb(&UnrecognizedInputEvent(vt_event));
        return capture;
    }

    // Advance with has_params marker
    if matches!(
        cursor.advance((!all_params.is_empty()).into()),
        Answer::DeadEnd
    ) {
        cb(&UnrecognizedInputEvent(vt_event));
        return capture;
    }

    // Advance through intermediate bytes for key matching
    let intermediates = seq.intermediates.as_ref();
    if !intermediates.is_empty()
        && cursor.advance_slice(intermediates) == Answer::DeadEnd
    {
        cb(&UnrecognizedInputEvent(vt_event));
        return capture;
    }

    // Advance with final byte
    let mut finalbyte_handler: Option<vtansi::registry::Handler> = None;

    match cursor.advance(seq.final_byte) {
        Answer::DeadEnd => {
            cb(&UnrecognizedInputEvent(vt_event));
            return capture;
        }
        Answer::Match(handler) => {
            if handler(&data.with_params(all_params), &mut intercept_cb).is_ok()
            {
                return capture;
            }
            cb(&UnrecognizedInputEvent(vt_event));
            return capture;
        }
        Answer::Prefix => (),
        Answer::PrefixAndMatch(handler) => {
            // Try the handler immediately - it might work with our params
            if handler(&data.with_params(all_params), &mut intercept_cb).is_ok()
            {
                return capture;
            }
            finalbyte_handler = Some(*handler);
        }
    }

    // Walk through params in the trie
    let mut param_prefix_handler: Option<vtansi::registry::Handler> = None;
    let mut consumed_params = 0;

    while consumed_params < param_count {
        let param = all_params[consumed_params];
        match cursor.advance_slice(param) {
            Answer::DeadEnd => break,
            Answer::Prefix => {
                consumed_params += 1;
            }
            answer @ (Answer::Match(handler)
            | Answer::PrefixAndMatch(handler)) => {
                consumed_params += 1;
                let static_params = &all_params[..consumed_params];
                let remaining_params = &all_params[consumed_params..];
                if handler(
                    &data
                        .with_params(remaining_params)
                        .with_static_params(static_params),
                    &mut intercept_cb,
                )
                .is_ok()
                {
                    return capture;
                }
                if answer.is_prefix() {
                    // Handler failed, save it as fallback and continue walking
                    param_prefix_handler = Some(*handler);
                } else {
                    break;
                }
            }
        }
    }

    let static_params = &all_params[..consumed_params];
    let remaining_params = &all_params[consumed_params..];

    if let Some(handler) = param_prefix_handler
        && handler(
            &data
                .with_params(remaining_params)
                .with_static_params(static_params),
            &mut intercept_cb,
        )
        .is_ok()
    {
        return capture;
    }

    if let Some(handler) = finalbyte_handler
        && handler(&data.with_params(all_params), &mut intercept_cb).is_ok()
    {
        return capture;
    }

    cb(&UnrecognizedInputEvent(vt_event));
    capture
}

fn parse_osc<F>(vt_event: &VTEvent, osc_data: &[u8], cb: &mut F)
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    if !common::parse_osc(
        osc_data,
        vtansi::registry::ansi_input_osc_trie_cursor,
        cb,
    ) {
        cb(&UnrecognizedInputEvent(vt_event));
    }
}

fn parse_dcs<F>(
    vt_event: &VTEvent,
    dcs_header: &DCSOwned,
    dcs_data: &[u8],
    cb: &mut F,
) where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    if !common::parse_dcs_owned(
        dcs_header,
        dcs_data,
        vtansi::registry::ansi_input_dcs_trie_cursor,
        cb,
    ) {
        cb(&UnrecognizedInputEvent(vt_event));
    }
}

fn parse_esc<F>(
    vt_event: &VTEvent,
    seq: vt_push_parser::event::Esc,
    _state: &mut ParserState,
    cb: &mut F,
) -> VTInputCapture
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    if !common::parse_esc(seq, vtansi::registry::ansi_input_esc_trie_cursor, cb)
    {
        cb(&UnrecognizedInputEvent(vt_event));
    }
    VTInputCapture::None
}

fn parse_esc_invalid<F>(
    vt_event: &VTEvent,
    seq: vt_push_parser::event::EscInvalid,
    _state: &mut ParserState,
    cb: &mut F,
) -> VTInputCapture
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    if !common::parse_esc_invalid(
        seq,
        vtansi::registry::ansi_input_esc_trie_cursor,
        cb,
    ) {
        cb(&UnrecognizedInputEvent(vt_event));
    }
    VTInputCapture::None
}

#[inline]
pub fn bytes_to_events<F>(
    bytes: &[u8],
    utf8_buffer: &mut [u8],
    cb: &mut F,
) -> usize
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    common::bytes_to_events(bytes, utf8_buffer, cb)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::mouse::{MouseButton, MouseEvent, MouseEventKind};
    use crate::event::terminal::BracketedPaste;
    use crate::event::{KeyCode, KeyEvent, KeyModifiers};
    use better_any::TidExt;

    #[test]
    fn test_mouse_events_default_format() {
        // Default mouse format: ESC [ M btn col row
        // where btn, col, row are raw bytes with value = 32 + actual_value
        let mut parser = TerminalInputParser::new();
        let mut events: Vec<MouseEvent> = Vec::new();

        // Test left button click at column 10, row 5
        // btn = 32 + 0 (left button) = 32 = 0x20
        // col = 32 + 10 = 42 = 0x2A
        // row = 32 + 5 = 37 = 0x25
        parser.feed_with(b"\x1b[M\x20\x2A\x25", &mut |event| {
            if let Some(mouse_event) = event.downcast_ref::<MouseEvent>() {
                events.push(*mouse_event);
            }
        });
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                ..
            }
        ));
        assert_eq!(events[0].col(), 9); // 0-based
        assert_eq!(events[0].row(), 4); // 0-based
    }

    #[test]
    fn test_mouse_events_utf8_multibyte_format() {
        // UTF-8 multibyte mouse format (p1005): ESC [ M btn col row
        // where btn, col, row are encoded as UTF-8 characters.
        // Values < 96 are single bytes (same as default format).
        // Values >= 96 are encoded as 2-byte UTF-8 (codepoint = value + 32).
        let mut parser = TerminalInputParser::new();
        let mut events: Vec<MouseEvent> = Vec::new();

        // Test 1: Left button click at column 100, row 50
        // btn = 32 + 0 (left button) = 32 = 0x20 (single byte)
        // col = 100 + 32 = 132 = U+0084 = 0xC2 0x84 (two-byte UTF-8)
        // row = 50 + 32 = 82 = 0x52 (single byte, < 128)
        parser.feed_with(b"\x1b[M\x20\xC2\x84\x52", &mut |event| {
            if let Some(mouse_event) = event.downcast_ref::<MouseEvent>() {
                events.push(*mouse_event);
            }
        });
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                ..
            }
        ));
        assert_eq!(events[0].col(), 99); // 0-based (100 - 1)
        assert_eq!(events[0].row(), 49); // 0-based (50 - 1)

        // Test 2: Right button click at column 200, row 150
        // btn = 32 + 2 (right button) = 34 = 0x22 (single byte)
        // col = 200 + 32 = 232 = U+00E8 = 0xC3 0xA8 (two-byte UTF-8)
        // row = 150 + 32 = 182 = U+00B6 = 0xC2 0xB6 (two-byte UTF-8)
        events.clear();
        parser.feed_with(b"\x1b[M\x22\xC3\xA8\xC2\xB6", &mut |event| {
            if let Some(mouse_event) = event.downcast_ref::<MouseEvent>() {
                events.push(*mouse_event);
            }
        });
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Right),
                ..
            }
        ));
        assert_eq!(events[0].col(), 199); // 0-based (200 - 1)
        assert_eq!(events[0].row(), 149); // 0-based (150 - 1)

        // Test 3: Large coordinates near max range (2015)
        // btn = 32 + 0 = 32 = 0x20
        // col = 2000 + 32 = 2032 = U+07F0 = 0xDF 0xB0 (two-byte UTF-8)
        // row = 1000 + 32 = 1032 = U+0408 = 0xD0 0x88 (two-byte UTF-8)
        events.clear();
        parser.feed_with(b"\x1b[M\x20\xDF\xB0\xD0\x88", &mut |event| {
            if let Some(mouse_event) = event.downcast_ref::<MouseEvent>() {
                events.push(*mouse_event);
            }
        });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].col(), 1999); // 0-based (2000 - 1)
        assert_eq!(events[0].row(), 999); // 0-based (1000 - 1)
    }

    fn collect_key_events(input: &[u8]) -> Vec<KeyEvent> {
        let mut parser = TerminalInputParser::new();
        let mut events = Vec::new();
        parser.feed_with(input, &mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });
        parser.idle(&mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });
        events
    }

    fn collect_key_events_with_idle(chunks: &[&[u8]]) -> Vec<KeyEvent> {
        let mut parser = TerminalInputParser::new();
        let mut events = Vec::new();
        for chunk in chunks {
            parser.feed_with(chunk, &mut |event| {
                if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                    events.push(key_event.clone());
                }
            });
            parser.idle(&mut |event| {
                if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                    events.push(key_event.clone());
                }
            });
        }
        events
    }

    #[test]
    fn test_basic_text_input() {
        let events = collect_key_events(b"hello");
        assert_eq!(events.len(), 5);
        for (i, ch) in "hello".chars().enumerate() {
            assert!(matches!(
                &events[i],
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers,
                    ..
                } if *c == ch && modifiers.is_empty()
            ));
        }
    }

    #[test]
    fn test_escape_key_with_idle() {
        // Test that a lone ESC byte followed by idle becomes an Escape key
        // event
        let mut parser = TerminalInputParser::new();
        let mut events: Vec<KeyEvent> = Vec::new();

        parser.feed_with(b"\x1b", &mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });
        // Without idle, the parser is waiting to see if more bytes follow
        assert_eq!(events.len(), 0);

        parser.idle(&mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });
        // After idle, the ESC is emitted as an Escape key
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            KeyEvent {
                code: KeyCode::Esc,
                modifiers,
                ..
            } if modifiers.is_empty()
        ));
    }

    #[test]
    fn test_alt_enter() {
        // ESC followed immediately by Enter should be Alt+Enter
        let events = collect_key_events(b"\x1b\n");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            KeyEvent {
                code: KeyCode::Enter,
                modifiers,
                ..
            } if modifiers.contains(KeyModifiers::ALT)
        ));
    }

    #[test]
    fn test_escape_key_followed_by_text() {
        // ESC with idle, then text - should get separate Escape key and text
        let events = collect_key_events_with_idle(&[b"\x1b", b"hi\r"]);
        assert_eq!(events.len(), 4);
        assert!(matches!(
            &events[0],
            KeyEvent {
                code: KeyCode::Esc,
                ..
            }
        ));
        assert!(matches!(
            &events[1],
            KeyEvent {
                code: KeyCode::Char('h'),
                ..
            }
        ));
        assert!(matches!(
            &events[2],
            KeyEvent {
                code: KeyCode::Char('i'),
                ..
            }
        ));
        assert!(matches!(
            &events[3],
            KeyEvent {
                code: KeyCode::Enter,
                ..
            }
        ));
    }

    #[test]
    fn test_arrow_key() {
        let events = collect_key_events(b"\x1b[A");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            KeyEvent {
                code: KeyCode::Up,
                modifiers,
                ..
            } if modifiers.is_empty()
        ));
    }

    #[test]
    fn test_emoji_input() {
        let events = collect_key_events("🤣🛜".as_bytes());
        assert_eq!(events.len(), 2);
        assert!(matches!(
            &events[0],
            KeyEvent {
                code: KeyCode::Char('🤣'),
                ..
            }
        ));
        assert!(matches!(
            &events[1],
            KeyEvent {
                code: KeyCode::Char('🛜'),
                ..
            }
        ));
    }

    #[test]
    fn test_mouse_events_sgr_format() {
        let mut parser = TerminalInputParser::new();
        let mut events: Vec<MouseEvent> = Vec::new();
        parser.feed_with(b"\x1b[<35;73;5M\x1b[<35;73;5M", &mut |event| {
            if let Some(mouse_event) = event.downcast_ref::<MouseEvent>() {
                events.push(*mouse_event);
            }
        });
        assert_eq!(events.len(), 2);
        assert!(matches!(
            &events[0],
            MouseEvent {
                kind: MouseEventKind::Moved,
                ..
            }
        ));
        assert_eq!(events[0].col(), 72);
        assert_eq!(events[0].row(), 4);
        assert!(matches!(
            &events[1],
            MouseEvent {
                kind: MouseEventKind::Moved,
                ..
            }
        ));
        assert_eq!(events[1].col(), 72);
        assert_eq!(events[1].row(), 4);
    }

    #[test]
    fn test_mouse_events_urxvt_format() {
        // urxvt format: ESC [ btn ; col ; row M
        // where btn is offset by 32 (like default format)
        let mut parser = TerminalInputParser::new();
        let mut events: Vec<MouseEvent> = Vec::new();

        // Test left button click at column 10, row 5
        // btn = 32 + 0 (left button) = 32
        // col = 10, row = 5
        parser.feed_with(b"\x1b[32;10;5M", &mut |event| {
            if let Some(mouse_event) = event.downcast_ref::<MouseEvent>() {
                events.push(*mouse_event);
            }
        });
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                ..
            }
        ));
        assert_eq!(events[0].col(), 9); // 0-based
        assert_eq!(events[0].row(), 4); // 0-based

        // Test right button click at column 200, row 150 (large coordinates)
        // btn = 32 + 2 (right button) = 34
        events.clear();
        parser.feed_with(b"\x1b[34;200;150M", &mut |event| {
            if let Some(mouse_event) = event.downcast_ref::<MouseEvent>() {
                events.push(*mouse_event);
            }
        });
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Right),
                ..
            }
        ));
        assert_eq!(events[0].col(), 199);
        assert_eq!(events[0].row(), 149);

        // Test scroll up
        // btn = 32 + 64 (scroll up) = 96
        events.clear();
        parser.feed_with(b"\x1b[96;10;5M", &mut |event| {
            if let Some(mouse_event) = event.downcast_ref::<MouseEvent>() {
                events.push(*mouse_event);
            }
        });
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            MouseEvent {
                kind: MouseEventKind::ScrollUp,
                ..
            }
        ));
    }

    #[test]
    #[cfg(unix)]
    fn test_cursor_position_report() {
        use crate::event::cursor::CursorPositionReport;
        let mut parser = TerminalInputParser::new();
        let mut report: Option<CursorPositionReport> = None;
        parser.feed_with(b"\x1b[3;1R", &mut |event| {
            if let Some(cpr) = event.downcast_ref::<CursorPositionReport>() {
                report = Some(*cpr);
            }
        });
        // CSI 3;1R means row 3, column 1 (1-indexed)
        let r = report.expect("Expected CursorPositionReport");
        assert_eq!(r.row, 3);
        assert_eq!(r.col, 1);
    }

    /// Test that `ESC [ 1 ; 1 R` (0x1b 0x5b 0x31 0x3b 0x31 0x52) is parsed as
    /// `CursorPositionReport`, NOT as F3 key.
    ///
    /// This is a regression test for a bug where the CSI function key fallback
    /// would incorrectly match sequences with parameters as function keys.
    #[test]
    #[cfg(unix)]
    fn test_cpr_not_misparse_as_f3() {
        use crate::event::cursor::CursorPositionReport;

        let mut parser = TerminalInputParser::new();
        let mut events = Vec::new();

        // ESC [ 1 ; 1 R is CPR (Cursor Position Report) with row=1, col=1
        parser.feed_with(b"\x1b[1;1R", &mut |event| {
            if let Some(cpr) = event.downcast_ref::<CursorPositionReport>() {
                events.push(CursorPositionReport {
                    row: cpr.row,
                    col: cpr.col,
                });
            }
        });

        assert_eq!(events, vec![CursorPositionReport { row: 1, col: 1 }]);
    }

    #[test]
    #[cfg(unix)]
    fn test_device_attributes() {
        // Device attributes response - test that it parses without error
        let mut parser = TerminalInputParser::new();
        let mut event_count = 0;
        parser.feed_with(b"\x1b[?62;22;52c", &mut |_event| {
            event_count += 1;
        });
        assert!(event_count >= 1);
    }

    #[test]
    #[cfg(unix)]
    fn test_keyboard_enhancement_flags() {
        use crate::event::keyboard::KeyboardEnhancementFlagsResponse;
        let mut parser = TerminalInputParser::new();
        let mut found = false;
        parser.feed_with(b"\x1b[?0u", &mut |event| {
            if event
                .downcast_ref::<KeyboardEnhancementFlagsResponse>()
                .is_some()
            {
                found = true;
            }
        });
        assert!(found);
    }

    #[test]
    #[cfg(unix)]
    fn test_cursor_keys_mode_response() {
        use crate::event::keyboard::CursorKeysMode;
        use crate::event::mode::TerminalModeState;

        let mut parser = TerminalInputParser::new();
        let mut found = false;
        let mut mode_state = None;

        // CSI ? 1 ; 2 $ y means CursorKeysMode with state 2 (Reset)
        // DECRPM response format: CSI ? Ps ; Pm $ y
        // Ps = mode number (1 = DECCKM/CursorKeysMode)
        // Pm = state (0=not recognized, 1=set, 2=reset, 3=permanently set, 4=permanently reset)
        parser.feed_with(b"\x1b[?1;2$y", &mut |event| {
            if let Some(mode) = event.downcast_ref::<CursorKeysMode>() {
                found = true;
                mode_state = Some(mode.state);
            }
        });

        assert!(
            found,
            "CursorKeysMode should be parsed from DECRPM response"
        );
        assert_eq!(mode_state, Some(TerminalModeState::Reset));
    }

    #[test]
    #[cfg(unix)]
    fn test_cursor_blinking_mode_response() {
        use crate::event::cursor::CursorBlinking;
        use crate::event::mode::TerminalModeState;

        let mut parser = TerminalInputParser::new();
        let mut found = false;
        let mut mode_state = None;

        // CSI ? 12 ; 1 $ y means CursorBlinking with state 1 (Set)
        // This tests that mode 12 doesn't conflict with mode 1 parsing
        parser.feed_with(b"\x1b[?12;1$y", &mut |event| {
            if let Some(mode) = event.downcast_ref::<CursorBlinking>() {
                found = true;
                mode_state = Some(mode.state);
            }
        });

        assert!(
            found,
            "CursorBlinking should be parsed from DECRPM response"
        );
        assert_eq!(mode_state, Some(TerminalModeState::Set));
    }

    #[test]
    #[cfg(unix)]
    fn test_bracketed_paste_mode_response() {
        use crate::event::mode::TerminalModeState;
        use crate::event::terminal::BracketedPasteMode;

        let mut parser = TerminalInputParser::new();
        let mut found = false;
        let mut mode_state = None;

        // CSI ? 2004 ; 2 $ y means BracketedPasteMode with state 2 (Reset)
        parser.feed_with(b"\x1b[?2004;2$y", &mut |event| {
            if let Some(mode) = event.downcast_ref::<BracketedPasteMode>() {
                found = true;
                mode_state = Some(mode.state);
            }
        });

        assert!(
            found,
            "BracketedPasteMode should be parsed from DECRPM response"
        );
        assert_eq!(mode_state, Some(TerminalModeState::Reset));
    }

    #[test]
    #[cfg(unix)]
    fn test_keyboard_enhancement_query() {
        // Query response - test that it parses without error
        let mut parser = TerminalInputParser::new();
        let mut event_count = 0;
        parser.feed_with(b"\x1b[?u", &mut |_event| {
            event_count += 1;
        });
        assert!(event_count >= 1);
    }

    #[test]
    #[cfg(unix)]
    fn test_keyboard_enhancement_push() {
        // Push keyboard enhancement - test that it parses without error
        let mut parser = TerminalInputParser::new();
        let mut event_count = 0;
        parser.feed_with(b"\x1b[>1u", &mut |_event| {
            event_count += 1;
        });
        assert!(event_count >= 1);
    }

    #[test]
    #[cfg(unix)]
    fn test_keyboard_enhancement_pop() {
        // Pop keyboard enhancement - test that it parses without error
        let mut parser = TerminalInputParser::new();
        let mut event_count = 0;
        parser.feed_with(b"\x1b[<u", &mut |_event| {
            event_count += 1;
        });
        assert!(event_count >= 1);
    }

    #[test]
    fn test_low_level_csi_event() {
        // Random CSI that doesn't map to a known event
        let mut parser = TerminalInputParser::new();
        let mut found_unrecognized = false;
        // Use 'z' as final byte - it's not registered as a known CSI sequence
        parser.feed_with(b"\x1b[12z", &mut |event| {
            if let Some(unrecognized) =
                event.downcast_ref::<UnrecognizedInputEvent>()
                && let VTEvent::Csi(csi) = unrecognized.0
            {
                assert_eq!(csi.final_byte, b'z');
                found_unrecognized = true;
            }
        });
        assert!(found_unrecognized);
    }

    #[test]
    fn test_osc_sequences() {
        // OSC sequences - test that they parse without error
        let mut parser = TerminalInputParser::new();
        let mut event_count = 0;
        parser.feed_with(b"\x1b]0;test\x07", &mut |_event| {
            event_count += 1;
        });
        // OSC sequences emit events (OscStart and OscEnd)
        assert!(event_count >= 1);
    }

    #[test]
    fn test_osc_and_keyboard_enhancement() {
        // OSC sequence followed by keyboard enhancement flags
        let mut parser = TerminalInputParser::new();
        let mut event_count = 0;
        parser.feed_with(b"\x1b]0;test\x07\x1b[?0u", &mut |_event| {
            event_count += 1;
        });
        // Should emit multiple events
        assert!(event_count >= 2);
    }

    #[test]
    #[cfg(unix)]
    fn test_complex_sequence_combination() {
        use crate::event::cursor::CursorPositionReport;
        // Test cursor position, device attributes, and OSC sequences
        let mut parser = TerminalInputParser::new();
        let mut cursor_report: Option<CursorPositionReport> = None;
        let mut event_count = 0;

        parser.feed_with(
            b"\x1b[3;1R\x1b[>1;10;0c\x1b]10;rgb:ffff/ffff/ffff\x07\x1b]11;rgb:2828/2c2c/3434\x07",
            &mut |event| {
                event_count += 1;
                if let Some(cpr) = event.downcast_ref::<CursorPositionReport>() {
                    cursor_report = Some(*cpr);
                }
            },
        );

        // Should receive multiple events
        assert!(event_count >= 4);

        // First should be cursor position report
        let cpr = cursor_report.expect("Expected CursorPositionReport");
        assert_eq!(cpr.row, 3);
        assert_eq!(cpr.col, 1);
    }

    #[test]
    fn test_bracketed_paste_basic() {
        let mut parser = TerminalInputParser::new();
        let mut paste_data: Option<Vec<u8>> = None;
        parser.feed_with(b"\x1b[200~Hello World\x1b[201~", &mut |event| {
            if let Some(paste) = event.downcast_ref::<BracketedPaste>() {
                paste_data = Some(paste.0.to_vec());
            }
        });
        assert_eq!(paste_data, Some(b"Hello World".to_vec()));
    }

    #[test]
    fn test_bracketed_paste_with_escape_sequences() {
        // Test that escape sequences inside paste are treated as raw content
        let mut parser = TerminalInputParser::new();
        let mut paste_data: Option<Vec<u8>> = None;

        // Paste containing an arrow key sequence - should be treated as raw
        // bytes
        let input = b"\x1b[200~text\x1b[A\x1b[201~";
        parser.feed_with(input, &mut |event| {
            if let Some(paste) = event.downcast_ref::<BracketedPaste>() {
                paste_data = Some(paste.0.to_vec());
            }
        });

        // The ESC[A should be included as raw content in the paste
        assert_eq!(paste_data, Some(b"text\x1b[A".to_vec()));
    }

    #[test]
    fn test_bracketed_paste_with_csi_sequence() {
        // Test that CSI sequences in paste are treated as raw content
        let mut parser = TerminalInputParser::new();
        let mut paste_data: Option<Vec<u8>> = None;

        // Start paste, add some data including CSI sequence, then end paste
        let input = b"\x1b[200~partial\x1b[A\x1b[201~";
        parser.feed_with(input, &mut |event| {
            if let Some(paste) = event.downcast_ref::<BracketedPaste>() {
                paste_data = Some(paste.0.to_vec());
            }
        });

        // Should emit paste with the CSI sequence as literal data
        assert_eq!(paste_data, Some(b"partial\x1b[A".to_vec()));
    }

    #[test]
    fn test_bracketed_paste_with_ss3_sequence() {
        // Test that SS3 sequences in paste are treated as raw content
        let mut parser = TerminalInputParser::new();
        let mut paste_data: Option<Vec<u8>> = None;

        // Start paste, add data including SS3 sequence, then end paste
        let input = b"\x1b[200~some text\x1bOH\x1b[201~";
        parser.feed_with(input, &mut |event| {
            if let Some(paste) = event.downcast_ref::<BracketedPaste>() {
                paste_data = Some(paste.0.to_vec());
            }
        });

        // ESC O H should be included as raw content, not interpreted as Home
        // key
        assert_eq!(paste_data, Some(b"some text\x1bOH".to_vec()));
    }

    #[test]
    fn test_bracketed_paste_with_newlines() {
        let mut parser = TerminalInputParser::new();
        let mut paste_data: Option<Vec<u8>> = None;
        parser.feed_with(
            b"\x1b[200~Line1\nLine2\r\nLine3\tTab\x1b[201~",
            &mut |event| {
                if let Some(paste) = event.downcast_ref::<BracketedPaste>() {
                    paste_data = Some(paste.0.to_vec());
                }
            },
        );
        assert_eq!(paste_data, Some(b"Line1\nLine2\r\nLine3\tTab".to_vec()));
    }

    #[test]
    fn test_bracketed_paste_large_content() {
        let long_content = "A".repeat(10000);
        let mut test_data = b"\x1b[200~".to_vec();
        test_data.extend_from_slice(long_content.as_bytes());
        test_data.extend_from_slice(b"\x1b[201~");

        let mut parser = TerminalInputParser::new();
        let mut paste_data: Option<Vec<u8>> = None;
        parser.feed_with(&test_data, &mut |event| {
            if let Some(paste) = event.downcast_ref::<BracketedPaste>() {
                paste_data = Some(paste.0.to_vec());
            }
        });
        assert_eq!(paste_data, Some(long_content.into_bytes()));
    }

    #[test]
    fn test_bracketed_paste_multiple_chunks() {
        // Test chunked paste input
        let mut parser = TerminalInputParser::new();
        let mut paste_data: Option<Vec<u8>> = None;

        parser.feed_with(b"\x1b[200~Chunk1", &mut |event| {
            if let Some(paste) = event.downcast_ref::<BracketedPaste>() {
                paste_data = Some(paste.0.to_vec());
            }
        });
        parser.feed_with(b"Chunk2\x1b[201~", &mut |event| {
            if let Some(paste) = event.downcast_ref::<BracketedPaste>() {
                paste_data = Some(paste.0.to_vec());
            }
        });

        assert_eq!(paste_data, Some(b"Chunk1Chunk2".to_vec()));
    }

    #[test]
    fn test_incomplete_utf8_across_chunks() {
        // Test UTF-8 handling when a multibyte character is split across
        // chunks
        let mut parser = TerminalInputParser::new();
        let mut events: Vec<KeyEvent> = Vec::new();

        // Split the emoji '🤣' (F0 9F A4 A3) across two chunks
        parser.feed_with(&[0xF0, 0x9F], &mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });
        // Should not emit anything yet
        assert_eq!(events.len(), 0);

        parser.feed_with(&[0xA4, 0xA3], &mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });
        // Now should emit the complete character
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            KeyEvent {
                code: KeyCode::Char('🤣'),
                ..
            }
        ));
    }

    #[test]
    fn test_idle_resets_incomplete_sequence() {
        // Test that calling idle handles incomplete escape sequences properly
        let mut parser = TerminalInputParser::new();
        let mut event_count = 0;

        // Start an escape sequence but don't complete it
        parser.feed_with(b"\x1b", &mut |_event| {
            event_count += 1;
        });

        // Incomplete escape - no events emitted yet
        assert_eq!(event_count, 0);

        // Call idle - this should emit the escape as a standalone event
        parser.idle(&mut |_event| {
            event_count += 1;
        });

        // idle should have emitted the pending escape
        assert!(event_count > 0);

        event_count = 0;

        // Additional input after idle - CSI sequence should parse correctly
        parser.feed_with(b"\x1b[3;1R", &mut |_event| {
            event_count += 1;
        });

        // Should have at least one event (cursor position report)
        assert!(event_count > 0);
    }

    #[test]
    fn test_ss3_f_key_sequences() {
        use crate::event::keyboard::{KeyCode, KeyEvent, KeyModifiers};

        let mut parser = TerminalInputParser::new();
        let mut events: Vec<KeyEvent> = Vec::new();

        // Test F1 key (ESC O P)
        parser.feed_with(b"\x1bOP", &mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].code, KeyCode::F(1));
        assert_eq!(events[0].modifiers, KeyModifiers::NONE);

        events.clear();

        // Test F2 key (ESC O Q)
        parser.feed_with(b"\x1bOQ", &mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].code, KeyCode::F(2));

        events.clear();

        // Test F3 key (ESC O R)
        parser.feed_with(b"\x1bOR", &mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].code, KeyCode::F(3));

        events.clear();

        // Test F4 key (ESC O S)
        parser.feed_with(b"\x1bOS", &mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].code, KeyCode::F(4));
    }

    #[test]
    fn test_ss3_cursor_keys() {
        use crate::event::keyboard::{KeyCode, KeyEvent, KeyModifiers};

        let mut parser = TerminalInputParser::new();
        let mut events: Vec<KeyEvent> = Vec::new();

        // Test Up arrow in application mode (ESC O A)
        parser.feed_with(b"\x1bOA", &mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].code, KeyCode::Up);
        assert_eq!(events[0].modifiers, KeyModifiers::NONE);

        events.clear();

        // Test Down arrow (ESC O B)
        parser.feed_with(b"\x1bOB", &mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].code, KeyCode::Down);

        events.clear();

        // Test Right arrow (ESC O C)
        parser.feed_with(b"\x1bOC", &mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].code, KeyCode::Right);

        events.clear();

        // Test Left arrow (ESC O D)
        parser.feed_with(b"\x1bOD", &mut |event| {
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                events.push(key_event.clone());
            }
        });

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].code, KeyCode::Left);
    }

    // ==========================================================================
    // OSC Sequence Tests
    // ==========================================================================

    #[test]
    fn test_osc_palette_color_response() {
        use crate::event::color::TerminalPaletteColorResponse;

        let mut parser = TerminalInputParser::new();
        let mut response: Option<TerminalPaletteColorResponse> = None;

        // OSC 4 ; 0 ; rgb:0000/0000/0000 BEL - palette color 0 response (black)
        parser.feed_with(b"\x1b]4;0;rgb:0000/0000/0000\x07", &mut |event| {
            if let Some(r) =
                event.downcast_ref::<TerminalPaletteColorResponse>()
            {
                response = Some(*r);
            }
        });

        let r = response.expect("Expected TerminalPaletteColorResponse");
        assert_eq!(r.index, 0);
        let (red, green, blue) = r.color.as_rgb().expect("Expected RGB color");
        assert_eq!(red, 0x0000);
        assert_eq!(green, 0x0000);
        assert_eq!(blue, 0x0000);
    }

    #[test]
    fn test_osc_foreground_color_response() {
        use crate::event::color::SpecialTextForegroundColorResponse;

        let mut parser = TerminalInputParser::new();
        let mut response: Option<SpecialTextForegroundColorResponse> = None;

        // OSC 10 ; rgb:ffff/ffff/ffff BEL - foreground color response (white)
        parser.feed_with(b"\x1b]10;rgb:ffff/ffff/ffff\x07", &mut |event| {
            if let Some(r) =
                event.downcast_ref::<SpecialTextForegroundColorResponse>()
            {
                response = Some(*r);
            }
        });

        let r = response.expect("Expected SpecialTextForegroundColorResponse");
        let (red, green, blue) = r.as_rgb().expect("Expected RGB color");
        assert_eq!(red, 0xffff);
        assert_eq!(green, 0xffff);
        assert_eq!(blue, 0xffff);
    }

    #[test]
    fn test_osc_unrecognized_number() {
        use crate::event::UnrecognizedInputEvent;

        let mut parser = TerminalInputParser::new();
        let mut unrecognized_count = 0;

        // OSC 9999 - unrecognized OSC number
        parser.feed_with(b"\x1b]9999;some data\x07", &mut |event| {
            if event.downcast_ref::<UnrecognizedInputEvent>().is_some() {
                unrecognized_count += 1;
            }
        });

        assert_eq!(unrecognized_count, 1);
    }

    #[test]
    fn test_osc_chunked_parsing() {
        use crate::event::color::SpecialTextForegroundColorResponse;

        let mut parser = TerminalInputParser::new();
        let mut response: Option<SpecialTextForegroundColorResponse> = None;

        // Feed in small chunks
        let chunks: &[&[u8]] = &[
            b"\x1b]", b"10", b";", b"rgb:", b"ff", b"ff/", b"00", b"00/",
            b"ff", b"ff", b"\x07",
        ];

        for chunk in chunks {
            parser.feed_with(chunk, &mut |event| {
                if let Some(r) =
                    event.downcast_ref::<SpecialTextForegroundColorResponse>()
                {
                    response = Some(*r);
                }
            });
        }

        let r = response.expect(
            "Expected SpecialTextForegroundColorResponse after chunked parsing",
        );
        let (red, green, blue) = r.as_rgb().expect("Expected RGB color");
        assert_eq!(red, 0xffff);
        assert_eq!(green, 0x0000);
        assert_eq!(blue, 0xffff);
    }

    #[test]
    fn test_terminal_name_and_version_response() {
        use crate::event::terminal::TerminalNameAndVersionResponse;

        let mut parser = TerminalInputParser::new();
        let mut found = false;
        let mut version_str = None;

        // DCS > | xterm(388) ST
        parser.feed_with(b"\x1bP>|xterm(388)\x1b\\", &mut |event| {
            if let Some(response) =
                event.downcast_ref::<TerminalNameAndVersionResponse>()
            {
                found = true;
                version_str = Some(response.version.to_string());
            }
        });

        assert!(found, "TerminalNameAndVersionResponse should be parsed");
        assert_eq!(version_str, Some("xterm(388)".to_string()));
    }

    #[test]
    fn test_tertiary_device_attributes_response() {
        use crate::event::terminal::TertiaryDeviceAttributesResponse;

        let mut parser = TerminalInputParser::new();
        let mut found = false;
        let mut unit_id_str = None;

        // DCS ! | 7E565445 ST (hex for "~VTE")
        parser.feed_with(b"\x1bP!|7E565445\x1b\\", &mut |event| {
            if let Some(response) =
                event.downcast_ref::<TertiaryDeviceAttributesResponse>()
            {
                found = true;
                unit_id_str = response.data.as_str().map(String::from);
            }
        });

        assert!(found, "TertiaryDeviceAttributesResponse should be parsed");
        assert_eq!(unit_id_str, Some("~VTE".to_string()));
    }

    #[test]
    fn test_osc_cancelled() {
        use crate::event::color::SpecialTextForegroundColorResponse;

        let mut parser = TerminalInputParser::new();
        let mut response_count = 0;

        // Start an OSC sequence but cancel it with CAN (0x18) before terminating
        // Then send a valid OSC sequence
        parser.feed_with(
            b"\x1b]10;rgb:ffff/0000/0000\x18\x1b]10;rgb:0000/ffff/0000\x07",
            &mut |event| {
                if event
                    .downcast_ref::<SpecialTextForegroundColorResponse>()
                    .is_some()
                {
                    response_count += 1;
                }
            },
        );

        // Only the second (non-cancelled) sequence should produce a response
        assert_eq!(response_count, 1);
    }

    #[test]
    fn test_printer_status_report() {
        use crate::event::dsr::{PrinterStatus, PrinterStatusReport};

        let mut parser = TerminalInputParser::new();

        // Test CSI ? 10 n (Printer Ready)
        let mut status: Option<PrinterStatus> = None;
        parser.feed_with(b"\x1b[?10n", &mut |event| {
            if let Some(report) = event.downcast_ref::<PrinterStatusReport>() {
                status = Some(report.status);
            }
        });
        assert_eq!(status, Some(PrinterStatus::Ready));

        // Test CSI ? 11 n (Printer Not Ready)
        status = None;
        parser.feed_with(b"\x1b[?11n", &mut |event| {
            if let Some(report) = event.downcast_ref::<PrinterStatusReport>() {
                status = Some(report.status);
            }
        });
        assert_eq!(status, Some(PrinterStatus::NotReady));

        // Test CSI ? 13 n (No Printer)
        status = None;
        parser.feed_with(b"\x1b[?13n", &mut |event| {
            if let Some(report) = event.downcast_ref::<PrinterStatusReport>() {
                status = Some(report.status);
            }
        });
        assert_eq!(status, Some(PrinterStatus::NoPrinter));
    }

    #[test]
    fn test_keyboard_status_report() {
        use crate::event::dsr::KeyboardStatusReport;

        let mut parser = TerminalInputParser::new();

        // Test CSI ? 27 ; 1 n (keyboard dialect 1)
        let mut dialect: Option<u8> = None;
        parser.feed_with(b"\x1b[?27;1n", &mut |event| {
            if let Some(report) = event.downcast_ref::<KeyboardStatusReport>() {
                dialect = Some(report.dialect);
            }
        });
        assert_eq!(dialect, Some(1));

        // Test CSI ? 27 ; 5 n (keyboard dialect 5)
        dialect = None;
        parser.feed_with(b"\x1b[?27;5n", &mut |event| {
            if let Some(report) = event.downcast_ref::<KeyboardStatusReport>() {
                dialect = Some(report.dialect);
            }
        });
        assert_eq!(dialect, Some(5));
    }

    #[test]
    fn test_memory_checksum_report() {
        use crate::event::dsr::MemoryChecksumReport;

        let mut parser = TerminalInputParser::new();

        // Test CSI ? 63 ; 42 ; 65535 n
        let mut id: Option<u16> = None;
        let mut checksum: Option<u16> = None;
        parser.feed_with(b"\x1b[?63;42;65535n", &mut |event| {
            if let Some(report) = event.downcast_ref::<MemoryChecksumReport>() {
                id = Some(report.id);
                checksum = Some(report.checksum);
            }
        });
        assert_eq!(id, Some(42));
        assert_eq!(checksum, Some(65535));
    }

    #[test]
    fn test_udk_status_report() {
        use crate::event::dsr::{UdkStatus, UdkStatusReport};

        let mut parser = TerminalInputParser::new();

        // Test CSI ? 20 n (UDK Locked)
        let mut status: Option<UdkStatus> = None;
        parser.feed_with(b"\x1b[?20n", &mut |event| {
            if let Some(report) = event.downcast_ref::<UdkStatusReport>() {
                status = Some(report.status);
            }
        });
        assert_eq!(status, Some(UdkStatus::Locked));

        // Test CSI ? 21 n (UDK Unlocked)
        status = None;
        parser.feed_with(b"\x1b[?21n", &mut |event| {
            if let Some(report) = event.downcast_ref::<UdkStatusReport>() {
                status = Some(report.status);
            }
        });
        assert_eq!(status, Some(UdkStatus::Unlocked));
    }

    #[test]
    fn test_locator_status_report() {
        use crate::event::dsr::{LocatorStatus, LocatorStatusReport};

        let mut parser = TerminalInputParser::new();

        // Test CSI ? 50 n (No Locator)
        let mut status: Option<LocatorStatus> = None;
        parser.feed_with(b"\x1b[?50n", &mut |event| {
            if let Some(report) = event.downcast_ref::<LocatorStatusReport>() {
                status = Some(report.status);
            }
        });
        assert_eq!(status, Some(LocatorStatus::NoLocator));

        // Test CSI ? 53 n (Locator Available)
        status = None;
        parser.feed_with(b"\x1b[?53n", &mut |event| {
            if let Some(report) = event.downcast_ref::<LocatorStatusReport>() {
                status = Some(report.status);
            }
        });
        assert_eq!(status, Some(LocatorStatus::Available));
    }

    #[test]
    fn test_operating_status_report() {
        use crate::event::dsr::OperatingStatusReport;

        let mut parser = TerminalInputParser::new();

        // Test CSI 0 n (Operating Status OK)
        let mut found = false;
        parser.feed_with(b"\x1b[0n", &mut |event| {
            if event.downcast_ref::<OperatingStatusReport>().is_some() {
                found = true;
            }
        });
        assert!(found, "OperatingStatusReport should be parsed");
    }

    #[test]
    fn test_operating_status_report_private() {
        use crate::event::dsr::OperatingStatusReportPrivate;

        let mut parser = TerminalInputParser::new();

        // Test CSI ? 0 n (Private Operating Status OK)
        let mut found = false;
        parser.feed_with(b"\x1b[?0n", &mut |event| {
            if event
                .downcast_ref::<OperatingStatusReportPrivate>()
                .is_some()
            {
                found = true;
            }
        });
        assert!(found, "OperatingStatusReportPrivate should be parsed");
    }
}
