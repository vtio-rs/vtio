//! Device Status Report (DSR) sequences.
//!
//! These sequences allow applications to request status information from
//! the terminal.
//!
//! See <https://terminalguide.namepad.de/seq/csi_sn/> for details.

// ============================================================================
// Standard (non-private) DSR
// ============================================================================

/// Request Operating Status (`DSR`).
///
/// *Sequence*: `CSI 5 n`
///
/// Request the terminal's operating status.
///
/// The terminal replies with `CSI 0 n` ([`OperatingStatusReport`]).
///
/// See <https://terminalguide.namepad.de/seq/csi_sn-5/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, params = ["5"], finalbyte = 'n')]
pub struct RequestOperatingStatus;

/// Operating Status Report (`DSR` response).
///
/// *Sequence*: `CSI 0 n`
///
/// Response from the terminal to [`RequestOperatingStatus`].
///
/// This report indicates that the terminal is operating correctly.
///
/// See <https://terminalguide.namepad.de/seq/csi_sn-5/> for terminal
/// support specifics.
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
#[vtansi(csi, params = ["0"], finalbyte = 'n')]
pub struct OperatingStatusReport;

// ============================================================================
// Private DSR Requests
// ============================================================================

/// Request Operating Status (private mode) (`DSR`).
///
/// *Sequence*: `CSI ? 5 n`
///
/// Request the terminal's operating status using the DEC private mode variant.
///
/// The terminal replies with `CSI ? 0 n` ([`OperatingStatusReportPrivate`]).
///
/// See <https://terminalguide.namepad.de/seq/csi_sn/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', params = ["5"], finalbyte = 'n')]
pub struct RequestOperatingStatusPrivate;

/// Request Printer Status (`DSR`).
///
/// *Sequence*: `CSI ? 15 n`
///
/// Request the printer status.
///
/// The terminal replies with `CSI ? Ps n` ([`PrinterStatusReport`]).
///
/// See <https://terminalguide.namepad.de/seq/csi_sn/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', params = ["15"], finalbyte = 'n')]
pub struct RequestPrinterStatus;

/// Request User Defined Key Status (`DSR`).
///
/// *Sequence*: `CSI ? 25 n`
///
/// Request the status of user-defined keys.
///
/// The terminal replies with `CSI ? Ps n` ([`UdkStatusReport`]).
///
/// See <https://terminalguide.namepad.de/seq/csi_sn/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', params = ["25"], finalbyte = 'n')]
pub struct RequestUdkStatus;

/// Request Keyboard Status (`DSR`).
///
/// *Sequence*: `CSI ? 26 n`
///
/// Request the keyboard status.
///
/// The terminal replies with `CSI ? 27 ; Ps n` ([`KeyboardStatusReport`]).
///
/// See <https://terminalguide.namepad.de/seq/csi_sn/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', params = ["26"], finalbyte = 'n')]
pub struct RequestKeyboardStatus;

/// Request DEC Locator Status (`DSR`).
///
/// *Sequence*: `CSI ? 55 n`
///
/// Request the status of the DEC locator (mouse).
///
/// The terminal replies with `CSI ? Ps n` ([`LocatorStatusReport`]).
///
/// See <https://terminalguide.namepad.de/seq/csi_sn/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', params = ["55"], finalbyte = 'n')]
pub struct RequestLocatorStatus;

/// Request DEC Locator Type (`DSR`).
///
/// *Sequence*: `CSI ? 56 n`
///
/// Request the type of the DEC locator (mouse).
///
/// The terminal replies with `CSI ? 57 ; Ps n` ([`LocatorTypeReport`]).
///
/// See <https://terminalguide.namepad.de/seq/csi_sn/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', params = ["56"], finalbyte = 'n')]
pub struct RequestLocatorType;

/// Request Macro Space Status (`DSR`).
///
/// *Sequence*: `CSI ? 62 n`
///
/// Request the available macro space.
///
/// The terminal replies with `CSI ? Ps * {` ([`MacroSpaceReport`]).
///
/// See <https://terminalguide.namepad.de/seq/csi_sn/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', params = ["62"], finalbyte = 'n')]
pub struct RequestMacroSpaceStatus;

/// Request Memory Checksum (`DSR`).
///
/// *Sequence*: `CSI ? 63 ; Ps n`
///
/// Request a memory checksum for the specified region.
///
/// The terminal replies with `DCS Ps ! ~ D...D ST` ([`MemoryChecksumReport`]).
///
/// See <https://terminalguide.namepad.de/seq/csi_sn/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', params = ["63"], finalbyte = 'n')]
pub struct RequestMemoryChecksum {
    /// Identifier for the memory region to checksum.
    pub id: u16,
}

