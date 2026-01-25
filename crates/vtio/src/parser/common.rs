//! Common parsing utilities shared between input and output parsers.

use crate::event::PlainText;
use vt_push_parser::event::DCSOwned;
use vtansi::registry::{AnsiControlFunctionTrieCursor, AnsiEventData, Answer};

/// Maximum number of params we support for CSI sequences.
/// This matches the `SmallVec` capacity used in vt-push-parser.
pub const MAX_CSI_PARAMS: usize = 16;

/// Try to handle a single byte by advancing the cursor and invoking the handler.
///
/// Returns `true` if a handler was found and invoked successfully.
#[inline]
pub fn maybe_handle_byte<F>(
    cursor: &mut AnsiControlFunctionTrieCursor,
    byte: u8,
    cb: &mut F,
) -> bool
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    match cursor.advance(byte) {
        Answer::Match(handler) | Answer::PrefixAndMatch(handler) => {
            handler(&AnsiEventData::default(), &mut |event| {
                cb(event);
            })
            .is_ok()
        }
        Answer::DeadEnd | Answer::Prefix => false,
    }
}

/// Try to handle data by dereferencing the cursor and invoking the handler.
///
/// Returns `true` if a handler was found and invoked successfully.
#[inline]
pub fn maybe_handle_data<F>(
    cursor: &AnsiControlFunctionTrieCursor,
    data: &AnsiEventData,
    cb: &mut F,
) -> bool
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    match cursor.deref() {
        Answer::Match(handler) | Answer::PrefixAndMatch(handler) => {
            handler(data, &mut |event| {
                cb(event);
            })
            .is_ok()
        }
        Answer::DeadEnd | Answer::Prefix => false,
    }
}

/// Parse a C0 control byte using the provided cursor factory.
///
/// Returns `true` if the event was handled, `false` if unrecognized.
pub fn parse_c0<F>(
    c0_byte: u8,
    cursor_factory: impl FnOnce() -> AnsiControlFunctionTrieCursor,
    cb: &mut F,
) -> bool
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    let mut cursor = cursor_factory();

    // Advance with the C0 byte
    if !cursor.advance(c0_byte).is_prefix() {
        return false;
    }

    // Advance with \0 placeholder (byte type keys end with \0)
    cursor.advance(0);

    match cursor.deref() {
        Answer::Match(handler) | Answer::PrefixAndMatch(handler) => {
            handler(&AnsiEventData::default(), cb).is_ok()
        }
        Answer::DeadEnd | Answer::Prefix => false,
    }
}

/// Parse an ESC sequence using the provided cursor factory.
///
/// Returns `true` if the event was handled, `false` if unrecognized.
pub fn parse_esc<F>(
    seq: vt_push_parser::event::Esc,
    cursor_factory: impl Fn() -> AnsiControlFunctionTrieCursor,
    cb: &mut F,
) -> bool
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    let mut cursor = cursor_factory();

    if let Some(private) = seq.private {
        let answer = cursor.advance(private);
        if answer.is_dead_end() {
            return false;
        }
    }

    // Advance through intermediate bytes for key matching
    let intermediates = seq.intermediates.as_ref();
    if !intermediates.is_empty()
        && cursor.advance_slice(intermediates) == Answer::DeadEnd
    {
        return false;
    }

    match cursor.deref() {
        Answer::Match(handler) | Answer::PrefixAndMatch(handler) => {
            let finalbyte_slice = std::slice::from_ref(&seq.final_byte);
            let data = AnsiEventData::new_with_finalbyte(finalbyte_slice);
            if handler(&data, cb).is_ok() {
                return true;
            }
        }
        Answer::DeadEnd | Answer::Prefix => (),
    }

    // Build suffix: \0 placeholder + final byte (for sequences without intermediates in key)
    let mut suffix = [0u8; 2];
    suffix[1] = seq.final_byte;

    match cursor.advance_slice(&suffix) {
        Answer::Match(handler) | Answer::PrefixAndMatch(handler) => {
            let data = AnsiEventData::new();
            if handler(&data, cb).is_ok() {
                return true;
            }
        }
        Answer::DeadEnd | Answer::Prefix => (),
    }

    false
}

