use std::fmt;
use std::hash::Hash;
use vtansi::bitflags;

bitflags! {
    /// Keyboard enhancement flags for the kitty keyboard protocol.
    ///
    /// Represents special flags that tell compatible terminals to add extra
    /// information to keyboard events.
    ///
    /// See <https://sw.kovidgoyal.net/kitty/keyboard-protocol/#progressive-enhancement>
    /// for more information.
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
    #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, Default)]
    pub struct KeyboardEnhancementFlags: u8 {
        /// Represent Escape and modified keys using CSI-u sequences.
        ///
        /// This allows them to be unambiguously read.
        const DISAMBIGUATE_ESCAPE_CODES = 0b0000_0001;
        /// Add extra events with repeat or release event types.
        ///
        /// Add extra events when keys are autorepeated or released.
        const REPORT_EVENT_TYPES = 0b0000_0010;
        /// Send alternate keycodes in addition to the base keycode.
        ///
        /// Send alternate keycodes as described in the kitty keyboard
        /// protocol. The alternate keycode overrides the base keycode in
        /// resulting key events.
        const REPORT_ALTERNATE_KEYS = 0b0000_0100;
        /// Represent all keyboard events as CSI-u sequences.
        ///
        /// This is required to get repeat/release events for plain-text keys.
        const REPORT_ALL_KEYS_AS_ESCAPE_CODES = 0b0000_1000;
        /// Send the Unicode codepoint as well as the keycode.
        const REPORT_ASSOCIATED_TEXT = 0b0001_0000;
    }
}

/// Query Keyboard Enhancement Flags.
///
/// *Sequence*: `CSI ? u`
///
/// Query the current keyboard enhancement flags.
///
/// The terminal responds with `CSI ? Ps u` ([`KeyboardEnhancementFlagsResponse`]).
///
/// See <https://sw.kovidgoyal.net/kitty/keyboard-protocol/> for more
/// information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, vtansi::derive::AnsiOutput)]
#[vtansi(csi, private = '?', finalbyte = 'u')]
pub struct KeyboardEnhancementFlagsQuery;

/// Keyboard Enhancement Flags Response.
///
/// *Sequence*: `CSI ? Ps u`
///
/// Response to [`KeyboardEnhancementFlagsQuery`] containing the current
/// keyboard enhancement flags.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiInput,
)]
#[vtansi(csi, private = '?', finalbyte = 'u')]
pub struct KeyboardEnhancementFlagsResponse(
    pub Option<KeyboardEnhancementFlags>,
);

/// Push Keyboard Enhancement Flags.
///
/// *Sequence*: `CSI > Ps u`
///
/// Enable the kitty keyboard protocol, which adds extra information to
/// keyboard events and removes ambiguity for modifier keys.
///
/// It should be paired with [`PopKeyboardEnhancementFlags`] to restore the
/// previous state.
///
/// See <https://sw.kovidgoyal.net/kitty/keyboard-protocol/> for more
/// information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, vtansi::derive::AnsiOutput)]
#[vtansi(csi, private = '>', finalbyte = 'u')]
pub struct PushKeyboardEnhancementFlags(pub KeyboardEnhancementFlags);

/// Set Keyboard Enhancement Flags.
///
/// *Sequence*: `CSI = Ps u`
///
/// Set the keyboard enhancement flags directly, without using the stack.
///
/// Unlike [`PushKeyboardEnhancementFlags`], this does not push the flags onto
/// a stack; it sets them directly.
///
/// See <https://sw.kovidgoyal.net/kitty/keyboard-protocol/#progressive-enhancement>
/// for more information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, vtansi::derive::AnsiOutput)]
#[vtansi(csi, private = '=', finalbyte = 'u')]
pub struct SetKeyboardEnhancementFlags(pub KeyboardEnhancementFlags);

/// Pop Keyboard Enhancement Flags.
///
/// *Sequence*: `CSI < u`
///
/// Disable extra kinds of keyboard events.
///
/// Specifically, it pops one level of keyboard enhancement flags.
///
/// See [`PushKeyboardEnhancementFlags`] and
/// <https://sw.kovidgoyal.net/kitty/keyboard-protocol/> for more information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, vtansi::derive::AnsiOutput)]
#[vtansi(csi, private = '<', finalbyte = 'u')]
pub struct PopKeyboardEnhancementFlags;

/// Set Application Keypad Mode (`DECKPAM`).
///
/// *Sequence*: `ESC =`
///
/// Enable application keypad mode.
///
/// See <https://terminalguide.namepad.de/seq/esc_a_eq/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, vtansi::derive::AnsiOutput)]
#[vtansi(esc, finalbyte = '=')]
pub struct SetApplicationKeypadMode;

/// Reset Application Keypad Mode (`DECKPNM`).
///
/// *Sequence*: `ESC >`
///
/// Disable application keypad mode.
///
/// See <https://terminalguide.namepad.de/seq/esc_a_gt/> for
/// terminal support specifics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, vtansi::derive::AnsiOutput)]
#[vtansi(esc, finalbyte = '>')]
pub struct ResetApplicationKeypadMode;

impl fmt::Display for KeyboardEnhancementFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("None");
        }

        let mut first = true;
        for (flag, name) in [
            (Self::DISAMBIGUATE_ESCAPE_CODES, "DISAMBIGUATE_ESCAPE_CODES"),
            (Self::REPORT_EVENT_TYPES, "REPORT_EVENT_TYPES"),
            (Self::REPORT_ALTERNATE_KEYS, "REPORT_ALTERNATE_KEYS"),
            (
                Self::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
                "REPORT_ALL_KEYS_AS_ESCAPE_CODES",
            ),
            (Self::REPORT_ASSOCIATED_TEXT, "REPORT_ASSOCIATED_TEXT"),
        ] {
            if self.contains(flag) {
                if !first {
                    write!(f, " | ")?;
                }
                f.write_str(name)?;
                first = false;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::AnsiEncode;

    #[test]
    fn test_keyboard_enhancement_flags_query() {
        let query = KeyboardEnhancementFlagsQuery;
        let mut buf = Vec::new();
        query.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[?u");
    }

    #[test]
    fn test_push_keyboard_enhancement_flags_disambiguate() {
        let push = PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES,
        );
        let mut buf = Vec::new();
        push.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>1u");
    }

    #[test]
    fn test_push_keyboard_enhancement_flags_all() {
        let push = PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ASSOCIATED_TEXT,
        );
        let mut buf = Vec::new();
        push.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>31u");
    }

    #[test]
    fn test_push_keyboard_enhancement_flags_empty() {
        let push =
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::empty());
        let mut buf = Vec::new();
        push.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>0u");
    }

    #[test]
    fn test_pop_keyboard_enhancement_flags() {
        let pop = PopKeyboardEnhancementFlags;
        let mut buf = Vec::new();
        pop.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[<u");
    }

    #[test]
    fn test_keyboard_enhancement_flags_display() {
        let flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_EVENT_TYPES;
        assert_eq!(
            format!("{flags}"),
            "DISAMBIGUATE_ESCAPE_CODES | REPORT_EVENT_TYPES"
        );

        let empty = KeyboardEnhancementFlags::empty();
        assert_eq!(format!("{empty}"), "None");
    }
}
