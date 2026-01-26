//! Core mouse event types.

use std::fmt::{self, Write};

use vtansi::{ParseError, TerseDisplay};

use crate::event::common::Coords;
use crate::event::keyboard::KeyModifiers;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[derive(
    Debug, Default, PartialEq, Eq, Clone, Copy, Hash, vtansi::derive::ToAnsi,
)]
#[vtansi(transparent)]
pub struct MouseKeyModifiers(pub(crate) KeyModifiers);

/// Extract modifier keys from a mouse button code.
///
/// The button code contains modifier bits:
/// - bit 2 (4): Shift
/// - bit 3 (8): Alt
/// - bit 4 (16): Ctrl
#[must_use]
pub const fn modifiers_from_button_code(btn_code: u16) -> KeyModifiers {
    let mut bits = KeyModifiers::NONE.bits();
    if (btn_code & 4) != 0 {
        bits |= KeyModifiers::SHIFT.bits();
    }
    if (btn_code & 8) != 0 {
        bits |= KeyModifiers::ALT.bits();
    }
    if (btn_code & 16) != 0 {
        bits |= KeyModifiers::CONTROL.bits();
    }
    KeyModifiers::from_bits_retain(bits)
}

impl ::std::ops::Deref for MouseKeyModifiers {
    type Target = KeyModifiers;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<KeyModifiers> for MouseKeyModifiers {
    fn from(modifiers: KeyModifiers) -> Self {
        Self(modifiers)
    }
}

impl vtansi::AnsiMuxEncode for MouseKeyModifiers {
    type BaseType = u16;

    fn mux_encode(
        &self,
        base: Option<&Self::BaseType>,
    ) -> Result<Self::BaseType, vtansi::EncodeError> {
        Ok((if let Some(base) = base { *base } else { 0 })
            | u16::from(self.bits()))
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for MouseKeyModifiers {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        let code = <u16 as vtansi::TryFromAnsi>::try_from_ansi(bytes)?;
        Ok(Self(modifiers_from_button_code(code)))
    }
}

/// Represents a mouse event.
///
/// This is the core mouse event structure that is encoding-agnostic.
/// Different terminal mouse reporting formats (SGR, urxvt, default/multibyte)
/// all parse into this common structure.
///
/// # Platform-specific Notes
///
/// ## Mouse Buttons
///
/// Some platforms/terminals do not report mouse button for the
/// `MouseEventKind::Up` and `MouseEventKind::Drag` events.
/// `MouseButton::Left` is returned if we don't know which button was
/// used.
///
/// ## Key Modifiers
///
/// Some platforms/terminals does not report all key modifiers
/// combinations for all mouse event types. For example - macOS reports
/// `Ctrl` + left mouse button click as a right mouse button click.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct MouseEvent {
    /// The kind of mouse event that was caused.
    pub kind: MouseEventKind,
    /// The key modifiers active when the event occurred.
    pub modifiers: MouseKeyModifiers,
    /// The coordinates where the event occurred.
    pub coords: Coords,
}

impl MouseEvent {
    /// Create a new mouse event.
    #[must_use]
    pub const fn new(
        kind: MouseEventKind,
        modifiers: MouseKeyModifiers,
        coords: Coords,
    ) -> Self {
        Self {
            kind,
            modifiers,
            coords,
        }
    }

    /// Get the column where the event occurred (0-based).
    #[must_use]
    pub const fn col(&self) -> u16 {
        self.coords.col.saturating_sub(1)
    }

    /// Get the row where the event occurred (0-based).
    #[must_use]
    pub const fn row(&self) -> u16 {
        self.coords.row.saturating_sub(1)
    }
}

better_any::tid! { MouseEvent }

impl TerseDisplay for MouseEvent {
    fn terse_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("mouse(")?;