/// Parse an invalid ESC sequence using the provided cursor factory.
///
/// Returns `true` if the event was handled, `false` if unrecognized.
pub fn parse_esc_invalid<F>(
    seq: vt_push_parser::event::EscInvalid,
    cursor_factory: impl FnOnce() -> AnsiControlFunctionTrieCursor,
    cb: &mut F,
) -> bool
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    let mut cursor = cursor_factory();

    // Advance with \0 placeholder to find fallback handlers like AltKeySeq
    cursor.advance(0);

    match cursor.deref() {
        Answer::Match(handler) | Answer::PrefixAndMatch(handler) => {
            let mut data = [0u8; 4];
            let data_slice = match seq {
                vt_push_parser::event::EscInvalid::Four(a, b, c, d) => {
                    data[0] = a;
                    data[1] = b;
                    data[2] = c;
                    data[3] = d;
                    &data[..]
                }
                vt_push_parser::event::EscInvalid::Three(a, b, c) => {
                    data[0] = a;
                    data[1] = b;
                    data[2] = c;
                    &data[..3]
                }
                vt_push_parser::event::EscInvalid::Two(a, b) => {
                    data[0] = a;
                    data[1] = b;
                    &data[..2]
                }
                vt_push_parser::event::EscInvalid::One(a) => {
                    data[0] = a;
                    &data[..1]
                }
            };

            let params: [&[u8]; 1] = [data_slice];
            let event_data = AnsiEventData::new_with_params(&params);
            handler(&event_data, cb).is_ok()
        }
        Answer::DeadEnd | Answer::Prefix => false,
    }
}

/// Parse a CSI sequence using the provided cursor factory.
///
/// The `intercept_cb` allows the caller to intercept events before they are
/// passed to the main callback (used for bracketed paste in input parser).
///
/// Returns `true` if the event was handled, `false` if unrecognized.
#[allow(clippy::too_many_lines)]
pub fn parse_csi<F>(
    seq: &vt_push_parser::event::CSI,
    cursor_factory: impl FnOnce() -> AnsiControlFunctionTrieCursor,
    cb: &mut F,
) -> bool
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
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

    // Create cursor once and copy for fallback attempts
    let base_cursor = cursor_factory();

    // Try normal param marker first (0 = no params, 1 = has params)
    let normal_marker: u8 = u8::from(!all_params.is_empty());
    if parse_csi_with_marker(
        seq,
        &base_cursor,
        normal_marker,
        all_params,
        &data,
        cb,
    ) {
        return true;
    }

    // If normal lookup failed and we have 2+ params, try disambiguated marker (2 + param_count)
    // This handles sequences marked with #[vtansi(disambiguate)] that use exact param count
    if param_count >= 2 {
        #[allow(clippy::cast_possible_truncation)]
        let disambiguated_marker: u8 = 2u8.saturating_add(param_count as u8);
        if parse_csi_with_marker(
            seq,
            &base_cursor,
            disambiguated_marker,
            all_params,
            &data,
            cb,
        ) {
            return true;
        }
    }

    false
}

