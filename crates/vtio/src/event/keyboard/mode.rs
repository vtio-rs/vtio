//! Keyboard mode control sequences.
//!
//! This module contains types for controlling keyboard behavior in terminal
//! emulators, including the Kitty keyboard protocol and xterm modifier key
//! options.

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

// ============================================================================
// XTerm Key Modifier Options (XTMODKEYS)
// ============================================================================

/// `XTerm` modifier key resource identifiers.
///
/// These identify which modifier key resource to configure with
/// [`SetXTermKeyModifierOptions`].
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html> for the
/// `xterm` specification.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u8)]
pub enum XTermModifierKeyResource {
    /// `modifyKeyboard` resource - controls keyboard modifier encoding.
    ///
    /// When set to a non-zero value, xterm will send escape sequences for
    /// modified keys that would otherwise send a control character.
    #[default]
    ModifyKeyboard = 0,
    /// `modifyCursorKeys` resource - controls cursor key modifier encoding.
    ///
    /// Affects how modifiers are reported with cursor keys (arrows, Home, End).
    ModifyCursorKeys = 1,
    /// `modifyFunctionKeys` resource - controls function key modifier encoding.
    ///
    /// Affects how modifiers are reported with function keys (F1-F12, etc.).
    ModifyFunctionKeys = 2,
    /// `modifyOtherKeys` resource - controls other key modifier encoding.
    ///
    /// When set, xterm will send escape sequences for modified keys that
    /// would otherwise be indistinguishable from unmodified keys.
    ///
    /// - Value 0: Disabled
    /// - Value 1: Report modifiers for keys without well-known encoding
    /// - Value 2: Report modifiers for all keys (except for special cases)
    ModifyOtherKeys = 4,
}

/// Set/Reset `XTerm` Key Modifier Options (`XTMODKEYS`).
///
/// *Sequence*: `CSI > Pp ; Pv m`
///
/// Set or reset `xterm` key modifier options. The `resource` parameter
/// specifies which modifier resource to configure, and `value` specifies
/// the new value.
///
/// If `value` is `None`, the resource is reset to its default value.
///
/// This is an `xterm`-specific sequence that may not be supported by all
/// terminal emulators.
///
/// # Resources
///
/// - `ModifyKeyboard` (0): Controls keyboard modifier encoding
/// - `ModifyCursorKeys` (1): Controls cursor key modifier encoding
/// - `ModifyFunctionKeys` (2): Controls function key modifier encoding
/// - `ModifyOtherKeys` (4): Controls other key modifier encoding
///
/// # Example
///
/// ```ignore
/// use vtio::event::keyboard::{SetXTermKeyModifierOptions, XTermModifierKeyResource};
/// use vtansi::AnsiEncode;
///
/// // Enable modifyOtherKeys mode 2
/// let cmd = SetXTermKeyModifierOptions {
///     resource: XTermModifierKeyResource::ModifyOtherKeys,
///     value: Some(2),
/// };
/// assert_eq!(cmd.to_ansi_string(), "\x1b[>4;2m");
///
/// // Reset modifyOtherKeys to default
/// let cmd = SetXTermKeyModifierOptions {
///     resource: XTermModifierKeyResource::ModifyOtherKeys,
///     value: None,
/// };
/// assert_eq!(cmd.to_ansi_string(), "\x1b[>4m");
/// ```
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html> for the
/// `xterm` specification.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '>', finalbyte = 'm')]
pub struct SetXTermKeyModifierOptions {
    /// The modifier key resource to configure.
    pub resource: XTermModifierKeyResource,
    /// The value to set. If `None`, the resource is reset to its default.
    pub value: Option<u16>,
}

// NOTE: QueryXTermKeyModifierOptions (XTQMODKEYS, `CSI ? Pp m`) and its response
// XTermKeyModifierOptionsResponse are intentionally not implemented because the
// sequence conflicts with SetLinuxMousePointerStyle (`CSI ? Ps ; Ps m`).
//
// The Linux mouse pointer style sequence was implemented first and is kept for
// compatibility. If XTQMODKEYS support is needed in the future, it would require
// a different parsing strategy to disambiguate between the two sequences.

/// Disable `XTerm` Key Modifier Options.
///
/// *Sequence*: `CSI > Ps n`
///
/// Disable specific `xterm` key modifier options. The parameter specifies which
/// modifier resource to disable.
///
/// This is an `xterm`-specific sequence that may not be supported by all
/// terminal emulators.
///
/// # Example
///
/// ```ignore
/// use vtio::event::keyboard::{DisableXTermKeyModifierOptions, XTermModifierKeyResource};
/// use vtansi::AnsiEncode;
///
/// // Disable modifyOtherKeys
/// let cmd = DisableXTermKeyModifierOptions(XTermModifierKeyResource::ModifyOtherKeys);
/// assert_eq!(cmd.to_ansi_string(), "\x1b[>4n");
/// ```
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html> for the
/// `xterm` specification.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '>', finalbyte = 'n')]
pub struct DisableXTermKeyModifierOptions(pub XTermModifierKeyResource);

