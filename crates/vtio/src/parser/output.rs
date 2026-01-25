//! Terminal output parser.
//!
//! This module provides parsing of terminal output sequences (host-to-terminal).
//! It uses the regular `VTPushParser` and output trie cursors to parse
//! CSI, OSC, ESC, DCS, and C0 sequences.

use vt_push_parser::{
    VT_PARSER_INTEREST_ALL, VTPushParser,
    event::{DCSOwned, VTEvent},
};
use vtansi::registry::AnsiEventData;

use crate::event::{PlainText, UnrecognizedOutputEvent};

use super::common;

const MAX_UTF8_CHAR_BYTES: usize = 4;

/// Tracks the type of capture mode currently active.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum CaptureMode {
    #[default]
    None,
    /// Capturing DCS data
    DcsData,
    /// Capturing OSC data
    OscData,
}

#[derive(Debug, Default)]
struct ParserState {
    // Buffer for incomplete UTF-8 sequences from previous feed (max 4 bytes)
    utf8_buffer: [u8; MAX_UTF8_CHAR_BYTES],
    utf8_buffer_len: usize,
    // Accumulator for OSC and DCS data
    capture_buffer: Vec<u8>,
    capture_mode: CaptureMode,
    dcs_header: Option<vt_push_parser::event::DCSOwned>,
}

impl ParserState {
    const fn new() -> Self {
        Self {
            utf8_buffer: [0; MAX_UTF8_CHAR_BYTES],
            utf8_buffer_len: 0,
            capture_buffer: Vec::new(),
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

/// Terminal output parser.
///
/// Parses terminal output sequences (host-to-terminal) and emits
/// typed events via callbacks.
pub struct TerminalOutputParser {
    seq_parser: VTPushParser<VT_PARSER_INTEREST_ALL>,
    state: ParserState,
}

impl Default for TerminalOutputParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalOutputParser {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            seq_parser: VTPushParser::new_with_interest::<VT_PARSER_INTEREST_ALL>(
            ),
            state: ParserState::new(),
        }
    }

    /// Decode a buffer of bytes into a series of events.
    pub fn decode_buffer<F>(input: &[u8], cb: &mut F)
    where
        for<'a> F: FnMut(&dyn vtansi::AnsiEvent),
    {
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(input, cb);
    }

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
        self.seq_parser.feed_with(input, |vt_event: VTEvent| {
            Self::process_vt_event(&vt_event, &mut self.state, cb);
        });
    }