/// Request Data Integrity Status (`DSR`).
///
/// *Sequence*: `CSI ? 75 n`
///
/// Request data integrity status.
///
/// The terminal replies with `CSI ? Ps n` ([`DataIntegrityReport`]).
///
/// See <https://terminalguide.namepad.de/seq/csi_sn/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', params = ["75"], finalbyte = 'n')]
pub struct RequestDataIntegrityStatus;

/// Request Multiple Session Status (`DSR`).
///
/// *Sequence*: `CSI ? 85 n`
///
/// Request multiple session status.
///
/// The terminal replies with `CSI ? Ps n` ([`MultipleSessionReport`]).
///
/// See <https://terminalguide.namepad.de/seq/csi_sn/> for terminal
/// support specifics.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', params = ["85"], finalbyte = 'n')]
pub struct RequestMultipleSessionStatus;

/// Request Current Color Preference.
///
/// *Sequence*: `CSI ? 996 n`
///
/// The terminal replies with `CSI ? 997 ; Ps n` ([`ColorPreferenceReport`]).
///
/// See <https://contour-terminal.org/vt-extensions/color-palette-update-notifications/>
/// for more information.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', params = ["996"], finalbyte = 'n')]
pub struct RequestColorPreference;

// ============================================================================
// Private DSR Reports - Status Enums
// ============================================================================

/// Printer status values.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Default,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u8)]
pub enum PrinterStatus {
    /// Printer is ready.
    #[default]
    Ready = 10,
    /// Printer is not ready.
    NotReady = 11,
    /// No printer is available.
    NoPrinter = 13,
}

/// User-defined key (UDK) status values.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Default,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u8)]
pub enum UdkStatus {
    /// User-defined keys are locked.
    #[default]
    Locked = 20,
    /// User-defined keys are unlocked.
    Unlocked = 21,
}

/// DEC locator status values.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Default,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u8)]
pub enum LocatorStatus {
    /// No locator device is available.
    #[default]
    NoLocator = 50,
    /// A locator device is available.
    Available = 53,
}

/// Data integrity status values.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Default,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u8)]
pub enum DataIntegrityStatus {
    /// Ready, no malfunction detected.
    #[default]
    Ready = 70,
    /// Malfunction detected.
    Malfunction = 71,
}

/// Multiple session status values.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Default,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u8)]
pub enum MultipleSessionStatus {
    /// Sessions are available.
    #[default]
    Available = 80,
    /// No sessions are available.
    NotAvailable = 81,
    /// Not configured for multiple sessions.
    NotConfigured = 83,
}

/// Color preference status values.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Default,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u8)]
pub enum ColorPreference {
    /// Dark mode is preferred.
    #[default]
    DarkMode = 1,
    /// Light mode is preferred.
    LightMode = 2,
}

// ============================================================================
// Private DSR Reports - Response Structs
// ============================================================================

/// Operating Status Report (private mode).
///
/// *Sequence*: `CSI ? 0 n`
///
/// Response to [`RequestOperatingStatusPrivate`].
/// Indicates the terminal is operating correctly.
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
#[vtansi(csi, private = '?', params = ["0"], finalbyte = 'n')]
pub struct OperatingStatusReportPrivate;

/// Printer Status Report.
///
/// *Sequence*: `CSI ? 10 n` | `CSI ? 11 n` | `CSI ? 13 n`
///
/// Response to [`RequestPrinterStatus`].
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
#[vtansi(csi, private = '?', params = ["10"] | ["11"] | ["13"], finalbyte = 'n')]
pub struct PrinterStatusReport {
    /// The printer status.
    #[vtansi(locate = "static_params")]
    pub status: PrinterStatus,
}

/// User-Defined Key Status Report.
///
/// *Sequence*: `CSI ? 20 n` | `CSI ? 21 n`
///
/// Response to [`RequestUdkStatus`].
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
#[vtansi(csi, private = '?', params = ["20"] | ["21"], finalbyte = 'n')]
pub struct UdkStatusReport {
    /// The UDK status.
    #[vtansi(locate = "static_params")]
    pub status: UdkStatus,
}

/// Keyboard Status Report.
///
/// *Sequence*: `CSI ? 27 ; Ps n`
///
/// Response to [`RequestKeyboardStatus`].
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
#[vtansi(csi, private = '?', params = ["27"], finalbyte = 'n')]
pub struct KeyboardStatusReport {
    /// Keyboard dialect code.
    pub dialect: u8,
}

/// DEC Locator Status Report.
///
/// *Sequence*: `CSI ? 50 n` | `CSI ? 53 n`
///
/// Response to [`RequestLocatorStatus`].
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
#[vtansi(csi, private = '?', params = ["50"] | ["53"], finalbyte = 'n')]
pub struct LocatorStatusReport {
    /// The locator status.
    #[vtansi(locate = "static_params")]
    pub status: LocatorStatus,
}