        // Event kind (down, up, drag, moved, scroll)
        match self.kind {
            MouseEventKind::Down(btn) => {
                f.write_str("down:")?;
                write_button(f, btn)?;
            }
            MouseEventKind::Up(btn) => {
                f.write_str("up:")?;
                write_button(f, btn)?;
            }
            MouseEventKind::Drag(btn) => {
                f.write_str("drag:")?;
                write_button(f, btn)?;
            }
            MouseEventKind::Moved => f.write_str("moved")?,
            MouseEventKind::ScrollUp => f.write_str("scroll:up")?,
            MouseEventKind::ScrollDown => f.write_str("scroll:down")?,
            MouseEventKind::ScrollLeft => f.write_str("scroll:left")?,
            MouseEventKind::ScrollRight => f.write_str("scroll:right")?,
        }

        // Modifiers (if any)
        if !self.modifiers.is_empty() {
            f.write_char(':')?;
            let mut first = true;
            if self.modifiers.contains(KeyModifiers::CONTROL) {
                f.write_str("ctrl")?;
                first = false;
            }
            if self.modifiers.contains(KeyModifiers::ALT) {
                if !first {
                    f.write_char('-')?;
                }
                f.write_str("alt")?;
                first = false;
            }
            if self.modifiers.contains(KeyModifiers::SHIFT) {
                if !first {
                    f.write_char('-')?;
                }
                f.write_str("shift")?;
            }
        }

        // Coordinates
        write!(f, "@{},{}", self.coords.col, self.coords.row)?;

        f.write_char(')')
    }
}

fn write_button(f: &mut fmt::Formatter<'_>, btn: MouseButton) -> fmt::Result {
    match btn {
        MouseButton::Left => f.write_str("left"),
        MouseButton::Right => f.write_str("right"),
        MouseButton::Middle => f.write_str("middle"),
        MouseButton::Nth(n) => write!(f, "btn{n}"),
    }
}

impl vtansi::AnsiEvent<'_> for MouseEvent {
    #[inline]
    fn ansi_control_kind(&self) -> Option<vtansi::AnsiControlFunctionKind> {
        Some(vtansi::AnsiControlFunctionKind::Csi)
    }

    #[inline]
    fn ansi_direction(&self) -> vtansi::AnsiControlDirection {
        vtansi::AnsiControlDirection::Input
    }

    vtansi::impl_ansi_event_encode!();
    vtansi::impl_ansi_event_terse_fmt!();
}

/// A mouse event kind.
///
/// # Platform-specific Notes
///
/// ## Mouse Buttons
///
/// Some platforms/terminals do not report mouse button for the
/// `MouseEventKind::Up` and `MouseEventKind::Drag` events.
/// `MouseButton::Left` is returned if we don't know which button was
/// used.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MouseEventKind {
    /// Pressed mouse button. Contains the button that was pressed.
    Down(MouseButton),
    /// Released mouse button. Contains the button that was released.
    Up(MouseButton),
    /// Moved the mouse cursor while pressing the contained mouse button.
    Drag(MouseButton),
    /// Moved the mouse cursor while not pressing a mouse button.
    Moved,
    /// Scrolled mouse wheel downwards (towards the user).
    ScrollDown,
    /// Scrolled mouse wheel upwards (away from the user).
    ScrollUp,
    /// Scrolled mouse wheel left (mostly on a laptop touchpad).
    ScrollLeft,
    /// Scrolled mouse wheel right (mostly on a laptop touchpad).
    ScrollRight,
}