    #[allow(clippy::too_many_lines)]
    fn process_vt_event<F>(
        vt_event: &VTEvent,
        state: &mut ParserState,
        cb: &mut F,
    ) where
        F: FnMut(&dyn vtansi::AnsiEvent),
    {
        match vt_event {
            VTEvent::Raw(bytes) => {
                if state.utf8_buffer_len == 0 {
                    state.utf8_buffer_len = common::bytes_to_plaintext(
                        bytes,
                        &mut state.utf8_buffer,
                        cb,
                    );
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

                    let incomplete_len = common::bytes_to_plaintext(
                        &temp_buf[..buf_len + take],
                        &mut state.utf8_buffer,
                        cb,
                    );

                    state.utf8_buffer_len =
                        if take < bytes.len() && incomplete_len <= take {
                            // Process from start of incomplete sequence (or from take if none)
                            common::bytes_to_plaintext(
                                &bytes[take - incomplete_len..],
                                &mut state.utf8_buffer,
                                cb,
                            )
                        } else {
                            // No more input, or incomplete bytes span buffer boundary
                            incomplete_len
                        };
                }
            }
            VTEvent::Csi(csi) => {
                parse_csi(vt_event, csi, cb);
            }
            VTEvent::C0(byte) => {
                parse_c0(vt_event, *byte, cb);
            }
            VTEvent::Esc(esc) => {
                parse_esc(vt_event, *esc, cb);
            }
            VTEvent::EscInvalid(esc) => {
                parse_esc_invalid(vt_event, *esc, cb);
            }
            VTEvent::Ss3(ss3) => {
                // SS3 is primarily for input; for output we treat it as unrecognized
                // but still try to parse it
                parse_ss3(vt_event, *ss3, cb);
            }
            VTEvent::OscStart => {
                state.capture_buffer.clear();
                state.capture_mode = CaptureMode::OscData;
            }
            VTEvent::OscCancel => {
                state.capture_buffer.clear();
                state.capture_mode = CaptureMode::None;
            }
            VTEvent::OscData(data) => {
                if state.capture_mode == CaptureMode::OscData {
                    state.capture_buffer.extend_from_slice(data);
                }
            }
            VTEvent::OscEnd { data, .. } => {
                state.capture_buffer.extend_from_slice(data);
                state.capture_mode = CaptureMode::None;
                let osc_data = std::mem::take(&mut state.capture_buffer);
                parse_osc(vt_event, &osc_data, cb);
            }
            VTEvent::DcsStart(dcs) => {
                state.store_dcs_header(dcs);
                state.capture_buffer.clear();
                state.capture_mode = CaptureMode::DcsData;
            }
            VTEvent::DcsCancel => {
                state.clear_dcs_header();
                state.capture_buffer.clear();
                state.capture_mode = CaptureMode::None;
            }
            VTEvent::DcsData(data) => {
                if state.capture_mode == CaptureMode::DcsData {
                    state.capture_buffer.extend_from_slice(data);
                }
            }
            VTEvent::DcsEnd(data) => {
                state.capture_buffer.extend_from_slice(data);
                let dcs_data = std::mem::take(&mut state.capture_buffer);
                if let Some(dcs_header) = state.dcs_header.take() {
                    parse_dcs(vt_event, &dcs_header, &dcs_data, cb);
                    state.clear_dcs_header();
                } else {
                    cb(&UnrecognizedOutputEvent(vt_event));
                }
                state.capture_buffer.clear();
                state.capture_mode = CaptureMode::None;
            }
            VTEvent::Ss2(_) => {
                // SS2 is rarely used in output; treat as unrecognized
                cb(&UnrecognizedOutputEvent(vt_event));
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

fn parse_c0<F>(_vt_event: &VTEvent, c0_byte: u8, cb: &mut F)
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    if !common::parse_c0(
        c0_byte,
        vtansi::registry::ansi_output_c0_trie_cursor,
        cb,
    ) {
        let s = str::from_utf8(std::slice::from_ref(&c0_byte)).unwrap();
        cb(&PlainText(s));
    }
}

fn parse_esc<F>(vt_event: &VTEvent, seq: vt_push_parser::event::Esc, cb: &mut F)
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    if !common::parse_esc(
        seq,
        vtansi::registry::ansi_output_esc_trie_cursor,
        cb,
    ) {
        cb(&UnrecognizedOutputEvent(vt_event));
    }
}

fn parse_esc_invalid<F>(
    vt_event: &VTEvent,
    seq: vt_push_parser::event::EscInvalid,
    cb: &mut F,
) where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    if !common::parse_esc_invalid(
        seq,
        vtansi::registry::ansi_output_esc_trie_cursor,
        cb,
    ) {
        cb(&UnrecognizedOutputEvent(vt_event));
    }
}

fn parse_ss3<F>(vt_event: &VTEvent, ss3: vt_push_parser::event::SS3, cb: &mut F)
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    // SS3 is primarily for input, but try to parse anyway
    // For output, we don't have SS3-specific handlers, so just emit unrecognized
    let mut cursor = vtansi::registry::ansi_output_esc_trie_cursor();

    // Try to advance with 'O' (SS3 introducer after ESC)
    cursor.advance(b'O');

    if common::maybe_handle_byte(&mut cursor, ss3.char, cb) {
        return;
    }

    // Try with data
    let mut cursor = vtansi::registry::ansi_output_esc_trie_cursor();
    cursor.advance(b'O');
    cursor.advance(0);

    if common::maybe_handle_data(
        &cursor,
        &AnsiEventData::new_with_data(std::slice::from_ref(&ss3.char)),
        cb,
    ) {
        return;
    }

    cb(&UnrecognizedOutputEvent(vt_event));
}

fn parse_csi<F>(
    vt_event: &VTEvent,
    csi: &vt_push_parser::event::CSI,
    cb: &mut F,
) where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    if !common::parse_csi(
        csi,
        vtansi::registry::ansi_output_csi_trie_cursor,
        cb,
    ) {
        cb(&UnrecognizedOutputEvent(vt_event));
    }
}

