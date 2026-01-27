//! Mouse event types and mode control commands.
//!
//! This module provides types for handling mouse events in terminal applications,
//! along with mode control commands for enabling different mouse tracking modes.
//!
//! # Mouse Event Formats
//!
//! Terminals support several mouse reporting formats:
//!
//! - **Default format**: Raw bytes after `CSI M`, limited range (1-223)
//! - **Multibyte format** (p1005): UTF-8 encoded, extended range (1-2015)
//! - **SGR format** (p1006): Digit sequences with `<` private marker, preferred format
//! - **urxvt format** (p1015): Digit sequences without private marker
//!
//! All formats parse into the common [`MouseEvent`] structure.
//!
//! # Example
//!
//! ```ignore
//! use vtio::event::mouse::{MouseEvent, MouseEventKind, MouseButton};
//!
//! // Enable mouse tracking
//! write!(stdout, "{}", EnableMouseCapture)?;
//!
//! // Later, disable mouse tracking
//! write!(stdout, "{}", DisableMouseCapture)?;
//! ```
//!
//! See <https://terminalguide.namepad.de/mouse/> for protocol details.

mod encoding;
mod event;
mod mode;
mod track;

// Re-export core event types
pub use event::{
    MouseButton, MouseEvent, MouseEventKind, MouseKeyModifiers,
    modifiers_from_button_code,
};

// Re-export encoding types
pub use encoding::{
    DefaultMouseEvent, MultibyteMouseEvent, SgrMouseEvent, SgrMouseEventSeq,
    UrxvtMouseEvent, UrxvtMouseEventSeq, parse_mouse_event_bytes,
};

// Re-export mode types
pub use mode::{
    // Mouse event modes
    DisableMouseAnyEventTrackingMode,
    // Composite commands
    DisableMouseCapture,
    DisableMouseClickAndDragTrackingMode,
    DisableMouseDownUpTrackingMode,
    DisableMouseHighlightMode,
    DisableMouseReportMultibyteMode,
    DisableMouseReportRxvtMode,
    DisableMouseReportSgrMode,
    DisableMouseWheelToCursorKeysMode,
    DisableMouseX10Mode,
    DisableSgrMousePixelMode,
    EnableMouseAnyEventTrackingMode,
    EnableMouseCapture,
    EnableMouseClickAndDragTrackingMode,
    EnableMouseDownUpTrackingMode,
    EnableMouseHighlightMode,
    EnableMouseReportMultibyteMode,
    EnableMouseReportRxvtMode,
    EnableMouseReportSgrMode,
    EnableMouseWheelToCursorKeysMode,
    EnableMouseX10Mode,
    EnableSgrMousePixelMode,
    MouseAnyEventTrackingMode,
    MouseClickAndDragTrackingMode,
    MouseDownUpTrackingMode,
    MouseHighlightMode,
    MouseReportMultibyteMode,
    MouseReportRxvtMode,
    MouseReportSgrMode,
    MouseWheelToCursorKeysMode,
    MouseX10Mode,
    // Pointer mode
    PointerMode,
    RequestMouseAnyEventTrackingMode,
    RequestMouseClickAndDragTrackingMode,
    RequestMouseDownUpTrackingMode,
    RequestMouseHighlightMode,
    RequestMouseReportMultibyteMode,
    RequestMouseReportRxvtMode,
    RequestMouseReportSgrMode,
    RequestMouseWheelToCursorKeysMode,
    RequestMouseX10Mode,
    RequestSgrMousePixelMode,
    SetLinuxMousePointerStyle,
    SetPointerMode,
    SgrMousePixelMode,
};

pub use track::TrackMouse;