impl MouseEventKind {
    /// Parse a mouse event kind from a button code.
    ///
    /// The `is_release` parameter indicates whether this is a button release
    /// event (used in SGR format where release is indicated by final byte 'm').
    /// In the default format, release is indicated by button code 3.
    ///
    /// # Errors
    ///
    /// Returns an error if the button code is invalid or unrecognized.
    pub fn from_button_code(
        btn_code: u16,
        is_release: bool,
    ) -> Result<Self, ParseError> {
        // Remove modifier bits (4, 8, 16)
        let base_code = btn_code & !0x1C;
        let is_drag = (btn_code & 32) != 0;

        let event_kind = if base_code >= 64 {
            // Scroll events (bit 6 set)
            match base_code & 0x03 {
                0 => MouseEventKind::ScrollUp,
                1 => MouseEventKind::ScrollDown,
                2 => MouseEventKind::ScrollLeft,
                3 => MouseEventKind::ScrollRight,
                code => {
                    return Err(vtansi::ParseError::InvalidValue(format!(
                        "unrecognized mouse button code: {code}"
                    )));
                }
            }
        } else if (base_code & !32) == 3 {
            // Button code 3: "moved" if drag bit set, otherwise release
            if is_drag {
                MouseEventKind::Moved
            } else {
                // In default format, we don't know which button was released
                MouseEventKind::Up(MouseButton::Left)
            }
        } else {
            let button = match base_code & 0x03 {
                0 => MouseButton::Left,
                1 => MouseButton::Middle,
                2 => MouseButton::Right,
                code => {
                    return Err(vtansi::ParseError::InvalidValue(format!(
                        "unrecognized mouse button code: {code}"
                    )));
                }
            };
            if is_release {
                MouseEventKind::Up(button)
            } else if is_drag {
                MouseEventKind::Drag(button)
            } else {
                MouseEventKind::Down(button)
            }
        };

        Ok(event_kind)
    }
}

/// Convert a mouse button to its protocol button code.
///
/// Protocol button codes:
/// - 0: left
/// - 1: middle
/// - 2: right
#[inline]
fn button_to_protocol_code(button: MouseButton) -> u16 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
        MouseButton::Nth(n) => n,
    }
}

/// Convert the base button code (without modifiers) into u16.
impl From<&MouseEventKind> for u16 {
    #[inline]
    fn from(value: &MouseEventKind) -> Self {
        match value {
            MouseEventKind::Down(button) | MouseEventKind::Up(button) => {
                button_to_protocol_code(*button)
            }
            MouseEventKind::Drag(button) => {
                32u16 + button_to_protocol_code(*button)
            } // Add drag bit
            MouseEventKind::Moved => 3 + 32, // Mouse move without button
            MouseEventKind::ScrollUp => 1 << 6,
            MouseEventKind::ScrollDown => (1 << 6) | 1,
            MouseEventKind::ScrollLeft => (1 << 6) | 2,
            MouseEventKind::ScrollRight => (1 << 6) | 3,
        }
    }
}

impl From<MouseEventKind> for u16 {
    #[inline]
    fn from(value: MouseEventKind) -> Self {
        u16::from(&value)
    }
}

impl vtansi::AnsiMuxEncode for MouseEventKind {
    type BaseType = u16;

    #[inline]
    fn mux_encode(
        &self,
        base: Option<&Self::BaseType>,
    ) -> Result<Self::BaseType, vtansi::EncodeError> {
        let other = if let Some(base) = base { *base } else { 0 };
        Ok(Self::BaseType::from(self) | other)
    }
}

impl vtansi::AnsiEncode for MouseEventKind {
    const ENCODED_LEN: Option<usize> = <u16 as vtansi::AnsiEncode>::ENCODED_LEN;

    #[inline]
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        <_ as vtansi::AnsiEncode>::encode_ansi_into(&u16::from(self), sink)
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for MouseEventKind {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        let code = <u16 as vtansi::TryFromAnsi>::try_from_ansi(bytes)?;
        // SGR format: release is indicated by final byte 'm', not button code
        // So we pass is_release=false here; the final byte handling is done elsewhere
        Self::from_button_code(code, false)
    }
}

/// Represents a mouse button.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
)]
#[repr(u16)]
pub enum MouseButton {
    /// Left mouse button.
    Left = 0,
    /// Right mouse button.
    Right = 1,
    /// Middle mouse button.
    Middle = 2,
    /// Nth mouse button.
    #[num_enum(catch_all)]
    Nth(u16),
}