// ============================================================================
// XTerm Key Format Options (XTFMTKEYS)
// ============================================================================

/// `XTerm` key format option identifiers.
///
/// These identify which key format option to configure with
/// [`SetXTermKeyFormatOptions`] or query with [`QueryXTermKeyFormatOptions`].
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html> for the
/// `xterm` specification.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u8)]
pub enum XTermKeyFormatOption {
    /// Format option for backarrow key behavior.
    ///
    /// Controls what the backarrow (backspace) key sends.
    #[default]
    Backarrow = 0,
    /// Format option for application escape sequences.
    ///
    /// Controls the format of application escape sequences.
    ApplicationEscape = 1,
    /// Format option for PC-style function keys.
    ///
    /// Controls the format of function key sequences.
    PcStyleFunctionKeys = 2,
    /// Format option for Sun function keys.
    ///
    /// Controls Sun-style function key encoding.
    SunFunctionKeys = 3,
    /// Format option for HP function keys.
    ///
    /// Controls HP-style function key encoding.
    HpFunctionKeys = 4,
    /// Format option for SCO function keys.
    ///
    /// Controls SCO-style function key encoding.
    ScoFunctionKeys = 5,
}

/// Set/Reset `XTerm` Key Format Options (`XTFMTKEYS`).
///
/// *Sequence*: `CSI > Pp ; Pv f`
///
/// Set or reset `xterm` key format options. The `option` parameter specifies
/// which format option to configure, and `value` specifies the new value.
///
/// If `value` is `None`, the option is reset to its default value.
///
/// This is an `xterm`-specific sequence that may not be supported by all
/// terminal emulators.
///
/// # Example
///
/// ```ignore
/// use vtio::event::keyboard::{SetXTermKeyFormatOptions, XTermKeyFormatOption};
/// use vtansi::AnsiEncode;
///
/// // Set backarrow format option
/// let cmd = SetXTermKeyFormatOptions {
///     option: XTermKeyFormatOption::Backarrow,
///     value: Some(1),
/// };
/// assert_eq!(cmd.to_ansi_string(), "\x1b[>0;1f");
///
/// // Reset backarrow to default
/// let cmd = SetXTermKeyFormatOptions {
///     option: XTermKeyFormatOption::Backarrow,
///     value: None,
/// };
/// assert_eq!(cmd.to_ansi_string(), "\x1b[>0f");
/// ```
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html> for the
/// `xterm` specification.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '>', finalbyte = 'f')]
pub struct SetXTermKeyFormatOptions {
    /// The key format option to configure.
    pub option: XTermKeyFormatOption,
    /// The value to set. If `None`, the option is reset to its default.
    pub value: Option<u16>,
}

/// Query `XTerm` Key Format Options (`XTQFMTKEYS`).
///
/// *Sequence*: `CSI ? Pp g`
///
/// Query the current value of an `xterm` key format option.
///
/// The terminal responds with [`XTermKeyFormatOptionsResponse`].
///
/// This is an `xterm`-specific sequence that may not be supported by all
/// terminal emulators.
///
/// # Example
///
/// ```ignore
/// use vtio::event::keyboard::{QueryXTermKeyFormatOptions, XTermKeyFormatOption};
/// use vtansi::AnsiEncode;
///
/// let query = QueryXTermKeyFormatOptions(Some(XTermKeyFormatOption::Backarrow));
/// assert_eq!(query.to_ansi_string(), "\x1b[?0g");
///
/// // Query all format options
/// let query = QueryXTermKeyFormatOptions(None);
/// assert_eq!(query.to_ansi_string(), "\x1b[?g");
/// ```
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html> for the
/// `xterm` specification.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', finalbyte = 'g')]
pub struct QueryXTermKeyFormatOptions(pub Option<XTermKeyFormatOption>);

/// Response to [`QueryXTermKeyFormatOptions`].
///
/// *Sequence*: `CSI > Pp ; Pv f`
///
/// Contains the current value of the queried key format option.
///
/// This is an `xterm`-specific sequence.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, private = '>', finalbyte = 'f')]
pub struct XTermKeyFormatOptionsResponse {
    /// The key format option that was queried.
    pub option: XTermKeyFormatOption,
    /// The current value of the option.
    pub value: u16,
}

// ============================================================================
// Display implementations
// ============================================================================

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

