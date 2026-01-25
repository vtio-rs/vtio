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
/// Request the terminal's operating status.
///
/// The terminal always replies with [`OperatingStatusReport`] (`CSI 0 n`).
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

/// Operating Status Report (`DSR`).
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
/// Request the terminal's operating status using the DEC private mode variant.
///
/// The terminal replies with [`OperatingStatusReportPrivate`].
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
/// Request the printer status (DSR 15).
///
/// The terminal replies with [`PrinterStatusReport`].
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
/// Request the status of user-defined keys (DSR 25).
///
/// The terminal replies with [`UdkStatusReport`].
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
/// Request the keyboard status (DSR 26).
///
/// The terminal replies with [`KeyboardStatusReport`].
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
/// Request the status of the DEC locator (mouse) (DSR 55).
///
/// The terminal replies with [`LocatorStatusReport`].
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
/// Request the type of the DEC locator (mouse) (DSR 56).
///
/// The terminal replies with [`LocatorTypeReport`].
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
/// Request the available macro space (DSR 62).
///
/// The terminal replies with [`MacroSpaceReport`].
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
/// Request a memory checksum (DSR 63).
///
/// The terminal replies with [`MemoryChecksumReport`].
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
/// Request data integrity status (DSR 75).
///
/// The terminal replies with [`DataIntegrityReport`].
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
/// Request multiple session status (DSR 85).
///
/// The terminal replies with [`MultipleSessionReport`].
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
/// The terminal replies with [`ColorPreferenceReport`].
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
/// Response to [`RequestOperatingStatusPrivate`].
/// Indicates the terminal is operating correctly.
///
/// Matches: `CSI ? 0 n`
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
/// Response to [`RequestPrinterStatus`].
///
/// Matches: `CSI ? 10 n` | `CSI ? 11 n` | `CSI ? 13 n`
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
/// Response to [`RequestUdkStatus`].
///
/// Matches: `CSI ? 20 n` | `CSI ? 21 n`
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
/// Response to [`RequestKeyboardStatus`].
///
/// Matches: `CSI ? 27 ; dialect n`
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
/// Response to [`RequestLocatorStatus`].
///
/// Matches: `CSI ? 50 n` | `CSI ? 53 n`
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
/// Response to [`RequestLocatorType`].
///
/// Matches: `CSI ? 57 ; type n`
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
/// Response to [`RequestMacroSpaceStatus`].
///
/// Matches: `CSI ? 62 ; space n`
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
/// Response to [`RequestMemoryChecksum`].
///
/// Matches: `CSI ? 63 ; id ; checksum n`
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
/// Response to [`RequestDataIntegrityStatus`].
///
/// Matches: `CSI ? 70 n` | `CSI ? 71 n`
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
/// Response to [`RequestMultipleSessionStatus`].
///
/// Matches: `CSI ? 80 n` | `CSI ? 81 n` | `CSI ? 83 n`
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
/// Response to [`RequestColorPreference`].
///
/// Matches: `CSI ? 997 ; color n`
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
