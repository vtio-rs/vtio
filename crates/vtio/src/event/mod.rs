//! Terminal event sequences.

pub mod charset;
pub mod color;
pub mod cursor;
pub mod dsr;
pub mod iterm;
pub mod keyboard;
pub mod mode;
pub mod mouse;
pub mod screen;
pub mod scroll;
pub mod sgr;
pub mod shell;
pub mod terminal;
pub mod text;
pub mod window;

// Re-export module-level input event enums
pub use mouse::MouseEvent;
pub use text::PlainText;

// Re-export commonly used types
pub use keyboard::{
    KeyCode, KeyEvent, KeyModifiers, KeyboardEnhancementFlags,
    KeyboardEnhancementFlagsQuery, KeyboardEnhancementFlagsResponse,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};

use vt_push_parser::event::VTEvent;

/// Unparsed or unrecognized terminal event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnrecognizedInputEvent<'a>(pub &'a VTEvent<'a>);

better_any::tid! {UnrecognizedInputEvent<'a>}

impl vtansi::TerseDisplay for UnrecognizedInputEvent<'_> {
    fn terse_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl vtansi::AnsiEncode for UnrecognizedInputEvent<'_> {
    #[inline]
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        self.0.write_to(sink).map_err(vtansi::EncodeError::IOError)
    }
}

impl<'a> vtansi::AnsiEvent<'a> for UnrecognizedInputEvent<'a> {
    fn ansi_control_kind(&self) -> Option<vtansi::AnsiControlFunctionKind> {
        match self.0 {
            VTEvent::Raw(_) => None,
            VTEvent::C0(_) => Some(vtansi::AnsiControlFunctionKind::Byte),
            VTEvent::Csi(_) => Some(vtansi::AnsiControlFunctionKind::Csi),
            VTEvent::Ss2(_)
            | VTEvent::Ss3(_)
            | VTEvent::Esc(_)
            | VTEvent::EscInvalid(_) => {
                Some(vtansi::AnsiControlFunctionKind::Esc)
            }

            VTEvent::DcsStart(_)
            | VTEvent::DcsData(_)
            | VTEvent::DcsEnd(_)
            | VTEvent::DcsCancel => Some(vtansi::AnsiControlFunctionKind::Dcs),

            VTEvent::OscStart
            | VTEvent::OscData(_)
            | VTEvent::OscEnd { .. }
            | VTEvent::OscCancel => Some(vtansi::AnsiControlFunctionKind::Osc),
        }
    }

    fn ansi_direction(&self) -> vtansi::AnsiControlDirection {
        vtansi::AnsiControlDirection::Input
    }

    vtansi::impl_ansi_event_encode!();
    vtansi::impl_ansi_event_terse_fmt!();
}

/// Unrecognized output event wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnrecognizedOutputEvent<'a>(pub &'a VTEvent<'a>);

better_any::tid! {UnrecognizedOutputEvent<'a>}

impl vtansi::TerseDisplay for UnrecognizedOutputEvent<'_> {
    fn terse_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl vtansi::AnsiEncode for UnrecognizedOutputEvent<'_> {
    #[inline]
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        self.0.write_to(sink).map_err(vtansi::EncodeError::IOError)
    }
}

impl<'a> vtansi::AnsiEvent<'a> for UnrecognizedOutputEvent<'a> {
    fn ansi_control_kind(&self) -> Option<vtansi::AnsiControlFunctionKind> {
        match self.0 {
            VTEvent::Raw(_) => None,
            VTEvent::C0(_) => Some(vtansi::AnsiControlFunctionKind::Byte),
            VTEvent::Csi(_) => Some(vtansi::AnsiControlFunctionKind::Csi),
            VTEvent::Ss2(_)
            | VTEvent::Ss3(_)
            | VTEvent::Esc(_)
            | VTEvent::EscInvalid(_) => {
                Some(vtansi::AnsiControlFunctionKind::Esc)
            }

            VTEvent::DcsStart(_)
            | VTEvent::DcsData(_)
            | VTEvent::DcsEnd(_)
            | VTEvent::DcsCancel => Some(vtansi::AnsiControlFunctionKind::Dcs),

            VTEvent::OscStart
            | VTEvent::OscData(_)
            | VTEvent::OscEnd { .. }
            | VTEvent::OscCancel => Some(vtansi::AnsiControlFunctionKind::Osc),
        }
    }

    fn ansi_direction(&self) -> vtansi::AnsiControlDirection {
        vtansi::AnsiControlDirection::Output
    }

    vtansi::impl_ansi_event_encode!();
    vtansi::impl_ansi_event_terse_fmt!();
}
