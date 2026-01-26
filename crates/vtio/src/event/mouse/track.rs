//! Mouse tracking event.

/// Track Mouse.
///
/// *Sequence*: `CSI Ps ; Ps ; Ps ; Ps ; Ps T`
///
/// This sequence is used with mouse highlight mode to communicate the
/// selection start and allowed rows.
///
/// If cmd is 0 then the highlighting is aborted and the terminal uses non
/// highlighting mouse handling as in mouse down+up tracking.
///
/// If cmd is non-zero then start-column and start-row specify the selection
/// start and first-row specifies the first allowed row for the selection and
/// last-row specifies the first row that the selection may not enter into.
///
/// See <https://terminalguide.namepad.de/seq/csi_ct_5param/> for terminal
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
#[vtansi(csi, finalbyte = 'T', disambiguate)]
pub struct TrackMouse {
    cmd: u8,
    start_column: u16,
    start_row: u16,
    first_row: u16,
    last_row: u16,
}

impl TrackMouse {
    /// Create a new `TrackMouse` sequence.
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command byte (0 to abort highlighting, non-zero to start)
    /// * `start_column` - Starting column for selection
    /// * `start_row` - Starting row for selection
    /// * `first_row` - First allowed row for selection
    /// * `last_row` - First row that selection may not enter
    #[must_use]
    pub const fn new(
        cmd: u8,
        start_column: u16,
        start_row: u16,
        first_row: u16,
        last_row: u16,
    ) -> Self {
        Self {
            cmd,
            start_column,
            start_row,
            first_row,
            last_row,
        }
    }

    /// Get the command byte.
    #[must_use]
    pub const fn cmd(&self) -> u8 {
        self.cmd
    }

    /// Get the start column.
    #[must_use]
    pub const fn start_column(&self) -> u16 {
        self.start_column
    }

    /// Get the start row.
    #[must_use]
    pub const fn start_row(&self) -> u16 {
        self.start_row
    }

    /// Get the first allowed row.
    #[must_use]
    pub const fn first_row(&self) -> u16 {
        self.first_row
    }

    /// Get the last row (first row selection may not enter).
    #[must_use]
    pub const fn last_row(&self) -> u16 {
        self.last_row
    }
}
