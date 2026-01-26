//! Keyboard-related messages.

mod encoding;
mod event;
mod keycode;
mod mode;
mod modifier;
mod util;

pub use encoding::{bytes_to_events, get_key_event_encoding};
pub use event::{KeyEvent, KeyEventBuilder};
pub use keycode::{KeyCode, MediaKeyCode};
pub use mode::{
    DisableXTermKeyModifierOptions, KeyboardEnhancementFlags,
    KeyboardEnhancementFlagsQuery, KeyboardEnhancementFlagsResponse,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    QueryXTermKeyFormatOptions, ResetApplicationKeypadMode,
    SetApplicationKeypadMode, SetKeyboardEnhancementFlags,
    SetXTermKeyFormatOptions, SetXTermKeyModifierOptions, XTermKeyFormatOption,
    XTermKeyFormatOptionsResponse, XTermModifierKeyResource,
};
pub use modifier::{
    KeyEventKind, KeyEventState, KeyModifiers, ModifierKeyCode,
};

use vtansi::bitflags;

use crate::terminal_mode;

bitflags! {
    /// Bitmask flags for keyboard-related terminal modes.
    ///
    /// These flags represent the state of various keyboard modes that can be
    /// queried from the terminal. Use [`collect_mode_flags`] to combine
    /// multiple mode query responses into a single bitmask.
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
    #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, Default)]
    pub struct KeyboardModeFlags: u64 {
        /// Cursor keys mode (`DECCKM`) is set.
        const CURSOR_KEYS = 1 << 0;
        /// Held keys repeat mode is set.
        const HELD_KEYS_REPEAT = 1 << 1;
        /// Application keypad mode (`DECNKM`) is set.
        const APPLICATION_KEYPAD = 1 << 2;
        /// Backspace sends delete mode (`DECBKM`) is set.
        const BACKSPACE_SENDS_DELETE = 1 << 3;
        /// Alt key high bit set mode is set.
        const ALT_KEY_HIGH_BIT_SET = 1 << 4;
        /// Ignore keypad application mode on numlock is set.
        const IGNORE_KEYPAD_APP_MODE_ON_NUMLOCK = 1 << 5;
        /// Alt key sends ESC prefix mode is set.
        const ALT_KEY_SENDS_ESC_PREFIX = 1 << 6;
        /// Delete key sends DEL mode is set.
        const DELETE_KEY_SENDS_DEL = 1 << 7;
        /// Additional modifier key sends ESC prefix mode is set.
        const ADDITIONAL_MODIFIER_KEY_SENDS_ESC_PREFIX = 1 << 8;
    }
}

/// Trait for terminal modes that can be represented as a bitmask flag.
///
/// Types implementing this trait can be combined into a [`KeyboardModeFlags`]
/// bitmask using [`collect_mode_flags`].
pub trait AsModeFlag {
    /// Returns the mode flag if the mode is set, or empty flags if not set.
    ///
    /// The returned value should be a single flag when the mode is active,
    /// allowing multiple modes to be OR-ed together.
    fn as_mode_flag(&self) -> KeyboardModeFlags;
}

/// Collect mode flags from an iterable of items implementing [`AsModeFlag`].
///
/// This function iterates over the provided items and combines their flag
/// bits using bitwise OR, producing a [`KeyboardModeFlags`] bitmask
/// representing all active modes.
///
/// # Example
///
/// ```ignore
/// use vtio::event::keyboard::{collect_mode_flags, AsModeFlag, KeyboardModeFlags};
///
/// let modes = [mode1, mode2, mode3];
/// let flags: KeyboardModeFlags = collect_mode_flags(&modes);
/// ```
pub fn collect_mode_flags<'a, I, T>(iter: I) -> KeyboardModeFlags
where
    I: IntoIterator<Item = &'a T>,
    T: AsModeFlag + 'a,
{
    iter.into_iter()
        .fold(KeyboardModeFlags::empty(), |acc, item| {
            acc | item.as_mode_flag()
        })
}

terminal_mode!(
    /// Disable Keyboard Input (`KAM`).
    ///
    /// When this mode is active it disables all keyboard input.
    ///
    /// See <https://terminalguide.namepad.de/mode/2/> for
    /// terminal support specifics.
    KeyboardInputDisabledMode, params = ["2"]
);

terminal_mode!(
    /// Cursor Key Format (`DECCKM`).
    ///
    /// Switches reporting format for cursor type keys.
    ///
    /// This changes the reported sequence for:
    /// - up, down, right, left
    /// - numpad up, down, right, left (for terminals that use ESC [ A etc
    ///   sequences for these keys)
    /// - home, end (for terminals that use ESC [ H etc sequences for these
    ///   keys)
    ///
    /// With the mode active these report with sequences beginning with ESC O.
    /// With the mode reset these report with sequences beginning with ESC [.
    ///
    /// See <https://terminalguide.namepad.de/mode/p1/> for
    /// terminal support specifics.
    CursorKeysMode, private = '?', params = ["1"], flag = KeyboardModeFlags::CURSOR_KEYS
);