/// DEC Locator Type Report.
///
/// *Sequence*: `CSI ? 57 ; Ps n`
///
/// Response to [`RequestLocatorType`].
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
#[vtansi(csi, private = '?', params = ["57"], finalbyte = 'n')]
pub struct LocatorTypeReport {
    /// Locator type code.
    pub locator_type: u8,
}

/// Macro Space Report.
///
/// *Sequence*: `CSI ? 62 ; Ps n`
///
/// Response to [`RequestMacroSpaceStatus`].
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
#[vtansi(csi, private = '?', params = ["62"], finalbyte = 'n')]
pub struct MacroSpaceReport {
    /// Available macro space in bytes.
    pub space: u16,
}

/// Memory Checksum Report.
///
/// *Sequence*: `CSI ? 63 ; Ps ; Ps n`
///
/// Response to [`RequestMemoryChecksum`].
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
#[vtansi(csi, private = '?', params = ["63"], finalbyte = 'n')]
pub struct MemoryChecksumReport {
    /// Memory region identifier.
    pub id: u16,
    /// Checksum value.
    pub checksum: u16,
}

/// Data Integrity Status Report.
///
/// *Sequence*: `CSI ? 70 n` | `CSI ? 71 n`
///
/// Response to [`RequestDataIntegrityStatus`].
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
#[vtansi(csi, private = '?', params = ["70"] | ["71"], finalbyte = 'n')]
pub struct DataIntegrityReport {
    /// The data integrity status.
    pub status: DataIntegrityStatus,
}

/// Multiple Session Status Report.
///
/// *Sequence*: `CSI ? 80 n` | `CSI ? 81 n` | `CSI ? 83 n`
///
/// Response to [`RequestMultipleSessionStatus`].
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
#[vtansi(csi, private = '?', params = ["80"] | ["81"] | ["83"], finalbyte = 'n')]
pub struct MultipleSessionReport {
    /// The multiple session status.
    pub status: MultipleSessionStatus,
}

/// Color Preference Report.
///
/// *Sequence*: `CSI ? 997 ; Ps n`
///
/// Response to [`RequestColorPreference`].
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
#[vtansi(csi, private = '?', params = ["997"], finalbyte = 'n')]
pub struct ColorPreferenceReport {
    /// The color preference.
    pub color: ColorPreference,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::AnsiEncode;

    #[test]
    fn test_request_operating_status() {
        assert_eq!(RequestOperatingStatus.encode_ansi().unwrap(), b"\x1b[5n");
    }

    #[test]
    fn test_request_operating_status_private() {
        assert_eq!(
            RequestOperatingStatusPrivate.encode_ansi().unwrap(),
            b"\x1b[?5n"
        );
    }

    #[test]
    fn test_request_printer_status() {
        assert_eq!(RequestPrinterStatus.encode_ansi().unwrap(), b"\x1b[?15n");
    }

    #[test]
    fn test_request_memory_checksum() {
        let req = RequestMemoryChecksum { id: 42 };
        assert_eq!(req.encode_ansi().unwrap(), b"\x1b[?63;42n");
    }

    #[test]
    fn test_request_udk_status() {
        assert_eq!(RequestUdkStatus.encode_ansi().unwrap(), b"\x1b[?25n");
    }

    #[test]
    fn test_request_keyboard_status() {
        assert_eq!(RequestKeyboardStatus.encode_ansi().unwrap(), b"\x1b[?26n");
    }

    #[test]
    fn test_request_locator_status() {
        assert_eq!(RequestLocatorStatus.encode_ansi().unwrap(), b"\x1b[?55n");
    }

    #[test]
    fn test_request_locator_type() {
        assert_eq!(RequestLocatorType.encode_ansi().unwrap(), b"\x1b[?56n");
    }

    #[test]
    fn test_request_macro_space_status() {
        assert_eq!(
            RequestMacroSpaceStatus.encode_ansi().unwrap(),
            b"\x1b[?62n"
        );
    }

    #[test]
    fn test_request_data_integrity_status() {
        assert_eq!(
            RequestDataIntegrityStatus.encode_ansi().unwrap(),
            b"\x1b[?75n"
        );
    }

    #[test]
    fn test_request_multiple_session_status() {
        assert_eq!(
            RequestMultipleSessionStatus.encode_ansi().unwrap(),
            b"\x1b[?85n"
        );
    }

    #[test]
    fn test_request_color_preference() {
        assert_eq!(
            RequestColorPreference.encode_ansi().unwrap(),
            b"\x1b[?996n"
        );
    }
}