/// Internal helper to parse CSI with a specific param marker byte.
fn parse_csi_with_marker<F>(
    seq: &vt_push_parser::event::CSI,
    cursor: &AnsiControlFunctionTrieCursor,
    param_marker: u8,
    all_params: &[&[u8]],
    data: &AnsiEventData,
    cb: &mut F,
) -> bool
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    let mut cursor = *cursor;
    let param_count = all_params.len();

    // Advance with private marker if present
    if let Some(private) = seq.private
        && !cursor.advance(private).is_prefix()
    {
        return false;
    }

    // Advance with param marker byte
    if matches!(cursor.advance(param_marker), Answer::DeadEnd) {
        return false;
    }

    // Advance through intermediate bytes for key matching
    let intermediates = seq.intermediates.as_ref();
    if !intermediates.is_empty()
        && cursor.advance_slice(intermediates) == Answer::DeadEnd
    {
        return false;
    }

    // Advance with final byte
    let mut finalbyte_handler: Option<vtansi::registry::Handler> = None;

    match cursor.advance(seq.final_byte) {
        Answer::DeadEnd => {
            return false;
        }
        Answer::Match(handler) => {
            return handler(&data.with_params(all_params), cb).is_ok();
        }
        Answer::Prefix => (),
        Answer::PrefixAndMatch(handler) => {
            // Try the handler immediately - it might work with our params
            if handler(&data.with_params(all_params), cb).is_ok() {
                return true;
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
                    cb,
                )
                .is_ok()
                {
                    return true;
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
            cb,
        )
        .is_ok()
    {
        return true;
    }

    if let Some(handler) = finalbyte_handler
        && handler(&data.with_params(all_params), cb).is_ok()
    {
        return true;
    }

    false
}

/// Parse an OSC sequence using the provided cursor factory.
///
/// Returns `true` if the event was handled, `false` if unrecognized.
pub fn parse_osc<F>(
    osc_data: &[u8],
    cursor_factory: impl FnOnce() -> AnsiControlFunctionTrieCursor,
    cb: &mut F,
) -> bool
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    // OSC format: [number;][static_data]dynamic_data
    // The number and static data are optional and matched via trie lookup.
    // Static data from the trie key (like `data = "A"`) is matched during trie walk.
    //
    // Strategy:
    // 1. Use longest match to find how much of osc_data is static (number + static data)
    // 2. Only Match or PrefixAndMatch are valid - DeadEnd or Prefix means unrecognized
    // 3. Pass the remaining data directly to the handler via AnsiEventData::new_with_data

    let mut cursor = cursor_factory();

    let (answer, consumed) = cursor.advance_longest_match(osc_data);

    match answer {
        Answer::Match(handler) | Answer::PrefixAndMatch(handler) => {
            let remaining = &osc_data[consumed..];
            let event_data = AnsiEventData::new_with_data(remaining);
            handler(&event_data, cb).is_ok()
        }
        Answer::DeadEnd | Answer::Prefix => false,
    }
}

/// Parse a DCS sequence header using the provided cursor factory.
///
/// Returns `true` if the event was handled, `false` if unrecognized.
pub fn parse_dcs<F>(
    seq: &vt_push_parser::event::DCS,
    dcs_data: &[u8],
    cursor_factory: impl FnOnce() -> AnsiControlFunctionTrieCursor,
    cb: &mut F,
) -> bool
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
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

    parse_dcs_internal(
        seq.private,
        all_params,
        seq.intermediates.as_ref(),
        seq.final_byte,
        dcs_data,
        cursor_factory,
        cb,
    )
}

/// Parse a DCS sequence with owned params.
///
/// This is useful when the DCS header has been stored and the original
/// `vt_push_parser::event::DCS` is no longer available.
///
/// Returns `true` if the event was handled, `false` if unrecognized.
pub fn parse_dcs_owned<F>(
    dcs_header: &DCSOwned,
    dcs_data: &[u8],
    cursor_factory: impl FnOnce() -> AnsiControlFunctionTrieCursor,
    cb: &mut F,
) -> bool
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    // Collect params into a stack-allocated array to avoid heap allocation.
    let mut params_storage: [&[u8]; MAX_CSI_PARAMS] = [&[]; MAX_CSI_PARAMS];
    let mut param_count = 0;
    for param in &dcs_header.params {
        if param_count < MAX_CSI_PARAMS {
            params_storage[param_count] = param;
            param_count += 1;
        }
    }
    let all_params = &params_storage[..param_count];

    parse_dcs_internal(
        dcs_header.private,
        all_params,
        dcs_header.intermediates.as_ref(),
        dcs_header.final_byte,
        dcs_data,
        cursor_factory,
        cb,
    )
}

