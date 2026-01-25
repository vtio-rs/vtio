//! Key event encoding and decoding.

use super::KeyboardModeFlags;

use vtansi::{
    AnsiEncode, EncodeError, TryFromAnsi, TryFromAnsiIter, write_byte_into,
    write_csi,
};

use super::mode::KeyboardEnhancementFlags;

use super::event::{KeyEvent, KeyEventBuilder};
use super::keycode::{KeyCode, MediaKeyCode};
use super::modifier::{
    CsiKeyModifiers, KeyEventKind, KeyEventState, KeyModifiers,
    ModifierKeyCode, parse_csi_u_modifiers,
};
use super::util::parse_colon_separated;

/// Map Ctrl+<char> to control code (ASCII).
///
/// This is the encoding direction: given a character, produce the control code
/// that would be sent when Ctrl is held. Use [`char_from_control_code`] for the
/// reverse mapping.
#[inline]
pub(crate) fn control_code_for(c: char) -> u8 {
    match c {
        '@' | ' ' => 0x00,
        '[' => 0x1b,
        '\\' => 0x1c,
        ']' => 0x1d,
        '^' => 0x1e,
        '_' => 0x1f,
        '?' => 0x7f,
        _ => c as u8 & 0x1f,
    }
}

/// Map control code (ASCII) back to Ctrl+<char>.
///
/// This is the parsing direction: given a control code byte, produce the
/// character that would generate it when Ctrl is held. Returns `None` for
/// bytes that aren't control codes.
///
/// This is the inverse of [`control_code_for`].
#[inline]
fn char_from_control_code(code: u8) -> Option<char> {
    match code {
        0x00 => Some(' '), // Ctrl+Space or Ctrl+@
        0x01..=0x1A => Some((code - 1 + b'a') as char), // Ctrl+A through Ctrl+Z
        0x1B => Some('['), // Ctrl+[ = ESC
        0x1C => Some('\\'), // Ctrl+\
        0x1D => Some(']'), // Ctrl+]
        0x1E => Some('^'), // Ctrl+^
        0x1F => Some('_'), // Ctrl+_
        0x7F => Some('?'), // Ctrl+? = DEL
        _ => None,
    }
}

/// Parse a raw byte into a `(KeyCode, KeyModifiers)` pair.
///
/// This handles:
/// - Control codes (0x00-0x1F, 0x7F) → Ctrl+key or special keys
/// - Printable ASCII (0x20-0x7E) → character keys with optional SHIFT
///
/// Returns `None` for non-ASCII bytes.
#[inline]
pub(crate) fn byte_to_key(byte: u8) -> Option<(KeyCode, KeyModifiers)> {
    match byte {
        // Special keys that map to dedicated KeyCodes
        b'\x1B' => Some((KeyCode::Esc, KeyModifiers::NONE)),
        b'\t' => Some((KeyCode::Tab, KeyModifiers::NONE)),
        b'\r' | b'\n' => Some((KeyCode::Enter, KeyModifiers::NONE)),
        b'\x7F' => Some((KeyCode::Backspace, KeyModifiers::NONE)),
        // Control codes → Ctrl+char
        c @ (0x00 | 0x01..=0x1A | 0x1C..=0x1F) => {
            let ch = char_from_control_code(c)?;
            Some((KeyCode::Char(ch), KeyModifiers::CONTROL))
        }
        // Printable ASCII
        c if c.is_ascii() && !c.is_ascii_control() => {
            let ch = c as char;
            let modifiers = if ch.is_ascii_uppercase() {
                KeyModifiers::SHIFT
            } else {
                KeyModifiers::NONE
            };
            Some((KeyCode::Char(ch), modifiers))
        }
        _ => None,
    }
}

