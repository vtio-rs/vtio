//! Terminal event sequences.
//!
//! # Sequence Reference
//!
//! ## clipboard
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `OSC 52 ; Pc ; Pd ST` (set) | [`clipboard::Clipboard`] with [`clipboard::ClipboardAction::Set`] |
//! | `OSC 52 ; Pc ; ? ST` (query) | [`clipboard::Clipboard`] with [`clipboard::ClipboardAction::Query`] |
//! | `OSC 52 ; Pc ; Pd ST` (response) | [`clipboard::ClipboardResponse`] |
//!
//! ## charset
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `ESC % G` | [`charset::EnableUTF8Mode`] |
//! | `ESC % @` | [`charset::DisableUTF8Mode`] |
//! | `0x0E` (C0 control code) | [`charset::ShiftOut`] |
//! | `0x0F` (C0 control code) | [`charset::ShiftIn`] |
//! | `ESC n` | [`charset::LockingShift2`] |
//! | `ESC o` | [`charset::LockingShift3`] |
//! | `ESC ~` | [`charset::LockingShift1Right`] |
//! | `ESC }` | [`charset::LockingShift2Right`] |
//! | `ESC \|` | [`charset::LockingShift3Right`] |
//! | `ESC N` | [`charset::SingleShift2`] |
//! | `ESC O` | [`charset::SingleShift3`] |
//! | `ESC ( Pc` | [`charset::DesignateG0`] |
//! | `ESC ) Pc` | [`charset::DesignateG1`] |
//! | `ESC * Pc` | [`charset::DesignateG2`] |
//! | `ESC + Pc` | [`charset::DesignateG3`] |
//! | `ESC - Pc` | [`charset::DesignateG1_96`] |
//! | `ESC . Pc` | [`charset::DesignateG2_96`] |
//! | `ESC / Pc` | [`charset::DesignateG3_96`] |
//!
//! ## color
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `OSC 4 ; Pc ; Pt ST` | [`color::TerminalPaletteColorResponse`], [`color::RequestOrSetTerminalPaletteColor`] |
//! | `OSC 10 ; Pt ST` | [`color::SpecialTextForegroundColorResponse`], [`color::RequestOrSetSpecialTextForegroundColor`] |
//! | `OSC 11 ; Pt ST` | [`color::SpecialTextBackgroundColorResponse`], [`color::RequestOrSetSpecialTextBackgroundColor`] |
//! | `OSC 12 ; Pt ST` | [`color::CursorColorResponse`], [`color::RequestOrSetCursorColor`] |
//! | `OSC 13 ; Pt ST` | [`color::PointerForegroundColorResponse`], [`color::RequestOrSetPointerForegroundColor`] |
//! | `OSC 14 ; Pt ST` | [`color::PointerBackgroundColorResponse`], [`color::RequestOrSetPointerBackgroundColor`] |
//! | `OSC 15 ; Pt ST` | [`color::TektronixForegroundColorResponse`], [`color::RequestOrSetTektronixForegroundColor`] |
//! | `OSC 16 ; Pt ST` | [`color::TektronixBackgroundColorResponse`], [`color::RequestOrSetTektronixBackgroundColor`] |
//! | `OSC 17 ; Pt ST` | [`color::HighlightBackgroundColorResponse`], [`color::RequestOrSetHighlightBackgroundColor`] |
//! | `OSC 18 ; Pt ST` | [`color::TektronixCursorColorResponse`], [`color::RequestOrSetTektronixCursorColor`] |
//! | `OSC 19 ; Pt ST` | [`color::HighlightForegroundColorResponse`], [`color::RequestOrSetHighlightForegroundColor`] |
//!
//! ## cursor
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `ESC 7` | [`cursor::SaveCursor`] |
//! | `ESC 8` | [`cursor::RestoreCursor`] |
//! | `0x08` (C0 control code) | [`cursor::Backspace`] |
//! | `0x09` (C0 control code) | [`cursor::HorizontalTab`] |
//! | `0x0A` (C0 control code) | [`cursor::LineFeed`] |
//! | `0x0B` (C0 control code) | [`cursor::VerticalTab`] |
//! | `0x0C` (C0 control code) | [`cursor::FormFeed`] |
//! | `0x0D` (C0 control code) | [`cursor::CarriageReturn`] |
//! | `CSI Ps ; Ps H` | [`cursor::SetCursorPosition`] |
//! | `ESC 6` | [`cursor::BackIndex`] |
//! | `ESC 9` | [`cursor::ForwardIndex`] |
//! | `ESC D` | [`cursor::Index`] |
//! | `ESC E` | [`cursor::NextLine`] |
//! | `ESC H` | [`cursor::HorizontalTabSet`] |
//! | `ESC M` | [`cursor::ReverseIndex`] |
//! | `CSI Ps A` | [`cursor::CursorUp`] |
//! | `CSI Ps B` | [`cursor::CursorDown`] |
//! | `CSI Ps D` | [`cursor::CursorLeft`] |
//! | `CSI Ps C` | [`cursor::CursorRight`] |
//! | `CSI Ps E` | [`cursor::CursorNextLine`] |
//! | `CSI Ps F` | [`cursor::CursorPreviousLine`] |
//! | `CSI Ps G` | [`cursor::CursorHorizontalAbsolute`] |
//! | `CSI Ps I` | [`cursor::CursorHorizontalForwardTab`] |
//! | `CSI Ps Z` | [`cursor::CursorHorizontalBackwardTab`] |
//! | `CSI Ps a` | [`cursor::CursorHorizontalRelative`] |
//! | `CSI Ps d` | [`cursor::CursorVerticalAbsolute`] |
//! | `CSI Ps e` | [`cursor::CursorVerticalRelative`] |
//! | `CSI Ps SP q` | [`cursor::SetCursorStyle`] |
//! | `DCS $ q SP q ST` | [`cursor::RequestCursorStyle`] |
//! | `CSI ? Ps ; Ps ; Ps c` | [`cursor::LinuxCursorStyle`] |
//! | `CSI 6 n` | [`cursor::RequestCursorPosition`] |
//! | `CSI Ps ; Ps R` | [`cursor::CursorPositionReport`] |
//! | `CSI 1 $ w` | [`cursor::RequestCursorInformationReport`] |
//! | `DCS 1 $ u Pr ; Pc ; Pp ; Srend ; Satt ; Sflag ; Pgl ; Pgr ; Scss ; Sdesig ST` | [`cursor::CursorInformationReport`] |
//! | `CSI 2 $ w` | [`cursor::RequestTabStopReport`] |
//! | `DCS 2 $ u Pc / Pc / ... ST` | [`cursor::TabStopReport`] |
//!
//! ## dsr
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `CSI 5 n` | [`dsr::RequestOperatingStatus`] |
//! | `CSI 0 n` | [`dsr::OperatingStatusReport`] |
//! | `CSI ? 5 n` | [`dsr::RequestOperatingStatusPrivate`] |
//! | `CSI ? 0 n` | [`dsr::OperatingStatusReportPrivate`] |
//! | `CSI ? 15 n` | [`dsr::RequestPrinterStatus`] |
//! | `CSI ? 10 n` \| `CSI ? 11 n` \| `CSI ? 13 n` | [`dsr::PrinterStatusReport`] |
//! | `CSI ? 25 n` | [`dsr::RequestUdkStatus`] |
//! | `CSI ? 20 n` \| `CSI ? 21 n` | [`dsr::UdkStatusReport`] |
//! | `CSI ? 26 n` | [`dsr::RequestKeyboardStatus`] |
//! | `CSI ? 27 ; Ps n` | [`dsr::KeyboardStatusReport`] |
//! | `CSI ? 55 n` | [`dsr::RequestLocatorStatus`] |
//! | `CSI ? 50 n` \| `CSI ? 53 n` | [`dsr::LocatorStatusReport`] |
//! | `CSI ? 56 n` | [`dsr::RequestLocatorType`] |
//! | `CSI ? 57 ; Ps n` | [`dsr::LocatorTypeReport`] |
//! | `CSI ? 62 n` | [`dsr::RequestMacroSpaceStatus`] |
//! | `CSI ? 62 ; Ps n` | [`dsr::MacroSpaceReport`] |
//! | `CSI ? 63 ; Ps n` | [`dsr::RequestMemoryChecksum`] |
//! | `CSI ? 63 ; Ps ; Ps n` | [`dsr::MemoryChecksumReport`] |
//! | `CSI ? 75 n` | [`dsr::RequestDataIntegrityStatus`] |
//! | `CSI ? 70 n` \| `CSI ? 71 n` | [`dsr::DataIntegrityReport`] |
//! | `CSI ? 85 n` | [`dsr::RequestMultipleSessionStatus`] |
//! | `CSI ? 80 n` \| `CSI ? 81 n` \| `CSI ? 83 n` | [`dsr::MultipleSessionReport`] |
//! | `CSI ? 996 n` | [`dsr::RequestColorPreference`] |
//! | `CSI ? 997 ; Ps n` | [`dsr::ColorPreferenceReport`] |
//!
//! ## iterm
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `OSC 1337 ; SetMark ST` | [`iterm::SetMark`] |
//! | `OSC 1337 ; StealFocus ST` | [`iterm::StealFocus`] |
//! | `OSC 1337 ; ClearScrollback ST` | [`iterm::ClearScrollback`] |
//! | `OSC 1337 ; EndCopy ST` | [`iterm::EndCopy`] |
//! | `OSC 1337 ; ReportCellSize ST` | [`iterm::ReportCellSize`] |
//! | `OSC 1337 ; PushKeyLabels ST` | [`iterm::PushKeyLabels`] |
//! | `OSC 1337 ; PopKeyLabels ST` | [`iterm::PopKeyLabels`] |
//! | `OSC 1337 ; Disinter ST` | [`iterm::Disinter`] |
//! | `OSC 1337 ; ClearCapturedOutput ST` | [`iterm::ClearCapturedOutput`] |
//! | `OSC 1337 ; CursorShape = Ps ST` | [`iterm::CursorShape`] |
//! | `OSC 1337 ; CurrentDir = Pt ST` | [`iterm::CurrentDir`] |
//! | `OSC 1337 ; SetProfile = Pt ST` | [`iterm::SetProfile`] |
//! | `OSC 1337 ; CopyToClipboard = Ps ST` | [`iterm::CopyToClipboard`] |
//! | `OSC 1337 ; SetBackgroundImageFile = Pt ST` | [`iterm::SetBackgroundImageFile`] |
//! | `OSC 1337 ; RequestAttention = Ps ST` | [`iterm::RequestAttention`] |
//! | `OSC 1337 ; UnicodeVersion = Ps ST` | [`iterm::UnicodeVersion`] |
//! | `OSC 1337 ; HighlightCursorLine = Ps ST` | [`iterm::HighlightCursorLine`] |
//! | `OSC 1337 ; Copy = Pt ST` | [`iterm::Copy`] |
//! | `OSC 1337 ; ReportVariable = Pt ST` | [`iterm::ReportVariable`] |
//! | `OSC 1337 ; RequestUpload = Pt ST` | [`iterm::RequestUpload`] |
//! | `OSC 1337 ; OpenUrl = Pt ST` | [`iterm::OpenUrl`] |
//! | `OSC 1337 ; key=value ; ... ST` | [`iterm::GenericCommand`] |
//! | `OSC 1337 ; AddAnnotation = [length\|]message[\|x\|y] ST` | [`iterm::AddAnnotation`] |
//! | `OSC 1337 ; AddHiddenAnnotation = [length\|]message[\|x\|y] ST` | [`iterm::AddHiddenAnnotation`] |
//!
//! ## keyboard
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `CSI ? u` | [`keyboard::KeyboardEnhancementFlagsQuery`] |
//! | `CSI ? Ps u` | [`keyboard::KeyboardEnhancementFlagsResponse`] |
//! | `CSI > Ps u` | [`keyboard::PushKeyboardEnhancementFlags`] |
//! | `CSI = Ps u` | [`keyboard::SetKeyboardEnhancementFlags`] |
//! | `CSI < u` | [`keyboard::PopKeyboardEnhancementFlags`] |
//! | `ESC =` | [`keyboard::SetApplicationKeypadMode`] |
//! | `ESC >` | [`keyboard::ResetApplicationKeypadMode`] |
//! | `CSI Ps ; Pm ~` | [`keyboard::KeyEvent`] |
//! | `CSI Ps ; Pm u` | [`keyboard::KeyEvent`] |
//! | `CSI A` / `CSI B` / `CSI C` / `CSI D` / `CSI F` / `CSI H` / `CSI P` / `CSI Q` / `CSI R` / `CSI S` / `CSI Z` | [`keyboard::KeyEvent`] |
//! | `CSI 1 ; Pm A` / `CSI 1 ; Pm B` / etc. | [`keyboard::KeyEvent`] |
//!
//! ## mouse
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `CSI < Pb ; Px ; Py M` (press) / `CSI < Pb ; Px ; Py m` (release) | [`mouse::SgrMouseEventSeq`] |
//! | `CSI Pb ; Px ; Py M` | [`mouse::UrxvtMouseEventSeq`] |
//! | `CSI M Cb Cx Cy` | [`mouse::DefaultMouseEvent`] |
//! | `CSI M Cb Cx Cy` (UTF-8 encoded values) | [`mouse::MultibyteMouseEvent`] |
//! | `CSI ? Ps ; Ps m` | [`mouse::SetLinuxMousePointerStyle`] |
//! | `CSI Ps ; Ps ; Ps ; Ps ; Ps T` | [`mouse::TrackMouse`] |
//!
//! ## screen
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `CSI Ps J` where `Ps` = 0 (default) | [`screen::EraseDisplayBelow`] |
//! | `CSI 1 J` | [`screen::EraseDisplayAbove`] |
//! | `CSI 2 J` | [`screen::EraseDisplayComplete`] |
//! | `CSI 3 J` | [`screen::EraseDisplayScrollback`] |
//! | `CSI Ps K` where `Ps` = 0 (default) | [`screen::EraseLineRight`] |
//! | `CSI 1 K` | [`screen::EraseLineLeft`] |
//! | `CSI 2 K` | [`screen::EraseLineComplete`] |
//! | `CSI ? Ps J` where `Ps` = 0 (default) | [`screen::SelectiveEraseDisplayBelow`] |
//! | `CSI ? 1 J` | [`screen::SelectiveEraseDisplayAbove`] |
//! | `CSI ? 2 J` | [`screen::SelectiveEraseDisplayComplete`] |
//! | `CSI ? Ps K` where `Ps` = 0 (default) | [`screen::SelectiveEraseLineRight`] |
//! | `CSI ? 1 K` | [`screen::SelectiveEraseLineLeft`] |
//! | `CSI ? 2 K` | [`screen::SelectiveEraseLineComplete`] |
//! | `CSI Ps @` | [`screen::InsertCharacter`] |
//! | `CSI Ps X` | [`screen::EraseCharacter`] |
//! | `CSI Ps b` | [`screen::RepeatCharacter`] |
//! | `CSI Ps L` | [`screen::InsertLine`] |
//! | `CSI Ps M` | [`screen::DeleteLine`] |
//! | `CSI Ps P` | [`screen::DeleteCharacter`] |
//! | `CSI Ps ' }` | [`screen::InsertColumn`] |
//! | `CSI Ps ' ~` | [`screen::DeleteColumn`] |
//! | `ESC # 8` | [`screen::FillScreenWithE`] |
//! | `ESC # 3` | [`screen::SetDoubleHeightLineTopHalf`] |
//! | `ESC # 4` | [`screen::SetDoubleHeightLineBottomHalf`] |
//! | `ESC # 5` | [`screen::SetSingleWidthLine`] |
//! | `ESC # 6` | [`screen::SetDoubleWidthLine`] |
//!
//! ## scroll
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `CSI Ps ; Ps r` | [`scroll::SetTopAndBottomMargins`] |
//! | `CSI Ps ; Ps s` | [`scroll::SetLeftAndRightMargins`] |
//! | `DCS $ q r ST` | [`scroll::RequestTopBottomMargins`] |
//! | `DCS $ q s ST` | [`scroll::RequestLeftRightMargins`] |
//! | `CSI Ps S` | [`scroll::ScrollUp`] |
//! | `CSI Ps T` | [`scroll::ScrollDown`] |
//! | `CSI Ps SP @` | [`scroll::ScrollLeft`] |
//! | `CSI Ps SP A` | [`scroll::ScrollRight`] |
//!
//! ## sgr
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `CSI Pm m` | [`sgr::Sgr`], [`sgr::LegacySgr`] |
//!
//! ## shell
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `OSC 7 ; Pt ST` | [`shell::CurrentLocation`] |
//! | `OSC 133 ; A ST` | [`shell::PromptStart`] |
//! | `OSC 133 ; B ST` | [`shell::PromptEnd`] |
//! | `OSC 133 ; C ST` | [`shell::CommandStart`] |
//! | `OSC 133 ; D ST` or `OSC 133 ; D ; Ps ST` | [`shell::CommandEnd`] |
//!
//! ## terminal
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `CSI 200 ~` | [`terminal::BracketedPasteStart`] |
//! | `CSI 201 ~` | [`terminal::BracketedPasteEnd`] |
//! | `0x07` (C0 control code) | [`terminal::Bell`] |
//! | `DCS $ q m ST` | [`terminal::RequestTextAttributes`] |
//! | `ESC c` | [`terminal::FullReset`] |
//! | `CSI ! p` | [`terminal::SoftReset`] |
//! | `ESC Z` | [`terminal::RequestTerminalID`] |
//! | `CSI c` or `CSI 0 c` | [`terminal::RequestPrimaryDeviceAttributes`] |
//! | `CSI ? Ps ; Ps ; ... c` | [`terminal::PrimaryDeviceAttributesResponse`] |
//! | `CSI > c` or `CSI > 0 c` | [`terminal::RequestSecondaryDeviceAttributes`] |
//! | `CSI > Ps ; Ps ; Ps c` | [`terminal::SecondaryDeviceAttributesResponse`] |
//! | `CSI = 0 c` | [`terminal::RequestTerminalUnitId`] |
//! | `DCS ! \| Pt ST` | [`terminal::TertiaryDeviceAttributesResponse`] |
//! | `CSI Ps ; Ps " p` | [`terminal::SelectVTConformanceLevel`] |
//! | `DCS $ q " p ST` | [`terminal::RequestVTConformanceLevel`] |
//! | `DCS + q Pt ST` | [`terminal::RequestTermcap`] |
//! | `DCS Ps + r Pt ST` | [`terminal::TermcapQueryResponse`] |
//! | `CSI > 0 q` | [`terminal::RequestTerminalNameAndVersion`] |
//! | `DCS > \| Pt ST` | [`terminal::TerminalNameAndVersionResponse`] |
//!
//! ## window
//!
//! | Sequence | Struct |
//! |----------|--------|
//! | `OSC 0 ; Pt ST` | [`window::SetTitleAndIconName`] |
//! | `OSC 2 ; Pt ST` | [`window::SetTitle`] |
//! | `OSC 1 ; Pt ST` | [`window::SetIconName`] |
//! | `CSI 21 t` | [`window::GetTitle`] |
//! | `CSI 20 t` | [`window::GetIconName`] |
//! | `CSI 22 ; Ps t` | [`window::PushTitle`] |
//! | `CSI 23 ; Ps t` | [`window::PopTitle`] |
//! | `CSI 1 t` | [`window::RestoreWindow`] |
//! | `CSI 2 t` | [`window::MinimizeWindow`] |
//! | `CSI 5 t` | [`window::RaiseWindow`] |
//! | `CSI 6 t` | [`window::LowerWindow`] |
//! | `CSI 7 t` | [`window::RefreshWindow`] |
//! | `CSI 8 ; Ps ; Ps t` | [`window::SetSize`] |
//! | `CSI 15 t` | [`window::ReportScreenSizePixels`] |
//! | `CSI 14 ; Ps t` | [`window::ReportWindowSizePixels`] |
//! | `CSI 16 t` | [`window::ReportCellSizePixels`] |
//! | `CSI 19 t` | [`window::ReportScreenSize`] |
//! | `CSI 18 t` | [`window::ReportSize`] |
//! | `CSI Ps t` where `Ps` is 1 (not iconified) or 2 (iconified) | [`window::WindowStateReport`] |
//! | `CSI 3 ; Ps ; Ps t` | [`window::WindowPositionReport`] |
//! | `CSI 4 ; Ps ; Ps t` | [`window::WindowSizePixelsReport`] |
//! | `CSI 5 ; Ps ; Ps t` | [`window::ScreenSizePixelsReport`] |
//! | `CSI 6 ; Ps ; Ps t` | [`window::CellSizePixelsReport`] |

pub mod charset;
pub mod clipboard;
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
