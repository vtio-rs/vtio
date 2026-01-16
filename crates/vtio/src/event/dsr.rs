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
/// The terminal always replies with:
///
/// `CSI 0 n` (operating correctly)
///
/// This is a basic status check that indicates the terminal is
/// functioning and able to respond to commands.
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
/// The status code is always 0.
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
/// Request the terminal's operating status using the DEC private mode
/// variant.
///
/// The terminal replies with [`DsrReport`] containing
/// [`DsrReportKind::OperatingStatus`].
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
/// Request the printer status (historically DSR 15).
///
/// The terminal replies with [`DsrReport`] containing
/// [`DsrReportKind::Printer`].
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
/// Request the status of user-defined keys (historically DSR 25).
///
/// The terminal replies with [`DsrReport`] containing
/// [`DsrReportKind::Udk`].
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
/// Request the keyboard status (historically DSR 26).
///
/// The terminal replies with [`DsrReport`] containing
/// [`DsrReportKind::Keyboard`].
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
/// Request the status of the DEC locator (mouse).
///
/// The terminal replies with [`DsrReport`] containing
/// [`DsrReportKind::Locator`].
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
/// Request the type of the DEC locator (mouse).
///
/// The terminal replies with [`DsrReport`] containing
/// [`DsrReportKind::LocatorType`].
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
/// Request the available macro space (historically DSR 62).
///
/// The terminal replies with [`DsrReport`] containing
/// [`DsrReportKind::MacroSpace`].
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
/// Request a memory checksum (historically DSR 63).
///
/// The terminal replies with [`DsrReport`] containing
/// [`DsrReportKind::MemoryChecksum`].
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
/// Request data integrity status (historically DSR 75).
///
/// The terminal replies with [`DsrReport`] containing
/// [`DsrReportKind::DataIntegrity`].
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
/// Request multiple session status (historically DSR 85).
///
/// The terminal replies with [`DsrReport`] containing
/// [`DsrReportKind::MultipleSession`].
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
/// The terminal replies with [`DsrReport`] containing
/// [`DsrReportKind::ColorPreference`].
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
// Private DSR Reports
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
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum PrinterStatus {
    /// Printer is ready.
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
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum UdkStatus {
    /// User-defined keys are locked.
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
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum LocatorStatus {
    /// No locator device is available.
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
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum DataIntegrityStatus {
    /// Ready, no malfunction detected.
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
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum MultipleSessionStatus {
    /// Sessions are available.
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
)]
#[repr(u8)]
pub enum ColorPreference {
    /// Dark mode is preferred.
    #[default]
    DarkMode = 1,
    /// Light mode is preferred.
    LightMode = 2,
}

/// Private DSR report kind.
///
/// This enum represents the different types of DSR reports that can be
/// received from the terminal. The first parameter of the CSI sequence
/// determines which variant is used.
#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Hash)]
pub enum DsrReportKind {
    /// Operating status report (private mode).
    ///
    /// Response to [`RequestOperatingStatusPrivate`].
    /// Indicates the terminal is operating correctly.
    OperatingStatus,

    /// Printer status report.
    ///
    /// Response to [`RequestPrinterStatus`].
    Printer(PrinterStatus),

    /// User-defined key status report.
    ///
    /// Response to [`RequestUdkStatus`].
    Udk(UdkStatus),

    /// Keyboard status report.
    ///
    /// Response to [`RequestKeyboardStatus`].
    /// Contains the keyboard dialect code.
    Keyboard {
        /// Keyboard dialect code.
        dialect: u8,
    },

    /// DEC locator status report.
    ///
    /// Response to [`RequestLocatorStatus`].
    Locator(LocatorStatus),

    /// DEC locator type report.
    ///
    /// Response to [`RequestLocatorType`].
    LocatorType {
        /// Locator type code.
        locator_type: u8,
    },

    /// Data integrity status report.
    ///
    /// Response to [`RequestDataIntegrityStatus`].
    DataIntegrity(DataIntegrityStatus),

    /// Multiple session status report.
    ///
    /// Response to [`RequestMultipleSessionStatus`].
    MultipleSession(MultipleSessionStatus),

    /// Macro space status report.
    ///
    /// Response to [`RequestMacroSpaceStatus`].
    MacroSpace {
        /// Available macro space in bytes.
        space: u16,
    },

    /// Memory checksum report.
    ///
    /// Response to [`RequestMemoryChecksum`].
    MemoryChecksum {
        /// Memory region identifier.
        id: u16,
        /// Checksum value.
        checksum: u16,
    },

    /// Color preference report.
    ///
    /// Response to [`RequestColorPreference`]
    ColorPreference(ColorPreference),

    /// Unknown or unrecognized DSR report.
    ///
    /// Contains the raw status code for reports we don't recognize.
    Unknown {
        /// The first parameter (status code).
        status: u16,
        /// Additional parameters, if any.
        params: Vec<u16>,
    },
}

impl vtansi::AnsiEncode for DsrReportKind {
    fn encode_ansi_into<W: std::io::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        match self {
            DsrReportKind::OperatingStatus => {
                vtansi::write_byte_into(sink, b'0')
            }
            DsrReportKind::Printer(status) => {
                let code: u8 = (*status).into();
                <u8 as vtansi::AnsiEncode>::encode_ansi_into(&code, sink)
            }
            DsrReportKind::Udk(status) => {
                let code: u8 = (*status).into();
                <u8 as vtansi::AnsiEncode>::encode_ansi_into(&code, sink)
            }
            DsrReportKind::Keyboard { dialect } => {
                let mut count = vtansi::write_str_into(sink, "27")?;
                count += vtansi::write_byte_into(sink, b';')?;
                count += <u8 as vtansi::AnsiEncode>::encode_ansi_into(
                    dialect, sink,
                )?;
                Ok(count)
            }
            DsrReportKind::Locator(status) => {
                let code: u8 = (*status).into();
                <u8 as vtansi::AnsiEncode>::encode_ansi_into(&code, sink)
            }
            DsrReportKind::LocatorType { locator_type } => {
                let mut count = vtansi::write_str_into(sink, "57")?;
                count += vtansi::write_byte_into(sink, b';')?;
                count += <u8 as vtansi::AnsiEncode>::encode_ansi_into(
                    locator_type,
                    sink,
                )?;
                Ok(count)
            }
            DsrReportKind::DataIntegrity(status) => {
                let code: u8 = (*status).into();
                <u8 as vtansi::AnsiEncode>::encode_ansi_into(&code, sink)
            }
            DsrReportKind::MultipleSession(status) => {
                let code: u8 = (*status).into();
                <u8 as vtansi::AnsiEncode>::encode_ansi_into(&code, sink)
            }
            DsrReportKind::MacroSpace { space } => {
                let mut count = vtansi::write_str_into(sink, "62")?;
                count += vtansi::write_byte_into(sink, b';')?;
                count +=
                    <u16 as vtansi::AnsiEncode>::encode_ansi_into(space, sink)?;
                Ok(count)
            }
            DsrReportKind::MemoryChecksum { id, checksum } => {
                let mut count = vtansi::write_str_into(sink, "63")?;
                count += vtansi::write_byte_into(sink, b';')?;
                count +=
                    <u16 as vtansi::AnsiEncode>::encode_ansi_into(id, sink)?;
                count += vtansi::write_byte_into(sink, b';')?;
                count += <u16 as vtansi::AnsiEncode>::encode_ansi_into(
                    checksum, sink,
                )?;
                Ok(count)
            }
            DsrReportKind::ColorPreference(color) => {
                let code: u8 = (*color).into();
                <u8 as vtansi::AnsiEncode>::encode_ansi_into(&code, sink)
            }
            DsrReportKind::Unknown { status, params } => {
                let mut count = <u16 as vtansi::AnsiEncode>::encode_ansi_into(
                    status, sink,
                )?;
                for param in params {
                    count += vtansi::write_byte_into(sink, b';')?;
                    count += <u16 as vtansi::AnsiEncode>::encode_ansi_into(
                        param, sink,
                    )?;
                }
                Ok(count)
            }
        }
    }
}

impl<'a> vtansi::TryFromAnsi<'a> for DsrReportKind {
    fn try_from_ansi(bytes: &'a [u8]) -> Result<Self, vtansi::ParseError> {
        // Parse semicolon-separated parameters
        let params: Vec<u16> = bytes
            .split(|b| *b == b';')
            .map(|p| {
                if p.is_empty() {
                    Ok(0)
                } else {
                    <u16 as vtansi::TryFromAnsi>::try_from_ansi(p)
                }
            })
            .collect::<Result<_, _>>()?;

        if params.is_empty() {
            return Err(vtansi::ParseError::InvalidValue(
                "empty DSR report".to_string(),
            ));
        }

        let status = params[0];

        #[allow(clippy::cast_possible_truncation)]
        Ok(match status {
            0 => DsrReportKind::OperatingStatus,
            10 => DsrReportKind::Printer(PrinterStatus::Ready),
            11 => DsrReportKind::Printer(PrinterStatus::NotReady),
            13 => DsrReportKind::Printer(PrinterStatus::NoPrinter),
            20 => DsrReportKind::Udk(UdkStatus::Locked),
            21 => DsrReportKind::Udk(UdkStatus::Unlocked),
            27 => DsrReportKind::Keyboard {
                dialect: params.get(1).copied().unwrap_or(0) as u8,
            },
            50 => DsrReportKind::Locator(LocatorStatus::NoLocator),
            53 => DsrReportKind::Locator(LocatorStatus::Available),
            57 => DsrReportKind::LocatorType {
                locator_type: params.get(1).copied().unwrap_or(0) as u8,
            },
            62 => DsrReportKind::MacroSpace {
                space: params.get(1).copied().unwrap_or(0),
            },
            63 => DsrReportKind::MemoryChecksum {
                id: params.get(1).copied().unwrap_or(0),
                checksum: params.get(2).copied().unwrap_or(0),
            },
            70 => DsrReportKind::DataIntegrity(DataIntegrityStatus::Ready),
            71 => {
                DsrReportKind::DataIntegrity(DataIntegrityStatus::Malfunction)
            }
            80 => {
                DsrReportKind::MultipleSession(MultipleSessionStatus::Available)
            }
            81 => DsrReportKind::MultipleSession(
                MultipleSessionStatus::NotAvailable,
            ),
            83 => DsrReportKind::MultipleSession(
                MultipleSessionStatus::NotConfigured,
            ),
            997 => DsrReportKind::ColorPreference(
                (params.get(1).copied().unwrap_or(1) as u8)
                    .try_into()
                    .unwrap_or_default(),
            ),
            _ => DsrReportKind::Unknown {
                status,
                params: params.into_iter().skip(1).collect(),
            },
        })
    }
}

/// Private DSR report wrapper.
///
/// This wrapper provides transparent encoding/decoding for the inner
/// [`DsrReportKind`] enum, handling the semicolon-delimited parameters.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Hash,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[vtansi(transparent)]
pub struct DsrReportParams(pub DsrReportKind);

impl From<DsrReportKind> for DsrReportParams {
    fn from(kind: DsrReportKind) -> Self {
        Self(kind)
    }
}

impl From<DsrReportParams> for DsrReportKind {
    fn from(params: DsrReportParams) -> Self {
        params.0
    }
}

/// Private DSR report.
///
/// This struct represents the response from the terminal using private mode
/// DSR sequences (`CSI ? Ps n`). The parameters are parsed into the
/// appropriate [`DsrReportKind`] variant based on the first parameter value.
#[derive(
    Debug, PartialOrd, PartialEq, Eq, Clone, Hash, vtansi::derive::AnsiInput,
)]
#[vtansi(csi, private = '?', finalbyte = 'n')]
pub struct DsrReport {
    /// The report kind and associated data.
    pub kind: DsrReportParams,
}