/// Internal DCS parsing implementation.
fn parse_dcs_internal<F>(
    private: Option<u8>,
    all_params: &[&[u8]],
    intermediates: &[u8],
    final_byte: u8,
    dcs_data: &[u8],
    cursor_factory: impl FnOnce() -> AnsiControlFunctionTrieCursor,
    cb: &mut F,
) -> bool
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    let mut cursor = cursor_factory();

    // Advance with private marker if present
    if let Some(priv_byte) = private
        && !cursor.advance(priv_byte).is_prefix()
    {
        return false;
    }

    // Advance through intermediate bytes for key matching
    if !intermediates.is_empty()
        && cursor.advance_slice(intermediates) == Answer::DeadEnd
    {
        return false;
    }

    // Advance with final byte
    let finalbyte_slice = std::slice::from_ref(&final_byte);

    match cursor.advance(final_byte) {
        Answer::DeadEnd => return false,
        Answer::Match(handler) => {
            let data = AnsiEventData::new_with_finalbyte(finalbyte_slice)
                .with_params(all_params)
                .with_data(dcs_data);
            return handler(&data, cb).is_ok();
        }
        Answer::Prefix | Answer::PrefixAndMatch(_) => {
            // Try advancing with the DCS data to find a more specific match
            let (answer, consumed) = cursor.advance_longest_match(dcs_data);
            match answer {
                Answer::Match(handler) | Answer::PrefixAndMatch(handler) => {
                    let remaining = &dcs_data[consumed..];
                    let data =
                        AnsiEventData::new_with_finalbyte(finalbyte_slice)
                            .with_params(all_params)
                            .with_data(remaining);
                    if handler(&data, cb).is_ok() {
                        return true;
                    }
                }
                Answer::DeadEnd | Answer::Prefix => (),
            }
        }
    }

    false
}

/// Convert raw bytes to a `PlainText` event, handling UTF-8 sequences.
///
/// Emits `PlainText` events for valid UTF-8 text. Invalid bytes are skipped.
/// Returns the number of incomplete UTF-8 bytes that should be buffered
/// for the next call.
#[inline]
pub fn bytes_to_plaintext<F>(
    bytes: &[u8],
    utf8_buffer: &mut [u8],
    cb: &mut F,
) -> usize
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    match std::str::from_utf8(bytes) {
        Ok(s) => {
            // Fast path: entire buffer is valid UTF-8
            if !s.is_empty() {
                cb(&PlainText(s));
            }
            0
        }
        Err(e) => {
            // Emit the valid portion
            let valid_up_to = e.valid_up_to();
            if valid_up_to > 0 {
                // SAFETY: from_utf8 told us bytes[..valid_up_to] is valid UTF-8
                let valid_str = unsafe {
                    std::str::from_utf8_unchecked(&bytes[..valid_up_to])
                };
                cb(&PlainText(valid_str));
            }

            // Check if error is due to incomplete sequence at end
            if let Some(error_len) = e.error_len() {
                // Invalid byte(s) - skip them and recurse on remaining
                let skip = valid_up_to + error_len;
                if skip < bytes.len() {
                    return bytes_to_plaintext(&bytes[skip..], utf8_buffer, cb);
                }
                0
            } else {
                // Incomplete sequence at end - buffer it for next call
                let remaining = &bytes[valid_up_to..];
                let len = remaining.len();
                utf8_buffer[..len].copy_from_slice(remaining);
                len
            }
        }
    }
}

/// Convert raw bytes to keyboard/text events, handling UTF-8 sequences.
///
/// Returns the number of incomplete UTF-8 bytes that should be buffered
/// for the next call.
#[inline]
pub fn bytes_to_events<F>(
    bytes: &[u8],
    utf8_buffer: &mut [u8],
    cb: &mut F,
) -> usize
where
    F: FnMut(&dyn vtansi::AnsiEvent),
{
    crate::event::keyboard::bytes_to_events(bytes, utf8_buffer, cb)
}