/// Convert a `(KeyCode, KeyModifiers)` to raw byte representation.
///
/// This is the inverse of [`byte_to_key`]. Returns `None` for keys that
/// cannot be represented as a single byte (non-ASCII, function keys, etc.).
#[inline]
pub(crate) fn key_to_byte(code: KeyCode, mods: KeyModifiers) -> Option<u8> {
    match code {
        KeyCode::Esc => Some(0x1B),
        KeyCode::Tab => Some(b'\t'),
        KeyCode::Enter => Some(b'\r'),
        KeyCode::Backspace => Some(0x7F),
        KeyCode::Char(ch) => {
            if mods.contains(KeyModifiers::CONTROL) {
                Some(control_code_for(ch))
            } else if mods.contains(KeyModifiers::SHIFT)
                && ch.is_ascii_lowercase()
            {
                Some(ch.to_ascii_uppercase() as u8)
            } else if ch.is_ascii() {
                Some(ch as u8)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Generates bidirectional byte↔key conversion functions from a single mapping definition.
///
/// This macro eliminates the need to maintain separate lookup tables and match statements
/// that must be kept in sync. It generates optimized match statements for both directions.
///
/// # Syntax
///
/// ```ignore
/// key_byte_conversions! {
///     byte_to_key_fn, key_to_byte_fn, {
///         [byte1, byte2, ...] => KeyVariant,           // Multiple bytes map to same key
///         [byte] => KeyVariant(arg),                   // Single byte with key argument
///     }
/// }
/// ```
///
/// The first byte in each list is used as the canonical encoding for key→byte conversion.
/// All listed bytes are accepted for byte→key conversion.
macro_rules! key_byte_conversions {
    (
        $byte_to_key:ident, $key_to_byte:ident, {
            $( [$first:expr $(, $rest:expr)*] => $key:ident $( ( $($arg:tt)* ) )? ),* $(,)?
        }
    ) => {
        #[inline]
        fn $byte_to_key(byte: u8) -> Option<KeyCode> {
            match byte {
                $( $first $( | $rest )* => Some(KeyCode::$key $( ( $($arg)* ) )?), )*
                _ => None,
            }
        }

        #[inline]
        fn $key_to_byte(key: KeyCode) -> Option<u8> {
            match key {
                $( KeyCode::$key $( ( $($arg)* ) )? => Some($first), )*
                _ => None,
            }
        }
    };
}

/// Generates bidirectional u32↔KeyCode conversion functions for CSI u sequences.
///
/// Similar to `key_byte_conversions` but uses `u32` to support Unicode codepoints
/// and functional key codes in the Private Use Area (57344-63743).
macro_rules! csi_u_key_conversions {
    (
        $code_to_key:ident, $key_to_code:ident, {
            $( [$first:expr $(, $rest:expr)*] => $key:expr ),* $(,)?
        }
    ) => {
        #[inline]
        fn $code_to_key(code: u32) -> Option<KeyCode> {
            match code {
                $( $first $( | $rest )* => Some($key), )*
                _ => None,
            }
        }

        #[inline]
        #[allow(dead_code)]
        fn $key_to_code(key: KeyCode) -> Option<u32> {
            match key {
                $( k if k == $key => Some($first), )*
                _ => None,
            }
        }
    };
}

// CSI u functional key code ↔ KeyCode conversions
//
// These are the functional key codes used in the Kitty keyboard protocol's
// CSI u sequences. Codes 57344-63743 are in the Unicode Private Use Area.
// See: https://sw.kovidgoyal.net/kitty/keyboard-protocol/#functional-key-definitions
csi_u_key_conversions! {
    csi_u_code_to_key, key_to_csi_u_code, {
        // Special keys with legacy codes
        [27] => KeyCode::Esc,
        [13] => KeyCode::Enter,
        [9] => KeyCode::Tab,
        [127] => KeyCode::Backspace,

        // Navigation keys (these also have CSI letter forms)
        // INSERT, DELETE, PAGE_UP, PAGE_DOWN use CSI ~ format primarily
        // LEFT, RIGHT, UP, DOWN, HOME, END use CSI letter format primarily

        // Lock keys
        [57358] => KeyCode::CapsLock,
        [57359] => KeyCode::ScrollLock,
        [57360] => KeyCode::NumLock,

        // Other special keys
        [57361] => KeyCode::PrintScreen,
        [57362] => KeyCode::Pause,
        [57363] => KeyCode::Menu,

        // Function keys (F1-F12)
        [57344] => KeyCode::F(1),
        [57345] => KeyCode::F(2),
        [57346] => KeyCode::F(3),
        [57347] => KeyCode::F(4),
        [57348] => KeyCode::F(5),
        [57349] => KeyCode::F(6),
        [57350] => KeyCode::F(7),
        [57351] => KeyCode::F(8),
        [57352] => KeyCode::F(9),
        [57353] => KeyCode::F(10),
        [57354] => KeyCode::F(11),
        [57355] => KeyCode::F(12),

        // Extended function keys (F13-F35)
        [57376] => KeyCode::F(13),
        [57377] => KeyCode::F(14),
        [57378] => KeyCode::F(15),
        [57379] => KeyCode::F(16),
        [57380] => KeyCode::F(17),
        [57381] => KeyCode::F(18),
        [57382] => KeyCode::F(19),
        [57383] => KeyCode::F(20),
        [57384] => KeyCode::F(21),
        [57385] => KeyCode::F(22),
        [57386] => KeyCode::F(23),
        [57387] => KeyCode::F(24),
        [57388] => KeyCode::F(25),
        [57389] => KeyCode::F(26),
        [57390] => KeyCode::F(27),
        [57391] => KeyCode::F(28),
        [57392] => KeyCode::F(29),
        [57393] => KeyCode::F(30),
        [57394] => KeyCode::F(31),
        [57395] => KeyCode::F(32),
        [57396] => KeyCode::F(33),
        [57397] => KeyCode::F(34),
        [57398] => KeyCode::F(35),

        // Keypad keys
        [57399] => KeyCode::Char('0'),  // KP_0
        [57400] => KeyCode::Char('1'),  // KP_1
        [57401] => KeyCode::Char('2'),  // KP_2
        [57402] => KeyCode::Char('3'),  // KP_3
        [57403] => KeyCode::Char('4'),  // KP_4
        [57404] => KeyCode::Char('5'),  // KP_5
        [57405] => KeyCode::Char('6'),  // KP_6
        [57406] => KeyCode::Char('7'),  // KP_7
        [57407] => KeyCode::Char('8'),  // KP_8
        [57408] => KeyCode::Char('9'),  // KP_9
        [57409] => KeyCode::Char('.'),  // KP_DECIMAL
        [57410] => KeyCode::Char('/'),  // KP_DIVIDE
        [57411] => KeyCode::Char('*'),  // KP_MULTIPLY
        [57412] => KeyCode::Char('-'),  // KP_SUBTRACT
        [57413] => KeyCode::Char('+'),  // KP_ADD
        [57414] => KeyCode::Enter,      // KP_ENTER
        [57415] => KeyCode::Char('='),  // KP_EQUAL
        [57416] => KeyCode::Char(','),  // KP_SEPARATOR
        [57417] => KeyCode::Left,       // KP_LEFT
        [57418] => KeyCode::Right,      // KP_RIGHT
        [57419] => KeyCode::Up,         // KP_UP
        [57420] => KeyCode::Down,       // KP_DOWN
        [57421] => KeyCode::PageUp,     // KP_PAGE_UP
        [57422] => KeyCode::PageDown,   // KP_PAGE_DOWN
        [57423] => KeyCode::Home,       // KP_HOME
        [57424] => KeyCode::End,        // KP_END
        [57425] => KeyCode::Insert,     // KP_INSERT
        [57426] => KeyCode::Delete,     // KP_DELETE
        [57427] => KeyCode::KeypadBegin, // KP_BEGIN

        // Media keys
        [57428] => KeyCode::Media(MediaKeyCode::Play),
        [57429] => KeyCode::Media(MediaKeyCode::Pause),
        [57430] => KeyCode::Media(MediaKeyCode::PlayPause),
        [57431] => KeyCode::Media(MediaKeyCode::Reverse),
        [57432] => KeyCode::Media(MediaKeyCode::Stop),
        [57433] => KeyCode::Media(MediaKeyCode::FastForward),
        [57434] => KeyCode::Media(MediaKeyCode::Rewind),
        [57435] => KeyCode::Media(MediaKeyCode::TrackNext),
        [57436] => KeyCode::Media(MediaKeyCode::TrackPrevious),
        [57437] => KeyCode::Media(MediaKeyCode::Record),
        [57438] => KeyCode::Media(MediaKeyCode::LowerVolume),
        [57439] => KeyCode::Media(MediaKeyCode::RaiseVolume),
        [57440] => KeyCode::Media(MediaKeyCode::MuteVolume),

        // Modifier keys
        [57441] => KeyCode::Modifier(ModifierKeyCode::LeftShift),
        [57442] => KeyCode::Modifier(ModifierKeyCode::LeftControl),
        [57443] => KeyCode::Modifier(ModifierKeyCode::LeftAlt),
        [57444] => KeyCode::Modifier(ModifierKeyCode::LeftSuper),
        [57445] => KeyCode::Modifier(ModifierKeyCode::LeftHyper),
        [57446] => KeyCode::Modifier(ModifierKeyCode::LeftMeta),
        [57447] => KeyCode::Modifier(ModifierKeyCode::RightShift),
        [57448] => KeyCode::Modifier(ModifierKeyCode::RightControl),
        [57449] => KeyCode::Modifier(ModifierKeyCode::RightAlt),
        [57450] => KeyCode::Modifier(ModifierKeyCode::RightSuper),
        [57451] => KeyCode::Modifier(ModifierKeyCode::RightHyper),
        [57452] => KeyCode::Modifier(ModifierKeyCode::RightMeta),
        [57453] => KeyCode::Modifier(ModifierKeyCode::IsoLevel3Shift),
        [57454] => KeyCode::Modifier(ModifierKeyCode::IsoLevel5Shift),
    }
}

/// Check if a CSI u code represents a keypad key.
#[inline]
fn is_csi_u_keypad_code(code: u32) -> bool {
    (57399..=57427).contains(&code)
}

// CSI final byte ↔ KeyCode conversions
//
// These keys use letter final bytes in CSI (`ESC [`) sequences:
// - Cursor keys: A (Up), B (Down), C (Right), D (Left)
// - Navigation: F (End), H (Home)
// - Function keys: P (F1), Q (F2), R (F3), S (F4)
// - BackTab: Z (CSI-only, not valid in SS3)
key_byte_conversions! {
    csi_final_byte_to_key, key_to_csi_final_byte, {
        [b'A'] => Up,
        [b'B'] => Down,
        [b'C'] => Right,
        [b'D'] => Left,
        [b'F'] => End,
        [b'H'] => Home,
        [b'P'] => F(1),
        [b'Q'] => F(2),
        [b'R'] => F(3),
        [b'S'] => F(4),
        [b'Z'] => BackTab,
    }
}

// CSI tilde code ↔ KeyCode conversions
//
// These keys use numeric codes followed by `~` in CSI sequences (`ESC [ <code> ~`):
// - Navigation: Home (1/7), Insert (2), Delete (3), End (4/8), PageUp (5), PageDown (6)
// - Function keys: F5-F20 (15-34, with gaps)
//
// Note: Home and End have both VT220 (1/4) and xterm (7/8) codes.
// The VT220 codes are used as the canonical encoding.
key_byte_conversions! {
    csi_tilde_code_to_key, key_to_csi_tilde_code, {
        [1, 7] => Home,       // VT220 (1) and xterm (7)
        [2] => Insert,
        [3] => Delete,
        [4, 8] => End,        // VT220 (4) and xterm (8)
        [5] => PageUp,
        [6] => PageDown,
        [15] => F(5),
        [17] => F(6),
        [18] => F(7),
        [19] => F(8),
        [20] => F(9),
        [21] => F(10),
        [23] => F(11),
        [24] => F(12),
        [25] => F(13),
        [26] => F(14),
        [28] => F(15),
        [29] => F(16),
        [31] => F(17),
        [32] => F(18),
        [33] => F(19),
        [34] => F(20),
    }
}

// SS3 data byte ↔ KeyCode conversions
//
// These are sent as `ESC O <byte>` sequences when application cursor/keypad mode is active:
// - Cursor keys and F1-F4 (shared with CSI final bytes)
// - Keypad keys in application mode (Enter, digits, operators)
//
// NOTE: The cursor keys and F1-F4 mappings are intentionally duplicated from
// `csi_final_byte_to_key` above for performance reasons.
key_byte_conversions! {
    ss3_byte_to_key, key_to_ss3_byte, {
        // Cursor keys and F1-F4
        [b'A'] => Up,
        [b'B'] => Down,
        [b'C'] => Right,
        [b'D'] => Left,
        [b'F'] => End,
        [b'H'] => Home,
        [b'P'] => F(1),
        [b'Q'] => F(2),
        [b'R'] => F(3),
        [b'S'] => F(4),
        // Keypad keys in application mode
        [b'M'] => Enter,
        [b'X'] => Char('='),
        [b'j'] => Char('*'),
        [b'k'] => Char('+'),
        [b'l'] => Char(','),
        [b'm'] => Char('-'),
        [b'n'] => Char('.'),
        [b'o'] => Char('/'),
        [b'p'] => Char('0'),
        [b'q'] => Char('1'),
        [b'r'] => Char('2'),
        [b's'] => Char('3'),
        [b't'] => Char('4'),
        [b'u'] => Char('5'),
        [b'v'] => Char('6'),
        [b'w'] => Char('7'),
        [b'x'] => Char('8'),
        [b'y'] => Char('9'),
    }
}

/// Represents how a `KeyEvent` should be encoded as an ANSI sequence.
///
/// This enum unifies the different encoding strategies for keyboard events,
/// allowing `AnsiEncode for KeyEvent` to delegate to the appropriate encoding
/// implementation rather than duplicating all the logic inline.
///
/// The variants correspond to different terminal input sequence formats:
/// - Raw bytes for simple characters and control codes
/// - ESC-prefixed bytes for Alt combinations
/// - SS3 sequences (ESC O) for application mode keys
/// - CSI sequences (ESC [) with various parameter formats
#[derive(Debug, Clone, PartialEq, Eq)]
enum KeyEncoding {
    /// Single raw byte (control codes, simple ASCII chars).
    Raw(u8),
    /// UTF-8 encoded character, optionally with ESC prefix for Alt.
    Char { alt_prefix: bool, ch: char },
    /// SS3 sequence: `ESC O <final_byte>`.
    Ss3(u8),
    /// CSI with letter final byte, no modifiers: `ESC [ <final_byte>`.
    CsiFinal(u8),
    /// CSI with letter final byte and modifiers: `ESC [ 1 ; <mods> <final_byte>`.
    CsiModFinal { mods: u8, final_byte: u8 },
    /// CSI with numeric code and tilde: ESC [ <code> ~.
    CsiTilde(u8),
    /// CSI with numeric code, modifiers, and tilde: ESC [ <code> ; <mods> ~.
    CsiModTilde { code: u8, mods: u8 },
    /// CSI-u format for keys with modifiers: ESC [ <code> ; <mods> u.
    CsiU { code: u8, mods: u8 },
    /// No encoding (unsupported keys like Media, Modifier, or non-press events).
    None,
}

impl KeyEncoding {
    /// Determine the encoding strategy for a key event.
    ///
    /// This method maps a `KeyEvent` to the appropriate ANSI encoding format.
    /// The encoding is determined by the key code and modifiers:
    ///
    /// - Character keys: raw byte, control code, or UTF-8 with optional ESC prefix
    /// - Special keys (Enter, Tab, Esc, Backspace): raw bytes or CSI-u with modifiers
    /// - Navigation keys (arrows, Home, End, F1-F4): CSI or SS3 sequences
    /// - Extended function keys (F5-F20, Insert, Delete, PageUp/Down): CSI tilde sequences
    fn from_key_event(event: &KeyEvent) -> Self {
        Self::from_key_event_with_modes(event, KeyboardModeFlags::empty())
    }

    /// Determine the encoding strategy for a key event, respecting terminal mode flags.
    ///
    /// This method is like [`from_key_event`] but takes into account the current
    /// terminal mode settings which affect how certain keys are encoded:
    ///
    /// - `CURSOR_KEYS`: When set, cursor keys use SS3 (ESC O) instead of CSI (ESC [)
    /// - `APPLICATION_KEYPAD`: When set, keypad keys use SS3 sequences
    /// - `BACKSPACE_SENDS_DELETE`: When set, Backspace sends BS (0x08) instead of DEL (0x7F)
    /// - `ALT_KEY_HIGH_BIT_SET`: When set, Alt sets high bit instead of ESC prefix
    /// - `DELETE_KEY_SENDS_DEL`: When set, Delete key sends DEL (0x7F) instead of escape sequence
    fn from_key_event_with_modes(
        event: &KeyEvent,
        mode_flags: KeyboardModeFlags,
    ) -> Self {
        // Only encode press events
        if event.kind != KeyEventKind::Press {
            return Self::None;
        }

        let mods = event.modifiers;
        let mod_param = mods.to_xterm_param();
        let has_mods = mod_param > 1;
        let code = event.code;

        // Determine Alt key behavior based on mode flags
        // ALT_KEY_HIGH_BIT_SET: set high bit instead of ESC prefix
        // Default (or ALT_KEY_SENDS_ESC_PREFIX): use ESC prefix
        let alt_high_bit =
            mode_flags.contains(KeyboardModeFlags::ALT_KEY_HIGH_BIT_SET);
        let alt_prefix = mods.contains(KeyModifiers::ALT) && !alt_high_bit;
        let alt_set_high_bit = mods.contains(KeyModifiers::ALT) && alt_high_bit;

        // Check cursor keys mode (DECCKM) - when set, cursor keys use SS3
        let cursor_keys_mode =
            mode_flags.contains(KeyboardModeFlags::CURSOR_KEYS);

        // Note: APPLICATION_KEYPAD mode (DECNKM) is not handled here because
        // we cannot distinguish keypad keys from main keyboard keys at the
        // KeyCode level - both appear as KeyCode::Char('5'), etc.

        // Character keys: control codes, shifted chars, or plain UTF-8
        if let KeyCode::Char(c) = code {
            return Self::encode_char(c, mods, alt_prefix, alt_set_high_bit);
        }

        // Special keys with raw byte or CSI-u encoding
        match code {
            KeyCode::Enter => {
                // In application keypad mode, Enter from keypad uses SS3 M
                // For regular Enter, use CR or CSI-u with modifiers
                return if has_mods {
                    Self::CsiU {
                        code: 13,
                        mods: mod_param,
                    }
                } else {
                    Self::Raw(b'\r')
                };
            }
            KeyCode::Backspace => {
                // BACKSPACE_SENDS_DELETE mode: when SET, backspace sends BS (0x08)
                // When RESET (default), backspace sends DEL (0x7F)
                // Note: The mode name is confusing - "delete" here means the DEL character
                return if mode_flags
                    .contains(KeyboardModeFlags::BACKSPACE_SENDS_DELETE)
                {
                    Self::Raw(0x08) // BS
                } else {
                    Self::Raw(0x7f) // DEL
                };
            }
            KeyCode::Delete => {
                // DELETE_KEY_SENDS_DEL mode: when set, Delete sends DEL (0x7F)
                // instead of the escape sequence
                if mode_flags.contains(KeyboardModeFlags::DELETE_KEY_SENDS_DEL)
                {
                    return Self::Raw(0x7f);
                }
                // Otherwise fall through to CSI tilde encoding below
            }
            KeyCode::Tab => {
                return if mods.contains(KeyModifiers::SHIFT) {
                    Self::CsiFinal(b'Z') // BackTab
                } else {
                    Self::Raw(b'\t')
                };
            }
            KeyCode::Esc => return Self::Raw(0x1b),
            _ => {}
        }

        // Navigation keys: CSI final byte format (cursor keys, Home/End, F1-F4)
        if let Some(final_byte) = key_to_csi_final_byte(code) {
            // Determine if we should use SS3 encoding:
            // - F1-F4: SS3 without modifiers (traditional behavior)
            // - Cursor keys: SS3 when CURSOR_KEYS mode (DECCKM) is set and no modifiers
            // - Home/End: SS3 when CURSOR_KEYS mode is set and no modifiers (some terminals)
            let is_cursor_key = matches!(
                code,
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
            );
            let is_f1_f4 = matches!(code, KeyCode::F(1..=4));
            let is_home_end = matches!(code, KeyCode::Home | KeyCode::End);

            let use_ss3 = !has_mods
                && (is_f1_f4
                    || (is_cursor_key && cursor_keys_mode)
                    || (is_home_end && cursor_keys_mode));

            return if use_ss3 {
                Self::Ss3(final_byte)
            } else if has_mods {
                Self::CsiModFinal {
                    mods: mod_param,
                    final_byte,
                }
            } else {
                Self::CsiFinal(final_byte)
            };
        }

        // Extended keys: CSI tilde format (Insert, Delete, PageUp/Down, F5-F20)
        // Note: Home/End have both final byte (H/F) and tilde codes (1/4);
        // we prefer the final byte format, so they're handled above.
        if let Some(tilde_code) = key_to_csi_tilde_code(code) {
            // Skip Home/End here since they were handled by csi_final_byte above
            if !matches!(code, KeyCode::Home | KeyCode::End) {
                return if has_mods {
                    Self::CsiModTilde {
                        code: tilde_code,
                        mods: mod_param,
                    }
                } else {
                    Self::CsiTilde(tilde_code)
                };
            }
        }

        Self::None
    }

    /// Encode a character key based on modifiers.
    ///
    /// - With CONTROL: produces control code (0x00-0x1F, 0x7F)
    /// - With SHIFT on lowercase: produces uppercase
    /// - With ALT: adds ESC prefix or sets high bit depending on mode
    #[inline]
    fn encode_char(
        c: char,
        mods: KeyModifiers,
        alt_prefix: bool,
        alt_set_high_bit: bool,
    ) -> Self {
        if mods.contains(KeyModifiers::CONTROL) {
            let ctrl = control_code_for(c);
            if alt_prefix {
                // Alt+Ctrl+char: ESC followed by control code
                Self::Char {
                    alt_prefix: true,
                    ch: ctrl as char,
                }
            } else if alt_set_high_bit {
                // Alt+Ctrl+char with high bit mode: set bit 7
                Self::Raw(ctrl | 0x80)
            } else {
                Self::Raw(ctrl)
            }
        } else {
            // Regular char, possibly with Shift (handled by case) or Alt
            let ch = if mods.contains(KeyModifiers::SHIFT)
                && c.is_ascii_lowercase()
            {
                c.to_ascii_uppercase()
            } else {
                c
            };

            if alt_set_high_bit && ch.is_ascii() {
                // Alt with high bit mode: set bit 7 on ASCII characters
                Self::Raw((ch as u8) | 0x80)
            } else {
                Self::Char { alt_prefix, ch }
            }
        }
    }
}

impl AnsiEncode for KeyEncoding {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        buf: &mut W,
    ) -> Result<usize, EncodeError> {
        match self {
            Self::None => Ok(0),

            Self::Raw(byte) => vtansi::write_byte_into(buf, *byte),

            Self::Char { alt_prefix, ch } => {
                let mut total = 0;
                if *alt_prefix {
                    total += vtansi::write_byte_into(buf, 0x1b)?;
                }
                let mut tmp = [0u8; 4];
                let s = ch.encode_utf8(&mut tmp);
                total += vtansi::write_bytes_into(buf, s.as_bytes())?;
                Ok(total)
            }

            Self::Ss3(final_byte) => {
                vtansi::write_bytes_into(buf, &[0x1b, b'O', *final_byte])
            }

            Self::CsiFinal(final_byte) => {
                vtansi::write_bytes_into(buf, &[0x1b, b'[', *final_byte])
            }

            Self::CsiModFinal { mods, final_byte } => {
                write_csi!(buf; "1;", *mods, *final_byte as char)
            }

            Self::CsiTilde(code) => {
                write_csi!(buf; *code, "~")
            }

            Self::CsiModTilde { code, mods } => {
                write_csi!(buf; *code, ";", *mods, "~")
            }

            Self::CsiU { code, mods } => {
                write_csi!(buf; *code, ";", *mods, "u")
            }
        }
    }
}

impl AnsiEncode for KeyEvent {
    #[inline]
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        buf: &mut W,
    ) -> Result<usize, EncodeError> {
        KeyEncoding::from_key_event(self).encode_ansi_into(buf)
    }
}

// ============================================================================
// Keyboard Enhancement Flags Encoding
// ============================================================================

/// Internal enum for choosing between legacy and CSI u encoding.
#[derive(Debug, Clone)]
enum EnhancedKeyEncoding {
    /// Legacy encoding (SS3, CSI ~, etc.)
    Legacy(KeyEncoding),
    /// CSI u encoding (kitty keyboard protocol)
    CsiU(CsiUKeyEventSeq),
}

impl AnsiEncode for EnhancedKeyEncoding {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, EncodeError> {
        match self {
            Self::Legacy(enc) => enc.encode_ansi_into(sink),
            Self::CsiU(seq) => seq.encode_ansi_into(sink),
        }
    }
}

/// Choose a key event according to the given keyboard enhancement flags and mode flags.
///
/// When `DISAMBIGUATE_ESCAPE_CODES` or `REPORT_ALL_KEYS_AS_ESCAPE_CODES` flags
/// are set, the key event would be encoded using the CSI u format (kitty keyboard
/// protocol). Otherwise, it uses the legacy terminal encoding.
///
/// The `mode_flags` parameter is a [`KeyboardModeFlags`] bitmask of keyboard modes,
/// typically collected from terminal mode query responses using
/// [`crate::event::keyboard::collect_mode_flags`].
///
/// # Example
///
/// ```ignore
/// use vtio::event::keyboard::{
///     KeyEvent,
///     KeyboardEnhancementFlags,
///     get_key_event_encoding,
/// };
/// use vtio::event::mode::collect_mode_flags;
/// use vtansi::AnsiEncode;
///
/// let event = KeyEvent::from(KeyCode::Enter);
/// let flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES;
///
/// // Collect mode flags from queried terminal modes
/// let modes = [cursor_keys_mode, app_keypad_mode];
/// let mode_flags = collect_mode_flags(&modes);
///
/// let mut buf = Vec::new();
/// get_key_event_encoding(&event, flags, mode_flags).encode_ansi_into(&mut buf)?;
/// ```
#[must_use]
pub fn get_key_event_encoding(
    event: &KeyEvent,
    flags: KeyboardEnhancementFlags,
    mode_flags: KeyboardModeFlags,
) -> impl AnsiEncode {
    let use_csi_u = flags
        .contains(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        || flags.contains(
            KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
        );

    if use_csi_u {
        EnhancedKeyEncoding::CsiU(CsiUKeyEventSeq(CsiUKeyEvent(event.clone())))
    } else {
        EnhancedKeyEncoding::Legacy(KeyEncoding::from_key_event_with_modes(
            event, mode_flags,
        ))
    }
}

// ============================================================================
// Key type macros and definitions
// ============================================================================

/// Helper macro to apply standard derives for key event structs.
macro_rules! key_struct_derives {
    (
        $(#[doc = $doc:expr])*
        #[vtansi($($vtansi_args:tt)*)]
        pub struct $($rest:tt)*
    ) => {
        $(#[doc = $doc])*
        #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, vtansi::derive::AnsiInput)]
        #[vtansi($($vtansi_args)*, into = KeyEvent)]
        pub struct $($rest)*
    };
}

/// Helper macro to generate `From<T> for KeyEvent` implementations.
///
/// Note: `From<T>` provides a blanket `TryFrom<T>` impl, so types using
/// `#[vtansi(..., into = KeyEvent)]` will work correctly.
macro_rules! key_from {
    // Unit struct -> KeyEvent with specific code and modifiers
    ($name:ident => $keycode:expr, $modifiers:expr) => {
        impl From<$name> for KeyEvent {
            fn from(_: $name) -> Self {
                KeyEvent::new($keycode, $modifiers)
            }
        }
    };
    // Wrapper struct (newtype) -> KeyEvent via inner's Into
    (wrapper $name:ident) => {
        impl From<$name> for KeyEvent {
            fn from(key: $name) -> Self {
                key.0.into()
            }
        }
    };
    // Struct with .key field -> KeyEvent via key's Into
    (key_field $name:ident) => {
        impl From<$name> for KeyEvent {
            fn from(key: $name) -> Self {
                key.key.into()
            }
        }
    };
}

/// Macro to generate key event types with reduced boilerplate.
///
/// # Patterns
///
/// **Byte-code key** - generates struct + `TryFrom`:
/// ```ignore
/// key_event! {
///     /// Doc comment
///     #[vtansi(byte, code = 0x1b)]
///     EscKey => KeyCode::Esc
/// }
/// ```
///
/// **Key with modifiers struct** - generates struct with `code`/`modifiers` fields:
/// ```ignore
/// key_event!(AltKey);
/// ```
///
/// **ESC-prefixed key** - generates base struct + `FooSeq` wrapper:
/// ```ignore
/// key_event! {
///     #[vtansi(esc)]
///     AltKey
/// }
/// ```
///
/// **SS3-prefixed key** - generates base struct + `FooSeq` wrapper:
/// ```ignore
/// key_event! {
///     #[vtansi(ss3)]
///     Ss3Key
/// }
/// ```
macro_rules! key_event {
    // Byte-code key (e.g., ESC = 0x1b)
    (
        $(#[doc = $doc:expr])*
        #[vtansi(byte, code = $code:expr)]
        $name:ident => $keycode:expr
    ) => {
        key_struct_derives! {
            $(#[doc = $doc])*
            #[vtansi(byte, code = $code)]
            pub struct $name;
        }
        key_from!($name => $keycode, KeyModifiers::empty());
    };

    // ESC-prefixed wrapper (newtype around inner type)
    (
        $(#[doc = $doc:expr])*
        #[vtansi(esc)]
        $name:ident => $inner:ty
    ) => {
        key_struct_derives! {
            $(#[doc = $doc])*
            #[vtansi(esc)]
            pub struct $name(pub $inner);
        }
        key_from!(wrapper $name);
    };

    // ESC-prefixed key with modifiers (generates base + Seq wrapper)
    (
        $(#[doc = $doc:expr])*
        #[vtansi(esc)]
        $name:ident
    ) => {
        key_event!($name);

        paste::paste! {
            key_struct_derives! {
                $(#[doc = $doc])*
                #[vtansi(esc)]
                pub struct [<$name Seq>](pub $name);
            }
            key_from!(wrapper [<$name Seq>]);
        }
    };

    // SS3-prefixed key with modifiers (generates base + Seq wrapper)
    (
        $(#[doc = $doc:expr])*
        #[vtansi(ss3)]
        $name:ident
    ) => {
        key_event!($name);

        paste::paste! {
            key_struct_derives! {
                $(#[doc = $doc])*
                #[vtansi(ss3)]
                pub struct [<$name Seq>] { pub key: $name, }
            }
            key_from!(key_field [<$name Seq>]);
        }
    };

    // Key with modifiers struct (full definition with From impls)
    ($name:ident) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash)]
        pub struct $name {
            pub code: KeyCode,
            pub modifiers: KeyModifiers,
        }

        impl $name {
            #[must_use]
            pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
                Self { code, modifiers }
            }
        }

        impl From<&$name> for KeyEvent {
            #[inline]
            fn from(value: &$name) -> Self {
                Self::new(value.code, value.modifiers)
            }
        }

        impl From<$name> for KeyEvent {
            #[inline]
            fn from(value: $name) -> Self {
                KeyEvent::from(&value)
            }
        }

        impl From<KeyCode> for $name {
            #[inline]
            fn from(value: KeyCode) -> Self {
                Self::new(value, KeyModifiers::empty())
            }
        }
    };
}

key_event! {
    /// Escape
    #[vtansi(byte, code = 0x1b)]
    EscKey => KeyCode::Esc
}

key_event! {
    /// Enter (line-feed)
    #[vtansi(byte, code = 0x0A)]
    EnterKey => KeyCode::Enter
}

key_event! {
    /// Backspace
    #[vtansi(byte, code = 0x7F)]
    Bs => KeyCode::Backspace
}

/// Macro to generate individual Ctrl+key definitions for control code ranges.
macro_rules! ctrl_keys {
    ($(
        $(#[doc = $doc:expr])*
        $name:ident: $code:expr => $keycode:expr
    );* $(;)?) => {$(
        key_struct_derives! {
            $(#[doc = $doc])*
            #[vtansi(byte, code = $code)]
            pub struct $name;
        }
        key_from!($name => $keycode, KeyModifiers::CONTROL);
        )*
    };
}

// Ctrl+key definitions using C0 control codes
ctrl_keys! {
    /// Ctrl+Space (NUL)
    CtrlSpace: 0x00 => KeyCode::Char(' ');
    /// Ctrl+A (SOH)
    CtrlA: 0x01 => KeyCode::Char('a');
    /// Ctrl+B (STX)
    CtrlB: 0x02 => KeyCode::Char('b');
    /// Ctrl+C (ETX)
    CtrlC: 0x03 => KeyCode::Char('c');
    /// Ctrl+D (EOT)
    CtrlD: 0x04 => KeyCode::Char('d');
    /// Ctrl+E (ENQ)
    CtrlE: 0x05 => KeyCode::Char('e');
    /// Ctrl+F (ACK)
    CtrlF: 0x06 => KeyCode::Char('f');
    /// Ctrl+G (BEL)
    CtrlG: 0x07 => KeyCode::Char('g');
    /// Ctrl+I (HT) - Tab
    CtrlI: 0x09 => KeyCode::Char('i');
    /// Ctrl+K (VT)
    CtrlK: 0x0B => KeyCode::Char('k');
    /// Ctrl+L (FF)
    CtrlL: 0x0C => KeyCode::Char('l');
    /// Ctrl+M (CR) - Enter
    CtrlM: 0x0D => KeyCode::Char('m');
    /// Ctrl+N (SO)
    CtrlN: 0x0E => KeyCode::Char('n');
    /// Ctrl+O (SI)
    CtrlO: 0x0F => KeyCode::Char('o');
    /// Ctrl+P (DLE)
    CtrlP: 0x10 => KeyCode::Char('p');
    /// Ctrl+Q (DC1)
    CtrlQ: 0x11 => KeyCode::Char('q');
    /// Ctrl+R (DC2)
    CtrlR: 0x12 => KeyCode::Char('r');
    /// Ctrl+S (DC3)
    CtrlS: 0x13 => KeyCode::Char('s');
    /// Ctrl+T (DC4)
    CtrlT: 0x14 => KeyCode::Char('t');
    /// Ctrl+U (NAK)
    CtrlU: 0x15 => KeyCode::Char('u');
    /// Ctrl+V (SYN)
    CtrlV: 0x16 => KeyCode::Char('v');
    /// Ctrl+W (ETB)
    CtrlW: 0x17 => KeyCode::Char('w');
    /// Ctrl+X (CAN)
    CtrlX: 0x18 => KeyCode::Char('x');
    /// Ctrl+Y (EM)
    CtrlY: 0x19 => KeyCode::Char('y');
    /// Ctrl+Z (SUB)
    CtrlZ: 0x1A => KeyCode::Char('z');
}

// CtrlH and CtrlJ are defined separately as aliases since they conflict
// with Backspace (0x08) and EnterKey (0x0A) respectively.

/// Ctrl+H (BS) - Backspace
///
/// This is an alias for the Backspace control code (0x08). When parsing input,
/// the byte 0x08 will be recognized as `Backspace` or converted to a `KeyEvent`
/// with `KeyCode::Backspace`. Use this type when you specifically want to
/// represent the Ctrl+H key combination.
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
#[vtansi(byte, code = 0x08, into = KeyEvent, alias_of = crate::event::cursor::Backspace)]
pub struct CtrlH;

impl From<CtrlH> for KeyEvent {
    fn from(_: CtrlH) -> Self {
        KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL)
    }
}

/// Ctrl+J (LF) - Line Feed
///
/// This is an alias for the Enter/Line Feed control code (0x0A). When parsing
/// input, the byte 0x0A will be recognized as `EnterKey` and converted to a
/// `KeyEvent` with `KeyCode::Enter`. Use this type when you specifically want
/// to represent the Ctrl+J key combination.
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
#[vtansi(byte, code = 0x0A, into = KeyEvent, alias_of = EnterKey)]
pub struct CtrlJ;

impl From<CtrlJ> for KeyEvent {
    fn from(_: CtrlJ) -> Self {
        KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL)
    }
}

ctrl_keys! {
    /// Ctrl+\ (FS)
    CtrlBackslash: 0x1C => KeyCode::Char('\\');
    /// Ctrl+] (GS)
    CtrlRightBracket: 0x1D => KeyCode::Char(']');
    /// Ctrl+^ (RS)
    CtrlCaret: 0x1E => KeyCode::Char('^');
    /// Ctrl+_ (US)
    CtrlUnderscore: 0x1F => KeyCode::Char('_');
}

key_event! {
    /// Alt+key
    #[vtansi(esc)]
    AltKey
}

impl<'a> TryFromAnsi<'a> for AltKey {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        if bytes.is_empty() {
            return Err(vtansi::ParseError::InvalidValue(
                "empty input".to_string(),
            ));
        }

        let (code, mods) = byte_to_key(bytes[0]).ok_or_else(|| {
            vtansi::ParseError::InvalidValue("unsupported key code".to_string())
        })?;

        Ok(Self::new(code, mods | KeyModifiers::ALT))
    }
}

impl AnsiEncode for AltKey {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, EncodeError> {
        // Strip ALT modifier since it's represented by the ESC prefix
        let mods_without_alt = self.modifiers - KeyModifiers::ALT;
        let byte = key_to_byte(self.code, mods_without_alt)
            .expect("AltKey should only contain keys representable as bytes");

        vtansi::write_byte_into(sink, byte)
    }
}

key_event!(ExtFnKey);

impl<'a> TryFromAnsi<'a> for ExtFnKey {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        let param = <u8 as TryFromAnsi>::try_from_ansi(bytes)?;

        let code = csi_tilde_code_to_key(param).ok_or_else(|| {
            vtansi::ParseError::InvalidValue(
                "unrecognized `CSI Ps ~` key parameter".to_string(),
            )
        })?;

        Ok(code.into())
    }
}

impl AnsiEncode for ExtFnKey {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, EncodeError> {
        let param = key_to_csi_tilde_code(self.code)
            .expect("ExtFnKey should only contain keys with CSI tilde codes");

        <u8 as AnsiEncode>::encode_ansi_into(&param, sink)
    }
}

/// Special keys using `CSI Ps ~` sequence (`Insert`, `Delete`, `PageUp`, `PageDown`, `Home`, `End`, `F5`-`F20`)
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
#[vtansi(csi, into = KeyEvent, finalbyte = '~')]
pub struct ExtFnKeySeq {
    pub key: ExtFnKey,
    pub modifiers: Option<CsiKeyModifiers>,
}

impl From<ExtFnKeySeq> for KeyEvent {
    fn from(key: ExtFnKeySeq) -> Self {
        Self::new(
            key.key.code,
            key.modifiers.map_or(KeyModifiers::empty(), Into::into),
        )
    }
}

// ============================================================================
// CSI u keyboard event parsing (Kitty keyboard protocol)
// ============================================================================

/// Convert a CSI u key code to a `KeyCode`.
///
/// This handles:
/// - Unicode codepoints (character keys)
/// - Functional key codes from the Private Use Area
/// - Special keys like Enter (13), Tab (9), Escape (27), Backspace (127)
fn csi_u_to_keycode(code: u32) -> Option<KeyCode> {
    // First check the functional key table
    if let Some(key) = csi_u_code_to_key(code) {
        return Some(key);
    }

    // Code 0 means "text event" with no key info
    if code == 0 {
        return None;
    }

    // Try to interpret as a Unicode codepoint
    char::from_u32(code).map(KeyCode::Char)
}

/// Parse associated text from colon-separated Unicode codepoints.
fn parse_csi_u_text(bytes: &[u8]) -> Option<String> {
    let codepoints: Vec<char> = parse_colon_separated(bytes)
        .filter_map(|opt| opt.and_then(char::from_u32))
        .collect();

    if codepoints.is_empty() {
        None
    } else {
        Some(codepoints.into_iter().collect())
    }
}

/// CSI u keyboard event from the Kitty keyboard protocol.
///
/// This parses sequences of the form:
/// ```text
/// CSI unicode-key-code:shifted-key:base-layout-key ; modifiers:event-type ; text u
/// ```
///
/// See: <https://sw.kovidgoyal.net/kitty/keyboard-protocol/#disambiguate-escape-codes>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsiUKeyEvent(pub KeyEvent);

impl<'a> TryFromAnsiIter<'a> for CsiUKeyEvent {
    fn try_from_ansi_iter<I>(params: &mut I) -> Result<Self, vtansi::ParseError>
    where
        I: Iterator<Item = &'a [u8]>,
    {
        let Some(key_part) = params.next() else {
            return Err(vtansi::ParseError::InvalidValue(
                "CSI u: missing key code parameter".to_string(),
            ));
        };

        // Parse first parameter: key-code:shifted-key:base-layout-key
        let mut key_parts = parse_colon_separated(key_part);
        let key_code_num = key_parts.next().flatten().ok_or_else(|| {
            vtansi::ParseError::InvalidValue(
                "CSI u: missing key code".to_string(),
            )
        })?;

        // Key code 0 means text-only event (composition)
        let mut key_code = if key_code_num == 0 {
            KeyCode::Char('\0')
        } else {
            csi_u_to_keycode(key_code_num).ok_or_else(|| {
                vtansi::ParseError::InvalidValue(format!(
                    "CSI u: invalid key code {key_code_num}"
                ))
            })?
        };

        // Optional shifted key (second sub-param)
        let shifted_key = key_parts.next().flatten().and_then(csi_u_to_keycode);

        // Optional base layout key (third sub-param)
        let base_layout_key =
            key_parts.next().flatten().and_then(csi_u_to_keycode);

        // Parse second parameter: modifiers:event-type
        let (mut modifiers, kind, mut state) = match params.next() {
            Some(v) => parse_csi_u_modifiers(v),
            None => {
                (KeyModifiers::NONE, KeyEventKind::Press, KeyEventState::NONE)
            }
        };

        // When a shifted key is provided and shift modifier is present,
        // use the shifted key as the actual key code and remove shift modifier.
        // This represents the actual character produced (e.g., Shift+a produces 'A').
        if let Some(shifted) = shifted_key
            && modifiers.contains(KeyModifiers::SHIFT)
        {
            key_code = shifted;
            modifiers = modifiers.difference(KeyModifiers::SHIFT);
        }

        // Convert Tab with Shift to BackTab
        if key_code == KeyCode::Tab && modifiers.contains(KeyModifiers::SHIFT) {
            key_code = KeyCode::BackTab;
            modifiers = modifiers.difference(KeyModifiers::SHIFT);
        }

        // Check if this is a keypad key
        if is_csi_u_keypad_code(key_code_num) {
            state |= KeyEventState::KEYPAD;
        }

        if let KeyCode::Modifier(modifier) = key_code {
            // When a modifier key itself is pressed without the modifier bit set,
            // add the corresponding modifier to the modifiers field
            modifiers |= modifier.into();
        }

        // Parse third parameter: alternate keycodes (currently unused/ignored)
        let _ = params.next();

        // Parse fourth parameter: text as codepoints
        let text = params.next().and_then(parse_csi_u_text);

        // Build the KeyEvent
        let mut builder = KeyEventBuilder::new(key_code, modifiers)
            .kind(kind)
            .state(state);

        if let Some(base_key) = base_layout_key {
            builder = builder.base_layout_key(base_key);
        }

        if let Some(txt) = text {
            builder = builder.text(txt);
        }

        Ok(CsiUKeyEvent(builder.build()))
    }
}

impl<'a> TryFromAnsi<'a> for CsiUKeyEvent {
    #[inline]
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        <CsiUKeyEvent as TryFromAnsiIter>::try_from_ansi_iter(
            &mut bytes.split(|&c| c == b';'),
        )
    }
}

impl From<CsiUKeyEvent> for KeyEvent {
    #[inline]
    fn from(csi_u: CsiUKeyEvent) -> Self {
        csi_u.0
    }
}

/// CSI u keyboard event input sequence.
#[derive(Debug, Clone, PartialEq, Eq, vtansi::derive::AnsiInput)]
#[vtansi(csi, into = KeyEvent, finalbyte = 'u')]
pub struct CsiUKeyEventSeq(#[vtansi(flatten)] pub CsiUKeyEvent);

impl From<CsiUKeyEventSeq> for KeyEvent {
    #[inline]
    fn from(seq: CsiUKeyEventSeq) -> Self {
        seq.0.into()
    }
}

impl AnsiEncode for CsiUKeyEvent {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, EncodeError> {
        // Encode the key event in CSI u format
        // This is a simplified encoding - full round-trip would need more work
        let event = &self.0;
        let code = match event.code {
            KeyCode::Char(c) => c as u32,
            KeyCode::Esc => 27,
            KeyCode::Enter => 13,
            KeyCode::Tab => 9,
            KeyCode::Backspace => 127,
            event_code => {
                // For other keys, try to find the CSI u code
                if let Some(code) = key_to_csi_u_code(event_code) {
                    code
                } else {
                    return Err(EncodeError::Unencodeable(format!(
                        "unsupported key code: {event_code:?}",
                    )));
                }
            }
        };

        let mut written = <u32 as AnsiEncode>::encode_ansi_into(&code, sink)?;

        let mod_param = event.modifiers.to_xterm_param();
        let event_type = match event.kind {
            KeyEventKind::Press => 1u8,
            KeyEventKind::Repeat => 2u8,
            KeyEventKind::Release => 3u8,
        };

        if mod_param != 0 || event_type != 1 {
            written += write_byte_into(sink, b';')?
                + <u8 as AnsiEncode>::encode_ansi_into(&mod_param, sink)?
                + if event_type == 1 {
                    0
                } else {
                    write_byte_into(sink, b':')?
                        + <u8 as AnsiEncode>::encode_ansi_into(
                            &event_type,
                            sink,
                        )?
                }
        }

        Ok(written)
    }
}

key_event!(FnKey);

impl AnsiEncode for FnKey {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, EncodeError> {
        vtansi::write_byte_into(sink, self.final_byte())
    }
}

impl FnKey {
    /// # Panics
    ///
    /// Panics if the `FnKey` contains a key code that doesn't have a CSI final byte.
    /// This should never happen if `FnKey` is constructed correctly.
    #[must_use]
    #[inline]
    pub fn final_byte(&self) -> u8 {
        key_to_csi_final_byte(self.code)
            .expect("FnKey should only contain keys with CSI final bytes")
    }
}

impl<'a> TryFromAnsi<'a> for FnKey {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        if bytes.is_empty() {
            return Err(vtansi::ParseError::InvalidValue(
                "empty input".to_string(),
            ));
        }

        let code = csi_final_byte_to_key(bytes[0]).ok_or_else(|| {
            vtansi::ParseError::InvalidValue("unrecognized key".to_string())
        })?;

        // BackTab has implicit SHIFT modifier
        let mods = if code == KeyCode::BackTab {
            KeyModifiers::SHIFT
        } else {
            KeyModifiers::NONE
        };

        Ok(Self::new(code, mods))
    }
}

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
#[vtansi(csi, into = KeyEvent, finalbyte = 'A' | 'B' | 'C' | 'D' | 'F' | 'H' | 'P' | 'Q' | 'R' | 'S' | 'Z')]
pub struct FnKeySeq {
    #[vtansi(locate = "final")]
    pub key: FnKey,
}

impl vtansi::AnsiFinalByte for FnKeySeq {
    #[inline]
    fn ansi_final_byte(&self) -> u8 {
        self.key.final_byte()
    }
}

impl From<FnKeySeq> for KeyEvent {
    fn from(key: FnKeySeq) -> Self {
        key.key.into()
    }
}

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
#[vtansi(csi, params = ["1"], into = KeyEvent, finalbyte = 'A' | 'B' | 'C' | 'D' | 'F' | 'H' | 'P' | 'Q' | 'R' | 'S' | 'Z')]
pub struct ModifiedFnKeySeq {
    pub modifiers: CsiKeyModifiers,
    #[vtansi(locate = "final")]
    pub key: FnKey,
}

impl vtansi::AnsiFinalByte for ModifiedFnKeySeq {
    #[inline]
    fn ansi_final_byte(&self) -> u8 {
        self.key.final_byte()
    }
}

impl From<ModifiedFnKeySeq> for KeyEvent {
    fn from(key: ModifiedFnKeySeq) -> Self {
        Self::new(key.key.code, key.modifiers.into())
    }
}

key_event! {
    /// SS3 keys (ESC O) for application cursor mode and keypad
    #[vtansi(ss3)]
    Ss3Key
}

impl Ss3Key {
    /// Returns the SS3 data byte for this key.
    ///
    /// # Panics
    ///
    /// Panics if the `Ss3Key` contains a key code that doesn't have an SS3 data byte.
    /// This should never happen if `Ss3Key` is constructed correctly through parsing.
    #[must_use]
    #[inline]
    pub fn data_byte(&self) -> u8 {
        key_to_ss3_byte(self.code)
            .expect("Ss3Key should only contain keys with SS3 data bytes")
    }

    /// Parse a `KeyCode` from an SS3 data byte.
    #[must_use]
    pub fn from_ss3_byte(byte: u8) -> Option<KeyCode> {
        ss3_byte_to_key(byte)
    }
}

impl<'a> TryFromAnsi<'a> for Ss3Key {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        if bytes.len() != 1 {
            return Err(vtansi::ParseError::InvalidValue(
                "SS3 key must be a single byte".to_string(),
            ));
        }

        Self::from_ss3_byte(bytes[0])
            .map(Into::into)
            .ok_or_else(|| {
                vtansi::ParseError::InvalidValue(format!(
                    "unrecognized SS3 key byte: 0x{:02x}",
                    bytes[0]
                ))
            })
    }
}

impl AnsiEncode for Ss3Key {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, EncodeError> {
        vtansi::write_byte_into(sink, self.data_byte())
    }
}

#[inline]
fn ascii_to_event<F>(byte: u8, cb: &mut F)
where
    F: FnMut(&dyn vtansi::AnsiEvent<'_>),
{
    // Note: Most C0 control codes (0x00-0x1F, 0x7F) are handled by the VT
    // parser and come through as VTEvent::C0. However, the VT parser
    // treats \t, \r, and \n as "whitespace control characters" and sends
    // them as Raw events, so we must handle them here.
    // We handle:
    // - Tab (0x09), LF (0x0A), CR (0x0D) - whitespace controls
    // - Printable ASCII (0x20-0x7E)
    // Other control codes are skipped since they come through other paths.
    let (code, modifiers) = match byte {
        b'\t' => (KeyCode::Tab, KeyModifiers::NONE),
        b'\r' => (KeyCode::Enter, KeyModifiers::NONE),
        b'\n' => (KeyCode::Char('\n'), KeyModifiers::NONE),
        0x20..=0x7E => {
            let ch = byte as char;
            let mods = if ch.is_ascii_uppercase() {
                KeyModifiers::SHIFT
            } else {
                KeyModifiers::NONE
            };
            (KeyCode::Char(ch), mods)
        }
        _ => return, // Skip other control codes
    };
    let key_event = KeyEvent::new(code, modifiers);
    cb(&key_event);
}

/// Returns the expected length of a UTF-8 character given its first byte.
///
/// Returns `None` if the byte is not a valid UTF-8 start byte (0x80-0xBF or 0xF8+).
#[inline]
const fn len_utf8(byte0: u8) -> Option<usize> {
    if byte0 & 0xE0 == 0xC0 {
        Some(2)
    } else if byte0 & 0xF0 == 0xE0 {
        Some(3)
    } else if byte0 & 0xF8 == 0xF0 {
        Some(4)
    } else {
        // Invalid start byte (continuation byte or overlong)
        None
    }
}

/// Check if all bytes are valid UTF-8 continuation bytes (10xxxxxx pattern).
#[inline]
fn are_valid_utf8_continuation_bytes(bytes: &[u8]) -> bool {
    bytes.iter().all(|&b| b & 0b1100_0000 == 0b1000_0000)
}

#[inline]
fn utf8_to_event<F>(bytes: &[u8], cb: &mut F) -> bool
where
    F: FnMut(&dyn vtansi::AnsiEvent<'_>),
{
    match std::str::from_utf8(bytes) {
        Ok(s) => {
            // SAFETY: we know there's at least one char
            let ch = unsafe { s.chars().next().unwrap_unchecked() };
            let modifiers = if ch.is_uppercase() {
                KeyModifiers::SHIFT
            } else {
                KeyModifiers::NONE
            };
            let key_event = KeyEvent::new(KeyCode::Char(ch), modifiers);
            cb(&key_event);
            true
        }
        Err(_) => false, // Invalid UTF-8 (overlong encoding, surrogates, etc.)
    }
}

/// Process raw bytes and emit Key events for valid UTF-8 characters.
/// Invalid UTF-8 sequences are silently skipped.  If the buffer ends
/// with a potentially incomplete UTF-8 sequence, returns the number
/// of bytes in the incomplete sequence.
#[inline]
pub fn bytes_to_events<F>(
    bytes: &[u8],
    utf8_buffer: &mut [u8],
    cb: &mut F,
) -> usize
where
    F: FnMut(&dyn vtansi::AnsiEvent<'_>),
{
    let mut pos = 0;
    while pos < bytes.len() {
        let byte = bytes[pos];

        // Fast path for ASCII (most common case)
        if byte < 0x80 {
            pos += 1;
            ascii_to_event(byte, cb);
            continue;
        }

        // Multi-byte UTF-8 character
        let Some(char_len) = len_utf8(byte) else {
            // Invalid start byte - skip it
            pos += 1;
            continue;
        };

        if pos + char_len > bytes.len() {
            // Incomplete sequence at end, validate and save it
            if are_valid_utf8_continuation_bytes(&bytes[pos + 1..]) {
                let remaining = &bytes[pos..];
                let len = remaining.len();
                utf8_buffer[..len].copy_from_slice(remaining);
                return len;
            }
            // Invalid continuation byte, skip
            pos += 1;
            continue;
        }

        // Process the multi-byte character
        if utf8_to_event(&bytes[pos..pos + char_len], cb) {
            pos += char_len;
        } else {
            // Invalid UTF-8 - skip the start byte
            pos += 1;
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use super::*;

    #[test]
    fn test_equality() {
        let lowercase_d_with_shift =
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::SHIFT);
        let uppercase_d_with_shift =
            KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT);
        let uppercase_d = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE);
        assert_eq!(lowercase_d_with_shift, uppercase_d_with_shift);
        assert_eq!(uppercase_d, uppercase_d_with_shift);
    }

    #[test]
    fn test_hash() {
        let lowercase_d_with_shift_hash = {
            let mut hasher = DefaultHasher::new();
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::SHIFT)
                .hash(&mut hasher);
            hasher.finish()
        };
        let uppercase_d_with_shift_hash = {
            let mut hasher = DefaultHasher::new();
            KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT)
                .hash(&mut hasher);
            hasher.finish()
        };
        let uppercase_d_hash = {
            let mut hasher = DefaultHasher::new();
            KeyEvent::new(KeyCode::Char('D'), KeyModifiers::NONE)
                .hash(&mut hasher);
            hasher.finish()
        };
        assert_eq!(lowercase_d_with_shift_hash, uppercase_d_with_shift_hash);
        assert_eq!(uppercase_d_hash, uppercase_d_with_shift_hash);
    }

    #[test]
    fn test_encode_key_event_char() {
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let mut buf = [0u8; 64];
        let len = event.encode_ansi_into(&mut &mut buf[..]).unwrap();
        assert_eq!(&buf[..len], b"a");
    }

    #[test]
    fn test_encode_key_event_ctrl_char() {
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let mut buf = [0u8; 64];
        let len = event.encode_ansi_into(&mut &mut buf[..]).unwrap();
        assert_eq!(&buf[..len], &[0x03]); // Ctrl-C
    }

    #[test]
    fn test_encode_key_event_enter() {
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let mut buf = [0u8; 64];
        let len = event.encode_ansi_into(&mut &mut buf[..]).unwrap();
        assert_eq!(&buf[..len], b"\r");
    }

    #[test]
    fn test_encode_key_event_arrow() {
        let event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let mut buf = [0u8; 64];
        let len = event.encode_ansi_into(&mut &mut buf[..]).unwrap();
        assert_eq!(&buf[..len], b"\x1b[A");
    }

    #[test]
    fn test_encode_key_event_arrow_with_modifiers() {
        let event = KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT);
        let mut buf = [0u8; 64];
        let len = event.encode_ansi_into(&mut &mut buf[..]).unwrap();
        assert_eq!(&buf[..len], b"\x1b[1;2A");
    }

    #[test]
    fn test_encode_key_event_f1() {
        let event = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        let mut buf = [0u8; 64];
        let len = event.encode_ansi_into(&mut &mut buf[..]).unwrap();
        assert_eq!(&buf[..len], b"\x1bOP");
    }

    #[test]
    fn test_base_layout_key() {
        // Test basic base layout key storage
        let event =
            KeyEvent::builder(KeyCode::Char('С'), KeyModifiers::CONTROL)
                .base_layout_key(KeyCode::Char('c'))
                .build();
        assert_eq!(event.base_layout_key(), Some(KeyCode::Char('c')));
    }

    #[test]
    fn test_matches_key_with_base_layout() {
        // Test that matches_key works with base layout key
        let event =
            KeyEvent::builder(KeyCode::Char('С'), KeyModifiers::CONTROL)
                .base_layout_key(KeyCode::Char('c'))
                .build();

        // Should match the actual key
        assert!(event.matches_key(KeyCode::Char('С')));

        // Should also match the base layout key
        assert!(event.matches_key(KeyCode::Char('c')));

        // Should not match other keys
        assert!(!event.matches_key(KeyCode::Char('d')));
    }

    #[test]
    fn test_matches_key_without_base_layout() {
        // Test that matches_key works without base layout key
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);

        // Should match the actual key
        assert!(event.matches_key(KeyCode::Char('a')));

        // Should not match other keys
        assert!(!event.matches_key(KeyCode::Char('b')));
    }

    #[test]
    fn test_base_layout_key_equality() {
        // Events with same base layout key should be equal
        let event1 =
            KeyEvent::builder(KeyCode::Char('С'), KeyModifiers::CONTROL)
                .base_layout_key(KeyCode::Char('c'))
                .build();
        let event2 =
            KeyEvent::builder(KeyCode::Char('С'), KeyModifiers::CONTROL)
                .base_layout_key(KeyCode::Char('c'))
                .build();
        assert_eq!(event1, event2);

        // Events with different base layout keys should not be equal
        let event3 =
            KeyEvent::builder(KeyCode::Char('С'), KeyModifiers::CONTROL)
                .base_layout_key(KeyCode::Char('d'))
                .build();
        assert_ne!(event1, event3);
    }

    #[test]
    fn test_base_layout_key_hash() {
        // Events with same base layout key should have same hash
        let hash1 = {
            let mut hasher = DefaultHasher::new();
            KeyEvent::builder(KeyCode::Char('С'), KeyModifiers::CONTROL)
                .base_layout_key(KeyCode::Char('c'))
                .build()
                .hash(&mut hasher);
            hasher.finish()
        };
        let hash2 = {
            let mut hasher = DefaultHasher::new();
            KeyEvent::builder(KeyCode::Char('С'), KeyModifiers::CONTROL)
                .base_layout_key(KeyCode::Char('c'))
                .build()
                .hash(&mut hasher);
            hasher.finish()
        };
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_encode_key_event_f5() {
        let event = KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE);
        let mut buf = [0u8; 64];
        let len = event.encode_ansi_into(&mut &mut buf[..]).unwrap();
        assert_eq!(&buf[..len], b"\x1b[15~");
    }

    #[test]
    fn test_ctrl_key_definitions() {
        // Test that Ctrl+key definitions produce correct KeyEvents with CONTROL modifier

        // Ctrl+A
        let ctrl_a: KeyEvent = CtrlA.into();
        assert_eq!(ctrl_a.code, KeyCode::Char('a'));
        assert_eq!(ctrl_a.modifiers, KeyModifiers::CONTROL);

        // Ctrl+C
        let ctrl_c: KeyEvent = CtrlC.into();
        assert_eq!(ctrl_c.code, KeyCode::Char('c'));
        assert_eq!(ctrl_c.modifiers, KeyModifiers::CONTROL);

        // Ctrl+Z
        let ctrl_z: KeyEvent = CtrlZ.into();
        assert_eq!(ctrl_z.code, KeyCode::Char('z'));
        assert_eq!(ctrl_z.modifiers, KeyModifiers::CONTROL);

        // Ctrl+Space
        let ctrl_space: KeyEvent = CtrlSpace.into();
        assert_eq!(ctrl_space.code, KeyCode::Char(' '));
        assert_eq!(ctrl_space.modifiers, KeyModifiers::CONTROL);

        // Ctrl+\
        let ctrl_backslash: KeyEvent = CtrlBackslash.into();
        assert_eq!(ctrl_backslash.code, KeyCode::Char('\\'));
        assert_eq!(ctrl_backslash.modifiers, KeyModifiers::CONTROL);

        // Ctrl+]
        let ctrl_right_bracket: KeyEvent = CtrlRightBracket.into();
        assert_eq!(ctrl_right_bracket.code, KeyCode::Char(']'));
        assert_eq!(ctrl_right_bracket.modifiers, KeyModifiers::CONTROL);

        // Ctrl+H (alias for Backspace) - should still convert to Ctrl+H
        let ctrl_h: KeyEvent = CtrlH.into();
        assert_eq!(ctrl_h.code, KeyCode::Char('h'));
        assert_eq!(ctrl_h.modifiers, KeyModifiers::CONTROL);

        // Ctrl+J (alias for EnterKey) - should still convert to Ctrl+J
        let ctrl_j: KeyEvent = CtrlJ.into();
        assert_eq!(ctrl_j.code, KeyCode::Char('j'));
        assert_eq!(ctrl_j.modifiers, KeyModifiers::CONTROL);
    }

    #[test]
    fn test_alias_types_encode() {
        // Test that alias types can be constructed and encode correctly
        use vtansi::encode::AnsiEncode;

        // CtrlH should encode to 0x08 (same as Backspace)
        let mut buf = [0u8; 1];
        let ctrl_h = CtrlH;
        let result = ctrl_h.encode_ansi_into(&mut buf.as_mut_slice());
        assert!(result.is_ok());
        assert_eq!(buf[0], 0x08);

        // CtrlJ should encode to 0x0A (same as EnterKey)
        let mut buf = [0u8; 1];
        let ctrl_j = CtrlJ;
        let result = ctrl_j.encode_ansi_into(&mut buf.as_mut_slice());
        assert!(result.is_ok());
        assert_eq!(buf[0], 0x0A);

        // EnterKey should also encode to 0x0A
        let mut buf = [0u8; 1];
        let enter = EnterKey;
        let result = enter.encode_ansi_into(&mut buf.as_mut_slice());
        assert!(result.is_ok());
        assert_eq!(buf[0], 0x0A);
    }

    // ========================================================================
    // CSI u keyboard event parsing tests (Kitty keyboard protocol)
    // ========================================================================

    #[test]
    fn test_csi_u_simple_char() {
        // CSI 97 u -> 'a' key
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"97").unwrap().into();
        assert_eq!(event.code, KeyCode::Char('a'));
        assert_eq!(event.modifiers, KeyModifiers::NONE);
        assert_eq!(event.kind, KeyEventKind::Press);
    }

    #[test]
    fn test_csi_u_char_with_modifiers() {
        // Ctrl+A (modifier 5 = 1 + 4, where 4 = ctrl)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"97;5").unwrap().into();
        assert_eq!(event.code, KeyCode::Char('a'));
        assert_eq!(event.modifiers, KeyModifiers::CONTROL);
    }

    #[test]
    fn test_csi_u_shift_alt_ctrl() {
        // Ctrl+Alt+Shift+A (modifier 8 = 1 + 7, where 7 = 1+2+4)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"97;8").unwrap().into();
        assert_eq!(event.code, KeyCode::Char('a'));
        assert!(event.modifiers.contains(KeyModifiers::SHIFT));
        assert!(event.modifiers.contains(KeyModifiers::ALT));
        assert!(event.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_csi_u_event_types() {
        // Press event (default)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"97;1:1").unwrap().into();
        assert_eq!(event.kind, KeyEventKind::Press);

        // Repeat event
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"97;1:2").unwrap().into();
        assert_eq!(event.kind, KeyEventKind::Repeat);

        // Release event
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"97;1:3").unwrap().into();
        assert_eq!(event.kind, KeyEventKind::Release);
    }

    #[test]
    fn test_csi_u_escape_key() {
        // Escape
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"27").unwrap().into();
        assert_eq!(event.code, KeyCode::Esc);
    }

    #[test]
    fn test_csi_u_enter_key() {
        // Enter
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"13").unwrap().into();
        assert_eq!(event.code, KeyCode::Enter);
    }

    #[test]
    fn test_csi_u_tab_key() {
        // Tab
        let event: KeyEvent = CsiUKeyEvent::try_from_ansi(b"9").unwrap().into();
        assert_eq!(event.code, KeyCode::Tab);
    }

    #[test]
    fn test_csi_u_backspace_key() {
        // Backspace
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"127").unwrap().into();
        assert_eq!(event.code, KeyCode::Backspace);
    }

    #[test]
    fn test_csi_u_function_keys() {
        // F13 (code 57376)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"57376").unwrap().into();
        assert_eq!(event.code, KeyCode::F(13));

        // F24 (code 57387)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"57387").unwrap().into();
        assert_eq!(event.code, KeyCode::F(24));
    }

    #[test]
    fn test_csi_u_media_keys() {
        // Media Play (code 57428)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"57428").unwrap().into();
        assert_eq!(event.code, KeyCode::Media(MediaKeyCode::Play));

        // Media Pause (code 57429)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"57429").unwrap().into();
        assert_eq!(event.code, KeyCode::Media(MediaKeyCode::Pause));
    }

    #[test]
    fn test_csi_u_modifier_keys() {
        // Left Shift (code 57441)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"57441").unwrap().into();
        assert_eq!(event.code, KeyCode::Modifier(ModifierKeyCode::LeftShift));

        // Right Control (code 57448)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"57448").unwrap().into();
        assert_eq!(
            event.code,
            KeyCode::Modifier(ModifierKeyCode::RightControl)
        );
    }

    #[test]
    fn test_csi_u_caps_lock_state() {
        // 'a' with caps lock (modifier 65 = 1 + 64)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"97;65").unwrap().into();
        assert_eq!(event.code, KeyCode::Char('a'));
        assert!(event.state.contains(KeyEventState::CAPS_LOCK));
    }

    #[test]
    fn test_csi_u_num_lock_state() {
        // 'a' with num lock (modifier 129 = 1 + 128)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"97;129").unwrap().into();
        assert_eq!(event.code, KeyCode::Char('a'));
        assert!(event.state.contains(KeyEventState::NUM_LOCK));
    }

    #[test]
    fn test_csi_u_keypad_state() {
        // Keypad 5 (code 57404)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"57404").unwrap().into();
        assert_eq!(event.code, KeyCode::Char('5'));
        assert!(event.state.contains(KeyEventState::KEYPAD));
    }

    #[test]
    fn test_csi_u_base_layout_key() {
        // Cyrillic 'С' with base layout 'c' (codepoint 1057)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"1057::99").unwrap().into();
        assert_eq!(event.code, KeyCode::Char('С'));
        assert_eq!(event.base_layout_key, Some(KeyCode::Char('c')));
    }

    #[test]
    fn test_csi_u_with_text() {
        // Shift+A with text 'A' (text parameter 65 in 4th position)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"97;2;;65").unwrap().into();
        assert_eq!(event.code, KeyCode::Char('a'));
        assert_eq!(event.modifiers, KeyModifiers::SHIFT);
        assert_eq!(event.text, Some("A".to_string()));
    }

    #[test]
    fn test_csi_u_unicode_char() {
        // 'é' (Unicode codepoint 233)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"233").unwrap().into();
        assert_eq!(event.code, KeyCode::Char('é'));
    }

    #[test]
    fn test_csi_u_full_sequence() {
        // Key 'a', shifted 'A', base layout 'c', Ctrl+Shift, press, text 'A'
        // When shifted key is provided with shift, the shifted key becomes the code
        // and shift is removed, but other modifiers like Ctrl remain
        let event: KeyEvent = CsiUKeyEvent::try_from_ansi(b"97:65:99;6:1;;65")
            .unwrap()
            .into();
        assert_eq!(event.code, KeyCode::Char('A'));
        assert_eq!(event.base_layout_key, Some(KeyCode::Char('c')));
        assert!(!event.modifiers.contains(KeyModifiers::SHIFT));
        assert!(event.modifiers.contains(KeyModifiers::CONTROL));
        assert_eq!(event.kind, KeyEventKind::Press);
        assert_eq!(event.text, Some("A".to_string()));
    }

    #[test]
    fn test_csi_u_lock_keys() {
        // CapsLock (code 57358)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"57358").unwrap().into();
        assert_eq!(event.code, KeyCode::CapsLock);

        // ScrollLock (code 57359)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"57359").unwrap().into();
        assert_eq!(event.code, KeyCode::ScrollLock);

        // NumLock (code 57360)
        let event: KeyEvent =
            CsiUKeyEvent::try_from_ansi(b"57360").unwrap().into();
        assert_eq!(event.code, KeyCode::NumLock);
    }

    // ========================================================================
    // Keyboard Mode Flags Tests
    // ========================================================================

    #[test]
    fn test_mode_flags_cursor_keys_mode() {
        use super::KeyboardModeFlags;

        let event = KeyEvent::from(KeyCode::Up);

        // Without CURSOR_KEYS mode: CSI [ A
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::empty(),
        );
        assert_eq!(enc, KeyEncoding::CsiFinal(b'A'));

        // With CURSOR_KEYS mode (DECCKM): SS3 A
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::CURSOR_KEYS,
        );
        assert_eq!(enc, KeyEncoding::Ss3(b'A'));
    }

    #[test]
    fn test_mode_flags_cursor_keys_with_modifiers() {
        use super::KeyboardModeFlags;

        // Cursor key with modifier should use CSI even in cursor keys mode
        let event = KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT);

        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::CURSOR_KEYS,
        );
        // Should be CSI 1;2 A (not SS3)
        assert_eq!(
            enc,
            KeyEncoding::CsiModFinal {
                mods: 2,
                final_byte: b'A'
            }
        );
    }

    #[test]
    fn test_mode_flags_backspace_sends_delete() {
        use super::KeyboardModeFlags;

        let event = KeyEvent::from(KeyCode::Backspace);

        // Default: Backspace sends DEL (0x7F)
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::empty(),
        );
        assert_eq!(enc, KeyEncoding::Raw(0x7f));

        // With BACKSPACE_SENDS_DELETE mode: Backspace sends BS (0x08)
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::BACKSPACE_SENDS_DELETE,
        );
        assert_eq!(enc, KeyEncoding::Raw(0x08));
    }

    #[test]
    fn test_mode_flags_delete_key_sends_del() {
        use super::KeyboardModeFlags;

        let event = KeyEvent::from(KeyCode::Delete);

        // Default: Delete sends CSI 3 ~
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::empty(),
        );
        assert_eq!(enc, KeyEncoding::CsiTilde(3));

        // With DELETE_KEY_SENDS_DEL mode: Delete sends DEL (0x7F)
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::DELETE_KEY_SENDS_DEL,
        );
        assert_eq!(enc, KeyEncoding::Raw(0x7f));
    }

    #[test]
    fn test_mode_flags_alt_high_bit_set() {
        use super::KeyboardModeFlags;

        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT);

        // Default: Alt+a sends ESC a
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::empty(),
        );
        assert_eq!(
            enc,
            KeyEncoding::Char {
                alt_prefix: true,
                ch: 'a'
            }
        );

        // With ALT_KEY_HIGH_BIT_SET mode: Alt+a sends 0xE1 (0x61 | 0x80)
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::ALT_KEY_HIGH_BIT_SET,
        );
        assert_eq!(enc, KeyEncoding::Raw(0x61 | 0x80));
    }

    #[test]
    fn test_mode_flags_alt_ctrl_high_bit_set() {
        use super::KeyboardModeFlags;

        let event = KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::ALT | KeyModifiers::CONTROL,
        );

        // Default: Alt+Ctrl+a sends ESC followed by Ctrl-A (0x01)
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::empty(),
        );
        assert_eq!(
            enc,
            KeyEncoding::Char {
                alt_prefix: true,
                ch: '\x01'
            }
        );

        // With ALT_KEY_HIGH_BIT_SET mode: Alt+Ctrl+a sends 0x81 (0x01 | 0x80)
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::ALT_KEY_HIGH_BIT_SET,
        );
        assert_eq!(enc, KeyEncoding::Raw(0x01 | 0x80));
    }

    #[test]
    fn test_mode_flags_application_keypad() {
        use super::KeyboardModeFlags;

        // Note: At the KeyCode level, we cannot distinguish between main keyboard
        // and keypad keys - they both appear as KeyCode::Char('5'). The APPLICATION_KEYPAD
        // mode flag is available for use by higher-level code that can track key origin.
        //
        // This test verifies that the SS3 mapping exists for keypad characters,
        // even though the current implementation doesn't automatically switch
        // based on the mode flag alone (since we can't tell keypad from main keyboard).
        let event = KeyEvent::from(KeyCode::Char('5'));

        // Character '5' always encodes as the character itself
        // (we can't distinguish keypad vs main keyboard at KeyCode level)
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::empty(),
        );
        assert_eq!(
            enc,
            KeyEncoding::Char {
                alt_prefix: false,
                ch: '5'
            }
        );

        // Even with APPLICATION_KEYPAD, regular Char still encodes as char
        // because we can't tell it's from the keypad
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::APPLICATION_KEYPAD,
        );
        assert_eq!(
            enc,
            KeyEncoding::Char {
                alt_prefix: false,
                ch: '5'
            }
        );

        // Verify that the SS3 mapping exists for '5' -> 'u'
        // (used when parsing SS3 sequences from terminal)
        assert_eq!(ss3_byte_to_key(b'u'), Some(KeyCode::Char('5')));
        assert_eq!(key_to_ss3_byte(KeyCode::Char('5')), Some(b'u'));
    }

    #[test]
    fn test_mode_flags_home_end_cursor_mode() {
        use super::KeyboardModeFlags;

        let event = KeyEvent::from(KeyCode::Home);

        // Default: Home sends CSI H
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::empty(),
        );
        assert_eq!(enc, KeyEncoding::CsiFinal(b'H'));

        // With CURSOR_KEYS mode: Home sends SS3 H
        let enc = KeyEncoding::from_key_event_with_modes(
            &event,
            KeyboardModeFlags::CURSOR_KEYS,
        );
        assert_eq!(enc, KeyEncoding::Ss3(b'H'));
    }

    #[test]
    fn test_get_key_event_encoding_with_mode_flags() {
        use super::{KeyboardEnhancementFlags, KeyboardModeFlags};
        use vtansi::AnsiEncode;

        let event = KeyEvent::from(KeyCode::Up);

        // Legacy mode without CURSOR_KEYS: CSI [ A
        let mut buf = Vec::new();
        get_key_event_encoding(
            &event,
            KeyboardEnhancementFlags::empty(),
            KeyboardModeFlags::empty(),
        )
        .encode_ansi_into(&mut buf)
        .unwrap();
        assert_eq!(buf, b"\x1b[A");

        // Legacy mode with CURSOR_KEYS: SS3 A
        let mut buf = Vec::new();
        get_key_event_encoding(
            &event,
            KeyboardEnhancementFlags::empty(),
            KeyboardModeFlags::CURSOR_KEYS,
        )
        .encode_ansi_into(&mut buf)
        .unwrap();
        assert_eq!(buf, b"\x1bOA");

        // CSI-u mode ignores CURSOR_KEYS flag
        let mut buf = Vec::new();
        get_key_event_encoding(
            &event,
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES,
            KeyboardModeFlags::CURSOR_KEYS,
        )
        .encode_ansi_into(&mut buf)
        .unwrap();
        // CSI-u format for Up arrow
        assert!(buf.starts_with(b"\x1b["));
        assert!(buf.ends_with(b"u"));
    }
}