impl fmt::Display for XTermModifierKeyResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModifyKeyboard => write!(f, "modifyKeyboard"),
            Self::ModifyCursorKeys => write!(f, "modifyCursorKeys"),
            Self::ModifyFunctionKeys => write!(f, "modifyFunctionKeys"),
            Self::ModifyOtherKeys => write!(f, "modifyOtherKeys"),
        }
    }
}

impl fmt::Display for XTermKeyFormatOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backarrow => write!(f, "backarrowKey"),
            Self::ApplicationEscape => write!(f, "applicationEscape"),
            Self::PcStyleFunctionKeys => write!(f, "pcStyleFunctionKeys"),
            Self::SunFunctionKeys => write!(f, "sunFunctionKeys"),
            Self::HpFunctionKeys => write!(f, "hpFunctionKeys"),
            Self::ScoFunctionKeys => write!(f, "scoFunctionKeys"),
        }
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

    // ========================================================================
    // XTMODKEYS tests
    // ========================================================================

    #[test]
    fn test_set_xterm_key_modifier_options_with_value() {
        let cmd = SetXTermKeyModifierOptions {
            resource: XTermModifierKeyResource::ModifyOtherKeys,
            value: Some(2),
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>4;2m");
    }

    #[test]
    fn test_set_xterm_key_modifier_options_reset() {
        let cmd = SetXTermKeyModifierOptions {
            resource: XTermModifierKeyResource::ModifyOtherKeys,
            value: None,
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>4m");
    }

    #[test]
    fn test_set_xterm_key_modifier_options_modify_keyboard() {
        let cmd = SetXTermKeyModifierOptions {
            resource: XTermModifierKeyResource::ModifyKeyboard,
            value: Some(1),
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>0;1m");
    }

    #[test]
    fn test_set_xterm_key_modifier_options_cursor_keys() {
        let cmd = SetXTermKeyModifierOptions {
            resource: XTermModifierKeyResource::ModifyCursorKeys,
            value: Some(3),
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>1;3m");
    }

    #[test]
    fn test_set_xterm_key_modifier_options_function_keys() {
        let cmd = SetXTermKeyModifierOptions {
            resource: XTermModifierKeyResource::ModifyFunctionKeys,
            value: Some(2),
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>2;2m");
    }

    #[test]
    fn test_disable_xterm_key_modifier_options() {
        let cmd = DisableXTermKeyModifierOptions(
            XTermModifierKeyResource::ModifyOtherKeys,
        );
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>4n");
    }

    #[test]
    fn test_disable_xterm_key_modifier_options_cursor_keys() {
        let cmd = DisableXTermKeyModifierOptions(
            XTermModifierKeyResource::ModifyCursorKeys,
        );
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>1n");
    }

    #[test]
    fn test_xterm_modifier_key_resource_display() {
        assert_eq!(
            format!("{}", XTermModifierKeyResource::ModifyKeyboard),
            "modifyKeyboard"
        );
        assert_eq!(
            format!("{}", XTermModifierKeyResource::ModifyOtherKeys),
            "modifyOtherKeys"
        );
    }

    // ========================================================================
    // XTFMTKEYS tests
    // ========================================================================

    #[test]
    fn test_set_xterm_key_format_options_with_value() {
        let cmd = SetXTermKeyFormatOptions {
            option: XTermKeyFormatOption::Backarrow,
            value: Some(1),
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>0;1f");
    }

    #[test]
    fn test_set_xterm_key_format_options_reset() {
        let cmd = SetXTermKeyFormatOptions {
            option: XTermKeyFormatOption::Backarrow,
            value: None,
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>0f");
    }

    #[test]
    fn test_set_xterm_key_format_options_pc_style() {
        let cmd = SetXTermKeyFormatOptions {
            option: XTermKeyFormatOption::PcStyleFunctionKeys,
            value: Some(2),
        };
        let mut buf = Vec::new();
        cmd.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[>2;2f");
    }

    #[test]
    fn test_query_xterm_key_format_options() {
        let query =
            QueryXTermKeyFormatOptions(Some(XTermKeyFormatOption::Backarrow));
        let mut buf = Vec::new();
        query.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[?0g");
    }

    #[test]
    fn test_query_xterm_key_format_options_all() {
        let query = QueryXTermKeyFormatOptions(None);
        let mut buf = Vec::new();
        query.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[?g");
    }

    #[test]
    fn test_query_xterm_key_format_options_sun() {
        let query = QueryXTermKeyFormatOptions(Some(
            XTermKeyFormatOption::SunFunctionKeys,
        ));
        let mut buf = Vec::new();
        query.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[?3g");
    }

    #[test]
    fn test_xterm_key_format_option_display() {
        assert_eq!(
            format!("{}", XTermKeyFormatOption::Backarrow),
            "backarrowKey"
        );
        assert_eq!(
            format!("{}", XTermKeyFormatOption::PcStyleFunctionKeys),
            "pcStyleFunctionKeys"
        );
    }
}