terminal_mode!(
    /// Repeat Held Keys.
    ///
    /// Repeat keys while being held down. If enabled a key held down
    /// automatically repeats in an implementation specific interval.
    ///
    /// See <https://terminalguide.namepad.de/mode/p8/> for
    /// terminal support specifics.
    HeldKeysRepeatMode, private = '?', params = ["8"], flag = KeyboardModeFlags::HELD_KEYS_REPEAT
);

terminal_mode!(
    /// Application Keypad Mode (`DECNKM`).
    ///
    /// This is a mirror of [`SetApplicationKeypadMode`].
    ///
    /// See <https://terminalguide.namepad.de/mode/p66/> for
    /// terminal support specifics.
    ApplicationKeypadMode, private = '?', params = ["66"], flag = KeyboardModeFlags::APPLICATION_KEYPAD
);

terminal_mode!(
    /// Backspace Sends Delete (`DECBKM`).
    ///
    /// When set the backspace key sends BS, when reset it sends DEL.
    ///
    /// See <https://terminalguide.namepad.de/mode/p67/> for
    /// terminal support specifics.
    BackspaceSendsDeleteMode, private = '?', params = ["67"], flag = KeyboardModeFlags::BACKSPACE_SENDS_DELETE
);

terminal_mode!(
    /// Alt + Key Sends Character with High Bit Set.
    ///
    /// Use high bit to signal keypresses with alt modifier held.
    ///
    /// When reporting key presses use the high (eighth) bit to indicate
    /// alt modifier. (This will collide with non ASCII characters)
    ///
    /// See <https://terminalguide.namepad.de/mode/p1034/> for
    /// terminal support specifics.
    AltKeyHighBitSetMode, private = '?', params = ["1034"], flag = KeyboardModeFlags::ALT_KEY_HIGH_BIT_SET
);

terminal_mode!(
    /// Ignore Keypad Application Mode When Numlock is Active.
    ///
    /// If application keypad mode is active:
    /// - The mathematical operations keys (/, *, -, +) are sent to the
    ///   application as escape sequence regardless of num lock state if this
    ///   mode is off. If this mode is on, the keys send their printable
    ///   character when num lock is active.
    /// - With num lock active, the number / edit keys send escape sequences
    ///   if this mode is off if shift is not pressed. When this mode is on
    ///   they send the printable character for their number. (In both
    ///   settings the keys send the same escape sequences when shift is held)
    /// - With num lock active, the enter key on the num pad sends escape
    ///   sequences if this mode is off. When this mode is on it sends CR.
    ///
    /// See <https://terminalguide.namepad.de/mode/p1035/> for
    /// terminal support specifics.
    IgnoreKeypadApplicationModeOnNumlockMode, private = '?', params = ["1035"], flag = KeyboardModeFlags::IGNORE_KEYPAD_APP_MODE_ON_NUMLOCK
);

terminal_mode!(
    /// Alt + Key Sends Esc as Prefix.
    ///
    /// When this mode is active Alt + Key sends ESC + the key for
    /// printable inputs instead of forcing the 8th bit of the character
    /// to high.
    ///
    /// See <https://terminalguide.namepad.de/mode/p1036/> for
    /// terminal support specifics.
    AltKeySendsEscPrefixMode, private = '?', params = ["1036"], flag = KeyboardModeFlags::ALT_KEY_SENDS_ESC_PREFIX
);

terminal_mode!(
    /// Delete Key sends DEL.
    ///
    /// If set use legacy DEL for the delete key instead of an
    /// escape sequence.
    ///
    /// See <https://terminalguide.namepad.de/mode/p1037/> for
    /// terminal support specifics.
    DeleteKeySendsDELMode, private = '?', params = ["1037"], flag = KeyboardModeFlags::DELETE_KEY_SENDS_DEL
);

terminal_mode!(
    /// Additional Modifier + Key Sends Esc as Prefix.
    ///
    /// This is similar to [`AltKeySendsEscPrefixMode`], but for
    /// an additionally configured modifier.
    ///
    /// See <https://terminalguide.namepad.de/mode/p1039/> for
    /// terminal support specifics.
    AdditionalModifierKeySendsEscPrefix, private = '?', params = ["1039"], flag = KeyboardModeFlags::ADDITIONAL_MODIFIER_KEY_SENDS_ESC_PREFIX
);