impl DsrReport {
    /// Create a new DSR report with the given kind.
    #[must_use]
    pub fn new(kind: DsrReportKind) -> Self {
        Self {
            kind: DsrReportParams(kind),
        }
    }

    /// Get the report kind.
    #[must_use]
    pub fn kind(&self) -> &DsrReportKind {
        &self.kind.0
    }

    /// Convert into the inner report kind.
    #[must_use]
    pub fn into_kind(self) -> DsrReportKind {
        self.kind.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::AnsiEncode;
    use vtansi::TryFromAnsi;

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
    fn test_dsr_report_kind_operating_status() {
        let kind = DsrReportKind::try_from_ansi(b"0").unwrap();
        assert_eq!(kind, DsrReportKind::OperatingStatus);
        assert_eq!(kind.encode_ansi().unwrap(), b"0");
    }

    #[test]
    fn test_dsr_report_kind_printer_ready() {
        let kind = DsrReportKind::try_from_ansi(b"10").unwrap();
        assert_eq!(kind, DsrReportKind::Printer(PrinterStatus::Ready));
        assert_eq!(kind.encode_ansi().unwrap(), b"10");
    }

    #[test]
    fn test_dsr_report_kind_printer_not_ready() {
        let kind = DsrReportKind::try_from_ansi(b"11").unwrap();
        assert_eq!(kind, DsrReportKind::Printer(PrinterStatus::NotReady));
    }

    #[test]
    fn test_dsr_report_kind_no_printer() {
        let kind = DsrReportKind::try_from_ansi(b"13").unwrap();
        assert_eq!(kind, DsrReportKind::Printer(PrinterStatus::NoPrinter));
    }

    #[test]
    fn test_dsr_report_kind_udk_locked() {
        let kind = DsrReportKind::try_from_ansi(b"20").unwrap();
        assert_eq!(kind, DsrReportKind::Udk(UdkStatus::Locked));
    }

    #[test]
    fn test_dsr_report_kind_udk_unlocked() {
        let kind = DsrReportKind::try_from_ansi(b"21").unwrap();
        assert_eq!(kind, DsrReportKind::Udk(UdkStatus::Unlocked));
    }

    #[test]
    fn test_dsr_report_kind_keyboard() {
        let kind = DsrReportKind::try_from_ansi(b"27;1").unwrap();
        assert_eq!(kind, DsrReportKind::Keyboard { dialect: 1 });
        assert_eq!(kind.encode_ansi().unwrap(), b"27;1");
    }

    #[test]
    fn test_dsr_report_kind_no_locator() {
        let kind = DsrReportKind::try_from_ansi(b"50").unwrap();
        assert_eq!(kind, DsrReportKind::Locator(LocatorStatus::NoLocator));
    }

    #[test]
    fn test_dsr_report_kind_locator_available() {
        let kind = DsrReportKind::try_from_ansi(b"53").unwrap();
        assert_eq!(kind, DsrReportKind::Locator(LocatorStatus::Available));
    }

    #[test]
    fn test_dsr_report_kind_locator_type() {
        let kind = DsrReportKind::try_from_ansi(b"57;2").unwrap();
        assert_eq!(kind, DsrReportKind::LocatorType { locator_type: 2 });
    }

    #[test]
    fn test_dsr_report_kind_data_integrity_ready() {
        let kind = DsrReportKind::try_from_ansi(b"70").unwrap();
        assert_eq!(
            kind,
            DsrReportKind::DataIntegrity(DataIntegrityStatus::Ready)
        );
    }

    #[test]
    fn test_dsr_report_kind_data_integrity_malfunction() {
        let kind = DsrReportKind::try_from_ansi(b"71").unwrap();
        assert_eq!(
            kind,
            DsrReportKind::DataIntegrity(DataIntegrityStatus::Malfunction)
        );
    }

    #[test]
    fn test_dsr_report_kind_sessions_available() {
        let kind = DsrReportKind::try_from_ansi(b"80").unwrap();
        assert_eq!(
            kind,
            DsrReportKind::MultipleSession(MultipleSessionStatus::Available)
        );
    }

    #[test]
    fn test_dsr_report_kind_sessions_not_available() {
        let kind = DsrReportKind::try_from_ansi(b"81").unwrap();
        assert_eq!(
            kind,
            DsrReportKind::MultipleSession(MultipleSessionStatus::NotAvailable)
        );
    }

    #[test]
    fn test_dsr_report_kind_sessions_not_configured() {
        let kind = DsrReportKind::try_from_ansi(b"83").unwrap();
        assert_eq!(
            kind,
            DsrReportKind::MultipleSession(
                MultipleSessionStatus::NotConfigured
            )
        );
    }

    #[test]
    fn test_dsr_report_kind_macro_space() {
        let kind = DsrReportKind::try_from_ansi(b"62;1024").unwrap();
        assert_eq!(kind, DsrReportKind::MacroSpace { space: 1024 });
        assert_eq!(kind.encode_ansi().unwrap(), b"62;1024");
    }

    #[test]
    fn test_dsr_report_kind_memory_checksum() {
        let kind = DsrReportKind::try_from_ansi(b"63;1;65535").unwrap();
        assert_eq!(
            kind,
            DsrReportKind::MemoryChecksum {
                id: 1,
                checksum: 65535
            }
        );
        assert_eq!(kind.encode_ansi().unwrap(), b"63;1;65535");
    }

    #[test]
    fn test_dsr_report_kind_unknown() {
        let kind = DsrReportKind::try_from_ansi(b"99;1;2;3").unwrap();
        assert_eq!(
            kind,
            DsrReportKind::Unknown {
                status: 99,
                params: vec![1, 2, 3]
            }
        );
    }
}