fn parse_osc<F>(vt_event: &VTEvent, osc_data: &[u8], cb: &mut F)
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    if !common::parse_osc(
        osc_data,
        vtansi::registry::ansi_output_osc_trie_cursor,
        cb,
    ) {
        cb(&UnrecognizedOutputEvent(vt_event));
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
        vtansi::registry::ansi_output_dcs_trie_cursor,
        cb,
    ) {
        cb(&UnrecognizedOutputEvent(vt_event));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::PlainText;
    use crate::event::sgr::{Sgr, SgrAttr, parse_sgr_params};
    use better_any::TidExt;

    fn count_events(input: &[u8]) -> usize {
        let mut count = 0;
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(input, &mut |_event: &dyn vtansi::AnsiEvent| {
            count += 1;
        });
        count
    }

    fn collect_plaintext(input: &[u8]) -> Vec<String> {
        let mut texts = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(input, &mut |event: &dyn vtansi::AnsiEvent| {
            if let Some(pt) = event.downcast_ref::<PlainText>() {
                texts.push(pt.0.to_string());
            }
        });
        texts
    }

    #[test]
    fn test_basic_text() {
        let count = count_events(b"Hello, world!");
        assert!(count > 0);
    }

    #[test]
    fn test_plaintext_ascii() {
        let texts = collect_plaintext(b"Hello, world!");
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0], "Hello, world!");
    }

    #[test]
    fn test_plaintext_utf8() {
        let texts = collect_plaintext("Hello, 世界! 🎉".as_bytes());
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0], "Hello, 世界! 🎉");
    }

    #[test]
    fn test_plaintext_encode() {
        let pt = PlainText("Hello, 世界!");
        let mut buf = Vec::new();
        vtansi::AnsiEncode::encode_ansi_into(&pt, &mut buf).unwrap();
        assert_eq!(buf, "Hello, 世界!".as_bytes());
    }

    #[test]
    fn test_plaintext_direction() {
        let pt = PlainText("test");
        assert_eq!(
            vtansi::AnsiEvent::ansi_direction(&pt),
            vtansi::AnsiControlDirection::Output
        );
        assert!(vtansi::AnsiEvent::ansi_control_kind(&pt).is_none());
    }

    #[test]
    fn test_plaintext_incomplete_utf8_across_chunks() {
        // "世" is 3 bytes: E4 B8 96
        let full = "世";
        let bytes = full.as_bytes();
        assert_eq!(bytes.len(), 3);

        let mut texts = Vec::new();
        let mut parser = TerminalOutputParser::new();

        // Feed first 2 bytes (incomplete)
        parser.feed_with(&bytes[..2], &mut |event: &dyn vtansi::AnsiEvent| {
            if let Some(pt) = event.downcast_ref::<PlainText>() {
                texts.push(pt.0.to_string());
            }
        });
        // No text should be emitted yet
        assert!(texts.is_empty(), "Should not emit incomplete UTF-8");

        // Feed last byte
        parser.feed_with(&bytes[2..], &mut |event: &dyn vtansi::AnsiEvent| {
            if let Some(pt) = event.downcast_ref::<PlainText>() {
                texts.push(pt.0.to_string());
            }
        });
        // Now we should have the complete character
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0], "世");
    }

    #[test]
    fn test_plaintext_invalid_utf8_skipped() {
        // Invalid UTF-8 byte followed by valid ASCII
        let input: &[u8] = &[0xFF, b'H', b'i'];
        let texts = collect_plaintext(input);
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0], "Hi");
    }

    #[test]
    fn test_csi_cursor_position() {
        // CSI 10;20 H - cursor position
        let count = count_events(b"\x1b[10;20H");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_csi_sgr() {
        // CSI 1;31 m - bold red
        let count = count_events(b"\x1b[1;31m");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_csi_sgr_extended_256_color() {
        // CSI 38;5;196 m - 256-color foreground (red)
        let count = count_events(b"\x1b[38;5;196m");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_csi_sgr_extended_rgb_color() {
        // CSI 38;2;255;128;0 m - RGB foreground (orange)
        let count = count_events(b"\x1b[38;2;255;128;0m");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_csi_sgr_complex_with_extended_colors() {
        // CSI 1;38;5;196;48;2;0;0;128 m - bold + 256 fg + RGB bg
        let count = count_events(b"\x1b[1;38;5;196;48;2;0;0;128m");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_csi_sgr_colon_format() {
        // CSI 38:5:196 m - 256-color using colon format (ITU T.416)
        let count = count_events(b"\x1b[38:5:196m");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_csi_sgr_underline_style() {
        // CSI 4:3 m - curly underline (kitty extension)
        let count = count_events(b"\x1b[4:3m");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_parse_sgr_params_semicolon_extended_colors() {
        use crate::event::sgr::ExtendedColor;

        // Test parsing 38;5;196 (256-color foreground)
        let attrs = parse_sgr_params(b"38;5;196").unwrap();
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0], SgrAttr::Foreground(ExtendedColor::Palette(196)));

        // Test parsing 38;2;255;128;0 (RGB foreground)
        let attrs = parse_sgr_params(b"38;2;255;128;0").unwrap();
        assert_eq!(attrs.len(), 1);
        assert_eq!(
            attrs[0],
            SgrAttr::Foreground(ExtendedColor::Rgb {
                r: 255,
                g: 128,
                b: 0
            })
        );

        // Test parsing complex sequence: 1;38;5;196;48;2;0;0;128
        let attrs = parse_sgr_params(b"1;38;5;196;48;2;0;0;128").unwrap();
        assert_eq!(attrs.len(), 3);
        assert_eq!(attrs[0], SgrAttr::Bold);
        assert_eq!(attrs[1], SgrAttr::Foreground(ExtendedColor::Palette(196)));
        assert_eq!(
            attrs[2],
            SgrAttr::Background(ExtendedColor::Rgb { r: 0, g: 0, b: 128 })
        );
    }

    #[test]
    fn test_sgr_roundtrip_extended_256() {
        use crate::event::sgr::ExtendedColor;
        use vtansi::AnsiEncode;

        // Create a 256-color SGR, encode it, then verify
        let sgr = Sgr::new(SgrAttr::Foreground(ExtendedColor::Palette(196)));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38:5:196m");

        // Parse through output parser
        let count = count_events(&buf);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sgr_roundtrip_extended_rgb() {
        use crate::event::sgr::ExtendedColor;
        use vtansi::AnsiEncode;

        // Create an RGB SGR, encode it, then verify
        let sgr = Sgr::new(SgrAttr::Foreground(ExtendedColor::Rgb {
            r: 255,
            g: 128,
            b: 64,
        }));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38:2::255:128:64m");

        // Parse through output parser
        let count = count_events(&buf);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_osc_title() {
        // OSC 0 ; title ST - set window title
        let count = count_events(b"\x1b]0;My Title\x1b\\");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_esc_save_cursor() {
        // ESC 7 - save cursor
        let count = count_events(b"\x1b7");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_mixed_sequences() {
        // Mix of text and escape sequences
        let input = b"Hello\x1b[31mRed\x1b[0mNormal";
        let count = count_events(input);
        assert!(count >= 3);
    }

    #[test]
    fn test_scroll_down_default_param() {
        use crate::event::scroll::ScrollDown;

        // CSI 1 T - scroll down with explicit param 1 (the default)
        // Note: CSI T without params won't match ScrollDown since it requires a u16 param
        let mut scroll_events: Vec<ScrollDown> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(b"\x1b[1T", &mut |event: &dyn vtansi::AnsiEvent| {
            if let Some(sd) = event.downcast_ref::<ScrollDown>() {
                scroll_events.push(*sd);
            }
        });
        assert_eq!(scroll_events.len(), 1);
        assert_eq!(scroll_events[0].0, 1);
    }

    #[test]
    fn test_scroll_down_with_param() {
        use crate::event::scroll::ScrollDown;

        // CSI 5 T - scroll down 5 lines
        let mut events: Vec<Box<dyn std::any::Any>> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(b"\x1b[5T", &mut |event: &dyn vtansi::AnsiEvent| {
            if let Some(sd) = event.downcast_ref::<ScrollDown>() {
                events.push(Box::new(*sd));
            }
        });
        assert_eq!(events.len(), 1);
        let sd = events[0].downcast_ref::<ScrollDown>().unwrap();
        assert_eq!(sd.0, 5);
    }

    #[test]
    fn test_track_mouse() {
        use crate::event::mouse::TrackMouse;

        // CSI 1;10;5;1;20 T - track mouse with 5 params
        let mut events: Vec<Box<dyn std::any::Any>> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(
            b"\x1b[1;10;5;1;20T",
            &mut |event: &dyn vtansi::AnsiEvent| {
                if let Some(tm) = event.downcast_ref::<TrackMouse>() {
                    events.push(Box::new(*tm));
                }
            },
        );
        assert_eq!(events.len(), 1, "TrackMouse should be parsed");
    }

    #[test]
    fn test_scroll_down_and_track_mouse_disambiguation() {
        use crate::event::mouse::TrackMouse;
        use crate::event::scroll::ScrollDown;

        // Test that ScrollDown (1 param) and TrackMouse (5 params) can coexist
        // with the same final byte 'T'

        let mut scroll_count = 0;
        let mut track_count = 0;
        let mut parser = TerminalOutputParser::new();

        // Parse ScrollDown
        parser.feed_with(b"\x1b[3T", &mut |event: &dyn vtansi::AnsiEvent| {
            if event.downcast_ref::<ScrollDown>().is_some() {
                scroll_count += 1;
            }
            if event.downcast_ref::<TrackMouse>().is_some() {
                track_count += 1;
            }
        });

        assert_eq!(scroll_count, 1, "ScrollDown should be parsed");
        assert_eq!(
            track_count, 0,
            "TrackMouse should not be parsed for 1 param"
        );

        // Parse TrackMouse
        scroll_count = 0;
        track_count = 0;
        parser.feed_with(
            b"\x1b[0;1;1;1;24T",
            &mut |event: &dyn vtansi::AnsiEvent| {
                if event.downcast_ref::<ScrollDown>().is_some() {
                    scroll_count += 1;
                }
                if event.downcast_ref::<TrackMouse>().is_some() {
                    track_count += 1;
                }
            },
        );

        assert_eq!(track_count, 1, "TrackMouse should be parsed for 5 params");
        assert_eq!(
            scroll_count, 0,
            "ScrollDown should not be parsed for 5 params"
        );
    }

    #[test]
    fn test_scroll_down_encode_roundtrip() {
        use crate::event::scroll::ScrollDown;
        use vtansi::AnsiEncode;

        let sd = ScrollDown(5);
        let encoded = sd.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b[5T");

        // Parse it back
        let mut events: Vec<ScrollDown> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(&encoded, &mut |event: &dyn vtansi::AnsiEvent| {
            if let Some(sd) = event.downcast_ref::<ScrollDown>() {
                events.push(*sd);
            }
        });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, 5);
    }

    #[test]
    fn test_track_mouse_encode_roundtrip() {
        use crate::event::mouse::TrackMouse;
        use vtansi::AnsiEncode;

        let tm = TrackMouse::new(1, 10, 5, 1, 24);
        let encoded = tm.encode_ansi().unwrap();
        assert_eq!(encoded, b"\x1b[1;10;5;1;24T");

        // Parse it back
        let mut events: Vec<TrackMouse> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(&encoded, &mut |event: &dyn vtansi::AnsiEvent| {
            if let Some(tm) = event.downcast_ref::<TrackMouse>() {
                events.push(*tm);
            }
        });
        assert_eq!(events.len(), 1);
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn test_osc_133_shell_integration() {
        use crate::event::shell::{
            CommandEnd, CommandStart, PromptEnd, PromptStart,
        };

        // Test OSC 133;A (PromptStart)
        let mut prompt_starts: Vec<PromptStart> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(
            b"\x1b]133;A\x1b\\",
            &mut |event: &dyn vtansi::AnsiEvent| {
                if let Some(ps) = event.downcast_ref::<PromptStart>() {
                    prompt_starts.push(*ps);
                }
            },
        );
        assert_eq!(
            prompt_starts.len(),
            1,
            "OSC 133;A (PromptStart) should be recognized"
        );

        // Test OSC 133;B (PromptEnd)
        let mut prompt_ends: Vec<PromptEnd> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(
            b"\x1b]133;B\x1b\\",
            &mut |event: &dyn vtansi::AnsiEvent| {
                if let Some(pe) = event.downcast_ref::<PromptEnd>() {
                    prompt_ends.push(*pe);
                }
            },
        );
        assert_eq!(
            prompt_ends.len(),
            1,
            "OSC 133;B (PromptEnd) should be recognized"
        );

        // Test OSC 133;C (CommandStart)
        let mut command_starts: Vec<CommandStart> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(
            b"\x1b]133;C\x1b\\",
            &mut |event: &dyn vtansi::AnsiEvent| {
                if let Some(cs) = event.downcast_ref::<CommandStart>() {
                    command_starts.push(*cs);
                }
            },
        );
        assert_eq!(
            command_starts.len(),
            1,
            "OSC 133;C (CommandStart) should be recognized"
        );

        // Test OSC 133;D (CommandEnd without exit code)
        let mut command_ends: Vec<CommandEnd> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(
            b"\x1b]133;D\x1b\\",
            &mut |event: &dyn vtansi::AnsiEvent| {
                if let Some(ce) = event.downcast_ref::<CommandEnd>() {
                    command_ends.push(*ce);
                }
            },
        );
        assert_eq!(
            command_ends.len(),
            1,
            "OSC 133;D (CommandEnd) should be recognized"
        );
        assert_eq!(
            command_ends[0].exit_code, None,
            "CommandEnd without exit code should have None"
        );

        // Test OSC 133;D;0 (CommandEnd with exit code 0)
        let mut command_ends: Vec<CommandEnd> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(
            b"\x1b]133;D;0\x1b\\",
            &mut |event: &dyn vtansi::AnsiEvent| {
                if let Some(ce) = event.downcast_ref::<CommandEnd>() {
                    command_ends.push(*ce);
                }
            },
        );
        assert_eq!(
            command_ends.len(),
            1,
            "OSC 133;D;0 (CommandEnd with exit code) should be recognized"
        );
        assert_eq!(
            command_ends[0].exit_code,
            Some(0),
            "CommandEnd should have exit_code = Some(0)"
        );

        // Test OSC 133;D;127 (CommandEnd with exit code 127)
        let mut command_ends: Vec<CommandEnd> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(
            b"\x1b]133;D;127\x1b\\",
            &mut |event: &dyn vtansi::AnsiEvent| {
                if let Some(ce) = event.downcast_ref::<CommandEnd>() {
                    command_ends.push(*ce);
                }
            },
        );
        assert_eq!(
            command_ends.len(),
            1,
            "OSC 133;D;127 (CommandEnd with exit code) should be recognized"
        );
        assert_eq!(
            command_ends[0].exit_code,
            Some(127),
            "CommandEnd should have exit_code = Some(127)"
        );
    }

    #[test]
    fn test_request_terminal_name_and_version() {
        use crate::event::UnrecognizedOutputEvent;
        use crate::event::terminal::RequestTerminalNameAndVersion;

        // CSI > 0 q
        let input = b"\x1b[>0q";
        let mut events = Vec::new();
        let mut unrecognized = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(input, &mut |event: &dyn vtansi::AnsiEvent| {
            if event
                .downcast_ref::<RequestTerminalNameAndVersion>()
                .is_some()
            {
                events.push("RequestTerminalNameAndVersion");
            }
            if let Some(u) = event.downcast_ref::<UnrecognizedOutputEvent>() {
                println!("Unrecognized event: {u:?}");
                unrecognized.push(format!("{u:?}"));
            }
        });
        println!("Unrecognized events: {unrecognized:?}");
        assert_eq!(
            events.len(),
            1,
            "CSI > 0 q should be parsed as RequestTerminalNameAndVersion"
        );
    }

    #[test]
    fn test_request_primary_device_attributes_without_param() {
        use crate::event::terminal::RequestPrimaryDeviceAttributes;

        // CSI c - DA1 without parameter
        let input = b"\x1b[c";
        let mut events: Vec<RequestPrimaryDeviceAttributes> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(input, &mut |event: &dyn vtansi::AnsiEvent| {
            if let Some(da) =
                event.downcast_ref::<RequestPrimaryDeviceAttributes>()
            {
                events.push(*da);
            }
        });
        assert_eq!(
            events.len(),
            1,
            "CSI c should be parsed as RequestPrimaryDeviceAttributes"
        );
    }

    #[test]
    fn test_request_primary_device_attributes_with_param() {
        use crate::event::terminal::RequestPrimaryDeviceAttributes;

        // CSI 0 c - DA1 with explicit parameter 0
        let input = b"\x1b[0c";
        let mut events: Vec<RequestPrimaryDeviceAttributes> = Vec::new();
        let mut parser = TerminalOutputParser::new();
        parser.feed_with(input, &mut |event: &dyn vtansi::AnsiEvent| {
            if let Some(da) =
                event.downcast_ref::<RequestPrimaryDeviceAttributes>()
            {
                events.push(*da);
            }
        });
        assert_eq!(
            events.len(),
            1,
            "CSI 0 c should be parsed as RequestPrimaryDeviceAttributes"
        );
    }
}
