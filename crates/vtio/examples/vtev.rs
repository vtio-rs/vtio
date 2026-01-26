//! Interactive Terminal Input Event Debugger
//!
//! This example is a full-featured raw mode fullscreen application that helps
//! debug and understand terminal input events, including the kitty keyboard
//! protocol enhancements, mouse events, and terminal mode states.
//!
//! # Usage
//!
//! Run the example interactively:
//!   ```bash
//!   cargo run --example vtev -p vtio
//!   ```
//!
//! Run in non-interactive mode (for piped input):
//!   ```bash
//!   printf "\x1b[A\x1b[B" | cargo run --example vtev -p vtio
//!   cat input.txt | cargo run --example vtev -p vtio
//!   ```
//!
//! Force non-interactive mode even with a TTY:
//!   ```bash
//!   cargo run --example vtev -p vtio -- --non-interactive < input.txt
//!   ```
//!
//! The terminal will enter raw mode and display a tabbed full-screen interface:
//! - Tab 1: Key Events - Shows last key/mouse event, raw bytes, keyboard enhancement toggles,
//!   and mouse capture toggle
//! - Tab 2: Terminal State - Shows all terminal modes and their current states
//! - Tab 3: Event Log - Shows a log of all events received from stdin (decoded and raw bytes)
//!
//! All events are also written to `keys.log` in the current working directory.
//! Debug tracing is written to `debug.log` in the current working directory.
//!
//! In non-interactive mode (when stdin is not a TTY), the program simply reads
//! input, parses it, and prints each event to stdout before exiting.
//!
//! # Key Bindings (Interactive Mode)
//!
//! - `F1-F3`: Switch between tabs
//! - `1-5`: Toggle individual keyboard enhancement flags (Tab 1)
//! - `0`: Toggle all keyboard enhancements on/off (Tab 1)
//! - `6-9`: Toggle individual mouse modes (Tab 1):
//!   - `6`: DOWN_UP_TRACKING (1000) - button press/release and scroll
//!   - `7`: CLICK_DRAG_TRACKING (1002) - adds drag events
//!   - `8`: ANY_EVENT_TRACKING (1003) - adds motion events
//!   - `9`: SGR_FORMAT (1006) - extended coordinate format
//! - `m`: Toggle all mouse modes on/off (Tab 1)
//! - `Ctrl+C` or `Ctrl+D` or `q`: Exit
//!
//! Note: When `REPORT_EVENT_TYPES` is enabled, toggle actions only occur on
//! key release events to prevent double-toggling.
//!
//! # Command Line Options
//!
//! - `--non-interactive` or `-n`: Force non-interactive mode (useful for testing)
//! - `--quiet` or `-q`: Suppress stderr output in non-interactive mode (only print events)
//!
//! # What it demonstrates
//!
//! - Setting up terminal raw mode (no echo, no line buffering)
//! - Parsing key events as they arrive
//! - Toggling kitty keyboard protocol flags
//! - Enabling/disabling individual mouse tracking modes:
//!   - Down/Up tracking (button press/release, scroll wheel)
//!   - Click and drag tracking (motion while button pressed)
//!   - Any event tracking (all motion events)
//!   - SGR reporting format (extended coordinates)
//! - Querying terminal mode states using DECRQM
//! - Displaying detailed information about key events, mouse events, and terminal state
//! - Logging all events to a file for post-analysis
//! - Debug tracing for troubleshooting parsing and timing issues
//! - Proper cleanup on exit
//! - Non-interactive mode for automated testing

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::time::{Duration, Instant};
use vtansi::{AnsiEncode, TerseDebug};
use vtio::event::mode::TerminalModeState;
use vtio::event::mouse::{
    DisableMouseAnyEventTrackingMode, DisableMouseClickAndDragTrackingMode,
    DisableMouseDownUpTrackingMode, DisableMouseReportSgrMode,
    EnableMouseAnyEventTrackingMode, EnableMouseClickAndDragTrackingMode,
    EnableMouseDownUpTrackingMode, EnableMouseReportSgrMode, MouseEventKind,
};
use vtio::event::{
    KeyCode, KeyEvent, KeyModifiers, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use vtio::event::{cursor, keyboard, mouse, screen, scroll, terminal};
use vtio::{AnyEvent, TerminalInputParser};

#[cfg(unix)]
mod raw_mode {
    use std::io;
    use std::os::unix::io::AsRawFd;
    use std::time::Duration;

    pub struct RawModeGuard {
        original_termios: Option<libc::termios>,
    }

    impl RawModeGuard {
        pub fn new() -> io::Result<Self> {
            let fd = io::stdin().as_raw_fd();

            // Check if stdin is a TTY
            let is_tty = unsafe { libc::isatty(fd) == 1 };
            if !is_tty {
                // Not a TTY, don't enable raw mode
                tracing::info!("stdin is not a TTY, raw mode not enabled");
                return Ok(Self {
                    original_termios: None,
                });
            }

            tracing::info!("stdin is a TTY, enabling raw mode");

            // Get current terminal settings
            let original_termios = unsafe {
                let mut termios = std::mem::zeroed();
                if libc::tcgetattr(fd, &mut termios) != 0 {
                    return Err(io::Error::last_os_error());
                }
                termios
            };

            // Create modified settings for raw mode
            let mut raw_termios = original_termios;

            // Disable canonical mode (line buffering) and echo
            raw_termios.c_lflag &= !(libc::ICANON
                | libc::ECHO
                | libc::ECHONL
                | libc::IEXTEN
                | libc::ISIG);

            // Disable input processing
            raw_termios.c_iflag &= !(libc::IXON
                | libc::ICRNL
                | libc::INLCR
                | libc::IGNCR
                | libc::BRKINT
                | libc::INPCK
                | libc::ISTRIP);

            // Disable output processing
            raw_termios.c_oflag &= !(libc::OPOST);

            // Set character size to 8 bits
            raw_termios.c_cflag &= !(libc::CSIZE | libc::PARENB);
            raw_termios.c_cflag |= libc::CS8;

            // Set read to return immediately with at least 1 byte
            raw_termios.c_cc[libc::VMIN] = 1;
            raw_termios.c_cc[libc::VTIME] = 0;

            // Apply the raw mode settings
            unsafe {
                if libc::tcsetattr(fd, libc::TCSANOW, &raw_termios) != 0 {
                    return Err(io::Error::last_os_error());
                }
            }

            Ok(Self {
                original_termios: Some(original_termios),
            })
        }
    }

    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            if let Some(original_termios) = &self.original_termios {
                let fd = io::stdin().as_raw_fd();
                unsafe {
                    let _ =
                        libc::tcsetattr(fd, libc::TCSANOW, original_termios);
                }
            }
        }
    }

    /// Poll stdin for data with a timeout
    /// Returns true if data is available, false if timeout elapsed
    pub fn poll_stdin(timeout: Duration) -> io::Result<bool> {
        let fd = io::stdin().as_raw_fd();
        let timeout_ms = timeout.as_millis() as libc::c_int;

        tracing::debug!("poll_stdin: fd={}, timeout_ms={}", fd, timeout_ms);

        unsafe {
            let mut pollfd = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };

            let result = libc::poll(&mut pollfd, 1, timeout_ms);

            tracing::debug!(
                "poll_stdin: result={}, revents={:x}",
                result,
                pollfd.revents
            );

            if result < 0 {
                let err = io::Error::last_os_error();
                tracing::debug!("poll_stdin: error={}", err);
                Err(err)
            } else if result == 0 {
                // Timeout
                tracing::debug!("poll_stdin: timeout, no data available");
                Ok(false)
            } else {
                // Data available
                tracing::debug!("poll_stdin: data available (POLLIN set)");
                Ok(true)
            }
        }
    }
}

#[cfg(not(unix))]
mod raw_mode {
    use std::io;
    use std::time::Duration;

    pub struct RawModeGuard;

    impl RawModeGuard {
        pub fn new() -> io::Result<Self> {
            // On non-Unix systems, raw mode is not supported
            Ok(Self)
        }
    }

    /// Poll stdin for data with a timeout (stub for non-Unix)
    pub fn poll_stdin(_timeout: Duration) -> io::Result<bool> {
        // On non-Unix, just return true to attempt read
        Ok(true)
    }
}

// Macro to define terminal modes in a single place to avoid sync issues
// between init_modes() and query_terminal_modes()
macro_rules! define_terminal_modes {
    (
        $(
            $comment:literal =>
            $name:literal : $request:path
        ),* $(,)?
    ) => {
        fn get_mode_names() -> Vec<&'static str> {
            vec![
                $(
                    $name,
                )*
            ]
        }

        fn send_all_mode_queries(stdout: &mut impl Write) -> io::Result<()> {
            $(
                $request.encode_ansi_into(stdout)?;
            )*
            stdout.flush()
        }
    };
}

// Define all terminal modes in one place
define_terminal_modes! {
    // Cursor modes
    "Cursor modes" => "RelativeCursorOrigin": cursor::RequestRelativeCursorOriginMode,
    "Cursor modes" => "CursorBlinking": cursor::RequestCursorBlinking,
    "Cursor modes" => "CursorVisibility": cursor::RequestCursorVisibility,

    // Keyboard modes
    "Keyboard modes" => "KeyboardInputDisabled": keyboard::RequestKeyboardInputDisabledMode,
    "Keyboard modes" => "CursorKeys": keyboard::RequestCursorKeysMode,
    "Keyboard modes" => "HeldKeysRepeat": keyboard::RequestHeldKeysRepeatMode,
    "Keyboard modes" => "ApplicationKeypad": keyboard::RequestApplicationKeypadMode,
    "Keyboard modes" => "BackspaceSendsDelete": keyboard::RequestBackspaceSendsDeleteMode,
    "Keyboard modes" => "AltKeyHighBitSet": keyboard::RequestAltKeyHighBitSetMode,
    "Keyboard modes" => "IgnoreKeypadAppModeWhenNumlockActive": keyboard::RequestIgnoreKeypadApplicationModeOnNumlockMode,
    "Keyboard modes" => "AltKeySendsEscPrefix": keyboard::RequestAltKeySendsEscPrefixMode,
    "Keyboard modes" => "DeleteKeySendsDEL": keyboard::RequestDeleteKeySendsDELMode,
    "Keyboard modes" => "AdditionalModifierKeySendsEscPrefix": keyboard::RequestAdditionalModifierKeySendsEscPrefix,

    // Mouse modes
    "Mouse modes" => "MouseClickOnlyTracking": mouse::RequestMouseX10Mode,
    "Mouse modes" => "MouseDownUpTracking": mouse::RequestMouseDownUpTrackingMode,
    "Mouse modes" => "MouseHighlight": mouse::RequestMouseHighlightMode,
    "Mouse modes" => "MouseClickAndDragTracking": mouse::RequestMouseClickAndDragTrackingMode,
    "Mouse modes" => "MouseAnyEventTracking": mouse::RequestMouseAnyEventTrackingMode,
    "Mouse modes" => "MouseReportMultibyte": mouse::RequestMouseReportMultibyteMode,
    "Mouse modes" => "MouseReportSGR": mouse::RequestMouseReportSgrMode,
    "Mouse modes" => "MouseReportUrxvt": mouse::RequestMouseReportRxvtMode,
    "Mouse modes" => "MouseWheelToCursorKeys": mouse::RequestMouseWheelToCursorKeysMode,

    // Scroll modes
    "Scroll modes" => "SmoothScroll": scroll::RequestSmoothScrollMode,

    // Terminal modes
    "Terminal modes" => "Insert": terminal::RequestInsertMode,
    "Terminal modes" => "Echo": terminal::RequestEchoMode,
    "Terminal modes" => "Linefeed": terminal::RequestLinefeedMode,
    "Terminal modes" => "VT52": terminal::RequestVT52Mode,
    "Terminal modes" => "HundredThirtyTwoColumn": terminal::RequestHundredThirtyTwoColumnMode,
    "Terminal modes" => "EnableSupportForHundredThirtyTwoColumn": terminal::RequestEnableSupportForHundredThirtyTwoColumnMode,
    "Terminal modes" => "KeepScreenOnHundredThirtyTwoColumnChange": terminal::RequestKeepScreenOnHundredThirtyTwoColumnChangeMode,
    "Terminal modes" => "ReverseDisplayColors": terminal::RequestReverseDisplayColorsMode,
    "Terminal modes" => "Wraparound": terminal::RequestLineWraparoundMode,
    "Terminal modes" => "ScrollbarVisibility": terminal::RequestScrollbarVisibilityMode,
    "Terminal modes" => "AlternateScreen": terminal::RequestAlternateScreenBasicMode,
    "Terminal modes" => "AlternateScreenWithClearOnExit": terminal::RequestAlternateScreenClearOnExitMode,
    "Terminal modes" => "AlternateScreenWithCursorSaveAndClear": terminal::RequestAlternateScreenMode,
    "Terminal modes" => "ReportFocusChange": terminal::RequestReportFocusChangeMode,
    "Terminal modes" => "InhibitScrollOnApplicationOutput": terminal::RequestInhibitScrollOnApplicationOutputMode,
    "Terminal modes" => "ScrollOnKeyboardInput": terminal::RequestScrollOnKeyboardInputMode,
    "Terminal modes" => "BoldBlinkingCellsAreBright": terminal::RequestBoldBlinkingBrightMode,
    "Terminal modes" => "BracketedPaste": terminal::RequestBracketedPasteMode,
    "Terminal modes" => "SynchronizedUpdate": terminal::RequestSynchronizedUpdateMode,
}

struct TerseDebugWrapper<'a, T: TerseDebug>(&'a T);

impl<'a, T: TerseDebug> std::fmt::Display for TerseDebugWrapper<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.terse_fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    KeyEvents,
    TerminalState,
    EventLog,
}

struct EventLogEntry {
    decoded: String,
    raw_bytes: String,
    source: &'static str,
    timestamp_ms: u128,
}

struct EventLog {
    entries: Vec<EventLogEntry>,
    max_entries: usize,
    log_file: Option<BufWriter<File>>,
}

impl EventLog {
    fn new(max_entries: usize) -> Self {
        // Open keys.log for writing (overwrite if exists)
        let log_file = File::create("keys.log").ok().map(BufWriter::new);

        Self {
            entries: Vec::new(),
            max_entries,
            log_file,
        }
    }

    fn add_entry(
        &mut self,
        decoded: String,
        raw_bytes: String,
        source: &'static str,
        timestamp_ms: u128,
    ) {
        self.entries.push(EventLogEntry {
            decoded: decoded.clone(),
            raw_bytes: raw_bytes.clone(),
            source,
            timestamp_ms,
        });
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }

        // Write to log file
        if let Some(ref mut file) = self.log_file {
            let _ = writeln!(
                file,
                "[{source}] @ {timestamp_ms}ms\n  Decoded: {decoded}\n  Raw:     {raw_bytes}\n",
            );
            let _ = file.flush();
        }
    }
}

struct KeyboardState {
    flags: KeyboardEnhancementFlags,
}

impl KeyboardState {
    fn new() -> Self {
        Self {
            flags: KeyboardEnhancementFlags::empty(),
        }
    }

    fn toggle_flag(&mut self, flag: KeyboardEnhancementFlags) {
        self.flags.toggle(flag);
    }

    #[allow(dead_code)]
    fn set_flags(&mut self, flags: KeyboardEnhancementFlags) {
        self.flags = flags;
    }

    fn get_flags(&self) -> KeyboardEnhancementFlags {
        self.flags
    }

    fn has_flag(&self, flag: KeyboardEnhancementFlags) -> bool {
        self.flags.contains(flag)
    }

    fn toggle_all(&mut self) {
        if self.flags.is_empty() {
            // Enable all flags
            self.flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ASSOCIATED_TEXT;
        } else {
            // Disable all flags
            self.flags = KeyboardEnhancementFlags::empty();
        }
    }
}

bitflags::bitflags! {
    /// Mouse mode flags for tracking which mouse modes are enabled.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct MouseModeFlags: u8 {
        /// Mouse down+up tracking (1000) - reports button press/release and scroll
        const DOWN_UP_TRACKING = 0b0001;
        /// Mouse click and drag tracking (1002) - adds drag events
        const CLICK_DRAG_TRACKING = 0b0010;
        /// Mouse any event tracking (1003) - adds motion events
        const ANY_EVENT_TRACKING = 0b0100;
        /// SGR mouse reporting format (1006) - extended coordinate format
        const SGR_FORMAT = 0b1000;
    }
}

struct MouseState {
    flags: MouseModeFlags,
}

impl MouseState {
    fn new() -> Self {
        Self {
            flags: MouseModeFlags::empty(),
        }
    }

    fn toggle_flag(&mut self, flag: MouseModeFlags) {
        self.flags.toggle(flag);
    }

    #[allow(dead_code)]
    fn set_flags(&mut self, flags: MouseModeFlags) {
        self.flags = flags;
    }

    fn get_flags(&self) -> MouseModeFlags {
        self.flags
    }

    fn has_flag(&self, flag: MouseModeFlags) -> bool {
        self.flags.contains(flag)
    }

    fn is_any_enabled(&self) -> bool {
        !self.flags.is_empty()
    }

    /// Toggle all mouse modes on/off
    fn toggle_all(&mut self) {
        if self.flags.is_empty() {
            // Enable all common mouse modes for full capture
            self.flags = MouseModeFlags::DOWN_UP_TRACKING
                | MouseModeFlags::CLICK_DRAG_TRACKING
                | MouseModeFlags::ANY_EVENT_TRACKING
                | MouseModeFlags::SGR_FORMAT;
        } else {
            // Disable all mouse modes
            self.flags = MouseModeFlags::empty();
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ModeResponseState {
    Unknown,                      // Haven't received a response yet
    Ignored, // Terminal ignored the query (we received a later query response)
    NotSupported, // Terminal explicitly said not supported
    Supported(TerminalModeState), // Terminal supports it with this state
}

#[derive(Debug, Clone)]
struct TerminalModeStatus {
    state: ModeResponseState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartupState {
    Querying,
    WaitingForResponses,
    ShowingSummary,
    Complete,
}

struct TerminalState {
    modes: HashMap<String, TerminalModeStatus>,
    mode_order: Vec<String>, // Order in which modes were queried
    last_received_index: Option<usize>, // Index of last mode we received
    expected_responses: usize,
    received_responses: usize,
    startup_state: StartupState,
    query_start_time: Option<Instant>,
}

impl TerminalState {
    fn new() -> Self {
        Self {
            modes: HashMap::new(),
            mode_order: Vec::new(),
            last_received_index: None,
            expected_responses: 0,
            received_responses: 0,
            startup_state: StartupState::Querying,
            query_start_time: None,
        }
    }

    fn init_modes(&mut self) -> usize {
        // Initialize all known modes as "Unknown" until we get responses
        // Uses get_mode_names() which is generated by the define_terminal_modes! macro
        let mode_names = get_mode_names();

        let count = mode_names.len();
        for name in mode_names {
            self.mode_order.push(name.to_string());
            self.modes.insert(
                name.to_string(),
                TerminalModeStatus {
                    state: ModeResponseState::Unknown,
                },
            );
        }
        self.expected_responses = count;
        count
    }

    fn update_mode(&mut self, name: &str, state: TerminalModeState) {
        // Find the index of this mode in the query order
        if let Some(current_index) =
            self.mode_order.iter().position(|n| n == name)
        {
            // Mark any skipped modes as Ignored
            if let Some(last_idx) = self.last_received_index {
                for idx in (last_idx + 1)..current_index {
                    if let Some(skipped_name) = self.mode_order.get(idx)
                        && let Some(skipped_mode) =
                            self.modes.get_mut(skipped_name)
                        && skipped_mode.state == ModeResponseState::Unknown
                    {
                        skipped_mode.state = ModeResponseState::Ignored;
                        self.received_responses += 1;
                    }
                }
            }

            // Update this mode
            if let Some(mode) = self.modes.get_mut(name) {
                if mode.state == ModeResponseState::Unknown {
                    // This is a new response (not an update)
                    self.received_responses += 1;
                }
                mode.state = if state == TerminalModeState::NotRecognized {
                    ModeResponseState::NotSupported
                } else {
                    ModeResponseState::Supported(state)
                };
            }

            self.last_received_index = Some(current_index);
        }
    }

    fn all_responses_received(&self) -> bool {
        self.received_responses >= self.expected_responses
    }

    fn start_waiting_for_responses(&mut self) {
        self.startup_state = StartupState::WaitingForResponses;
        self.query_start_time = Some(Instant::now());
    }

    fn show_summary(&mut self) {
        self.startup_state = StartupState::ShowingSummary;
    }

    fn complete_startup(&mut self) {
        self.startup_state = StartupState::Complete;
    }

    fn is_startup_complete(&self) -> bool {
        self.startup_state == StartupState::Complete
    }

    fn is_showing_summary(&self) -> bool {
        self.startup_state == StartupState::ShowingSummary
    }

    fn finalize_remaining_modes(&mut self) {
        // Mark all remaining Unknown modes as Ignored (terminal never responded)
        for mode in self.modes.values_mut() {
            if mode.state == ModeResponseState::Unknown {
                mode.state = ModeResponseState::Ignored;
                self.received_responses += 1;
            }
        }
    }
}

fn handle_mode_response<'a>(
    terminal_state: &mut TerminalState,
    event: &dyn better_any::Tid<'a>,
) -> Option<String> {
    // Cursor modes
    if let Some(mode) = event.downcast_ref::<cursor::RelativeCursorOriginMode>() {
        terminal_state.update_mode("RelativeCursorOrigin", mode.state);
        Some(format!("RelativeCursorOrigin: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<cursor::CursorBlinking>() {
        terminal_state.update_mode("CursorBlinking", mode.state);
        Some(format!("CursorBlinking: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<cursor::CursorVisibility>() {
        terminal_state.update_mode("CursorVisibility", mode.state);
        Some(format!("CursorVisibility: {:?}", mode.state))
    }
    // Keyboard modes
    else if let Some(mode) = event.downcast_ref::<keyboard::KeyboardInputDisabledMode>() {
        terminal_state.update_mode("KeyboardInputDisabled", mode.state);
        Some(format!("KeyboardInputDisabled: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<keyboard::CursorKeysMode>() {
        terminal_state.update_mode("CursorKeys", mode.state);
        Some(format!("CursorKeys: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<keyboard::HeldKeysRepeatMode>() {
        terminal_state.update_mode("HeldKeysRepeat", mode.state);
        Some(format!("HeldKeysRepeat: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<keyboard::ApplicationKeypadMode>() {
        terminal_state.update_mode("ApplicationKeypad", mode.state);
        Some(format!("ApplicationKeypad: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<keyboard::BackspaceSendsDeleteMode>() {
        terminal_state.update_mode("BackspaceSendsDelete", mode.state);
        Some(format!("BackspaceSendsDelete: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<keyboard::AltKeyHighBitSetMode>() {
        terminal_state.update_mode("AltKeyHighBitSet", mode.state);
        Some(format!("AltKeyHighBitSet: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<keyboard::IgnoreKeypadApplicationModeOnNumlockMode>() {
        terminal_state.update_mode("IgnoreKeypadAppModeWhenNumlockActive", mode.state);
        Some(format!("IgnoreKeypadAppModeWhenNumlockActive: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<keyboard::AltKeySendsEscPrefixMode>() {
        terminal_state.update_mode("AltKeySendsEscPrefix", mode.state);
        Some(format!("AltKeySendsEscPrefix: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<keyboard::DeleteKeySendsDELMode>() {
        terminal_state.update_mode("DeleteKeySendsDEL", mode.state);
        Some(format!("DeleteKeySendsDEL: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<keyboard::AdditionalModifierKeySendsEscPrefix>() {
        terminal_state.update_mode("AdditionalModifierKeySendsEscPrefix", mode.state);
        Some(format!("AdditionalModifierKeySendsEscPrefix: {:?}", mode.state))
    }
    // Mouse modes
    else if let Some(mode) = event.downcast_ref::<mouse::MouseX10Mode>() {
        terminal_state.update_mode("MouseClickOnlyTracking", mode.state);
        Some(format!("MouseClickOnlyTracking: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<mouse::MouseDownUpTrackingMode>() {
        terminal_state.update_mode("MouseDownUpTracking", mode.state);
        Some(format!("MouseDownUpTracking: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<mouse::MouseHighlightMode>() {
        terminal_state.update_mode("MouseHighlight", mode.state);
        Some(format!("MouseHighlight: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<mouse::MouseClickAndDragTrackingMode>() {
        terminal_state.update_mode("MouseClickAndDragTracking", mode.state);
        Some(format!("MouseClickAndDragTracking: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<mouse::MouseAnyEventTrackingMode>() {
        terminal_state.update_mode("MouseAnyEventTracking", mode.state);
        Some(format!("MouseAnyEventTracking: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<mouse::MouseReportMultibyteMode>() {
        terminal_state.update_mode("MouseReportMultibyte", mode.state);
        Some(format!("MouseReportMultibyte: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<mouse::MouseReportSgrMode>() {
        terminal_state.update_mode("MouseReportSGR", mode.state);
        Some(format!("MouseReportSGR: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<mouse::MouseReportRxvtMode>() {
        terminal_state.update_mode("MouseReportUrxvt", mode.state);
        Some(format!("MouseReportUrxvt: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<mouse::MouseWheelToCursorKeysMode>() {
        terminal_state.update_mode("MouseWheelToCursorKeys", mode.state);
        Some(format!("MouseWheelToCursorKeys: {:?}", mode.state))
    }
    // Scroll modes
    else if let Some(mode) = event.downcast_ref::<scroll::SmoothScrollMode>() {
        terminal_state.update_mode("SmoothScroll", mode.state);
        Some(format!("SmoothScroll: {:?}", mode.state))
    }
    // Terminal modes
    else if let Some(mode) = event.downcast_ref::<terminal::InsertMode>() {
        terminal_state.update_mode("Insert", mode.state);
        Some(format!("Insert: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::EchoMode>() {
        terminal_state.update_mode("Echo", mode.state);
        Some(format!("Echo: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::LinefeedMode>() {
        terminal_state.update_mode("Linefeed", mode.state);
        Some(format!("Linefeed: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::VT52Mode>() {
        terminal_state.update_mode("VT52", mode.state);
        Some(format!("VT52: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::HundredThirtyTwoColumnMode>() {
        terminal_state.update_mode("HundredThirtyTwoColumn", mode.state);
        Some(format!("HundredThirtyTwoColumn: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::EnableSupportForHundredThirtyTwoColumnMode>() {
        terminal_state.update_mode("EnableSupportForHundredThirtyTwoColumn", mode.state);
        Some(format!("EnableSupportForHundredThirtyTwoColumn: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::KeepScreenOnHundredThirtyTwoColumnChangeMode>() {
        terminal_state.update_mode("KeepScreenOnHundredThirtyTwoColumnChange", mode.state);
        Some(format!("KeepScreenOnHundredThirtyTwoColumnChange: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::ReverseDisplayColorsMode>() {
        terminal_state.update_mode("ReverseDisplayColors", mode.state);
        Some(format!("ReverseDisplayColors: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::LineWraparoundMode>() {
        terminal_state.update_mode("Wraparound", mode.state);
        Some(format!("Wraparound: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::ScrollbarVisibilityMode>() {
        terminal_state.update_mode("ScrollbarVisibility", mode.state);
        Some(format!("ScrollbarVisibility: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::AlternateScreenBasicMode>() {
        terminal_state.update_mode("AlternateScreen", mode.state);
        Some(format!("AlternateScreen: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::AlternateScreenClearOnExitMode>() {
        terminal_state.update_mode("AlternateScreenWithClearOnExit", mode.state);
        Some(format!("AlternateScreenWithClearOnExit: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::AlternateScreenMode>() {
        terminal_state.update_mode("AlternateScreenWithCursorSaveAndClear", mode.state);
        Some(format!("AlternateScreenWithCursorSaveAndClear: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::ReportFocusChangeMode>() {
        terminal_state.update_mode("ReportFocusChange", mode.state);
        Some(format!("ReportFocusChange: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::InhibitScrollOnApplicationOutputMode>() {
        terminal_state.update_mode("InhibitScrollOnApplicationOutput", mode.state);
        Some(format!("InhibitScrollOnApplicationOutput: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::ScrollOnKeyboardInputMode>() {
        terminal_state.update_mode("ScrollOnKeyboardInput", mode.state);
        Some(format!("ScrollOnKeyboardInput: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::BoldBlinkingBrightMode>() {
        terminal_state.update_mode("BoldBlinkingCellsAreBright", mode.state);
        Some(format!("BoldBlinkingCellsAreBright: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::BracketedPasteMode>() {
        terminal_state.update_mode("BracketedPaste", mode.state);
        Some(format!("BracketedPaste: {:?}", mode.state))
    } else if let Some(mode) = event.downcast_ref::<terminal::SynchronizedUpdateMode>() {
        terminal_state.update_mode("SynchronizedUpdate", mode.state);
        Some(format!("SynchronizedUpdate: {:?}", mode.state))
    } else {
        None
    }
}

fn send_keyboard_enhancement_flags(
    stdout: &mut impl Write,
    flags: KeyboardEnhancementFlags,
) -> io::Result<()> {
    PopKeyboardEnhancementFlags.encode_ansi_into(stdout)?;

    if !flags.is_empty() {
        PushKeyboardEnhancementFlags(flags).encode_ansi_into(stdout)?;
    }

    stdout.flush()
}

fn send_mouse_mode_change(
    stdout: &mut impl Write,
    flag: MouseModeFlags,
    enable: bool,
) -> io::Result<()> {
    match flag {
        MouseModeFlags::DOWN_UP_TRACKING => {
            if enable {
                tracing::info!("Enabling MouseDownUpTrackingMode (1000)");
                EnableMouseDownUpTrackingMode.encode_ansi_into(stdout)?;
            } else {
                tracing::info!("Disabling MouseDownUpTrackingMode (1000)");
                DisableMouseDownUpTrackingMode.encode_ansi_into(stdout)?;
            }
        }
        MouseModeFlags::CLICK_DRAG_TRACKING => {
            if enable {
                tracing::info!("Enabling MouseClickAndDragTrackingMode (1002)");
                EnableMouseClickAndDragTrackingMode.encode_ansi_into(stdout)?;
            } else {
                tracing::info!(
                    "Disabling MouseClickAndDragTrackingMode (1002)"
                );
                DisableMouseClickAndDragTrackingMode
                    .encode_ansi_into(stdout)?;
            }
        }
        MouseModeFlags::ANY_EVENT_TRACKING => {
            if enable {
                tracing::info!("Enabling MouseAnyEventTrackingMode (1003)");
                EnableMouseAnyEventTrackingMode.encode_ansi_into(stdout)?;
            } else {
                tracing::info!("Disabling MouseAnyEventTrackingMode (1003)");
                DisableMouseAnyEventTrackingMode.encode_ansi_into(stdout)?;
            }
        }
        MouseModeFlags::SGR_FORMAT => {
            if enable {
                tracing::info!("Enabling MouseReportSgrMode (1006)");
                EnableMouseReportSgrMode.encode_ansi_into(stdout)?;
            } else {
                tracing::info!("Disabling MouseReportSgrMode (1006)");
                DisableMouseReportSgrMode.encode_ansi_into(stdout)?;
            }
        }
        _ => {
            // Handle combined flags by iterating
            for single_flag in [
                MouseModeFlags::DOWN_UP_TRACKING,
                MouseModeFlags::CLICK_DRAG_TRACKING,
                MouseModeFlags::ANY_EVENT_TRACKING,
                MouseModeFlags::SGR_FORMAT,
            ] {
                if flag.contains(single_flag) {
                    send_mouse_mode_change(stdout, single_flag, enable)?;
                }
            }
        }
    }
    stdout.flush()
}

fn sync_mouse_modes(
    stdout: &mut impl Write,
    old_flags: MouseModeFlags,
    new_flags: MouseModeFlags,
) -> io::Result<()> {
    // Disable modes that were on but are now off
    let to_disable = old_flags - new_flags;
    // Enable modes that were off but are now on
    let to_enable = new_flags - old_flags;

    // Disable first (in reverse order for proper cleanup)
    for flag in [
        MouseModeFlags::SGR_FORMAT,
        MouseModeFlags::ANY_EVENT_TRACKING,
        MouseModeFlags::CLICK_DRAG_TRACKING,
        MouseModeFlags::DOWN_UP_TRACKING,
    ] {
        if to_disable.contains(flag) {
            send_mouse_mode_change(stdout, flag, false)?;
        }
    }

    // Then enable (in forward order)
    for flag in [
        MouseModeFlags::DOWN_UP_TRACKING,
        MouseModeFlags::CLICK_DRAG_TRACKING,
        MouseModeFlags::ANY_EVENT_TRACKING,
        MouseModeFlags::SGR_FORMAT,
    ] {
        if to_enable.contains(flag) {
            send_mouse_mode_change(stdout, flag, true)?;
        }
    }

    Ok(())
}

fn enter_alternate_screen(stdout: &mut impl Write) -> io::Result<()> {
    terminal::EnableAlternateScreenMode.encode_ansi_into(stdout)?;
    cursor::DisableCursorVisibility.encode_ansi_into(stdout)?;
    stdout.flush()
}

fn exit_alternate_screen(stdout: &mut impl Write) -> io::Result<()> {
    terminal::DisableAlternateScreenMode.encode_ansi_into(stdout)?;
    stdout.flush()
}

fn clear_screen(stdout: &mut impl Write) -> io::Result<()> {
    screen::SelectiveEraseDisplayComplete.encode_ansi_into(stdout)?;
    cursor::SetCursorPosition::new().encode_ansi_into(stdout)?;
    stdout.flush()
}

fn draw_startup_ui(
    stdout: &mut impl Write,
    terminal_state: &TerminalState,
    elapsed_ms: u128,
) -> io::Result<()> {
    clear_screen(stdout)?;

    writeln!(
        stdout,
        "\r\n═══════════════════════════════════════════════════════════════════"
    )?;
    writeln!(stdout, "\r  TERMINAL INPUT EVENT DEBUGGER")?;
    writeln!(
        stdout,
        "\r═══════════════════════════════════════════════════════════════════"
    )?;
    writeln!(stdout, "\r")?;
    writeln!(stdout, "\r  ⏳ Initializing...")?;
    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r  Querying terminal capabilities and waiting for responses..."
    )?;
    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r  Responses received: {}/{}",
        terminal_state.received_responses, terminal_state.expected_responses
    )?;
    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r  Elapsed time: {:.1}s",
        elapsed_ms as f64 / 1000.0
    )?;
    writeln!(stdout, "\r")?;

    if elapsed_ms > 1000 {
        writeln!(
            stdout,
            "\r  Note: Some terminals may not respond to all queries."
        )?;
        writeln!(
            stdout,
            "\r  Will timeout after 5 seconds if responses are incomplete."
        )?;
    }

    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r═══════════════════════════════════════════════════════════════════"
    )?;

    stdout.flush()
}

fn draw_startup_summary(
    stdout: &mut impl Write,
    keyboard_state: &KeyboardState,
    terminal_state: &TerminalState,
    elapsed_ms: u128,
) -> io::Result<()> {
    clear_screen(stdout)?;

    // Title
    writeln!(
        stdout,
        "\r\n═══════════════════════════════════════════════════════════════════"
    )?;
    writeln!(stdout, "\r  TERMINAL INITIALIZATION COMPLETE")?;
    writeln!(
        stdout,
        "\r═══════════════════════════════════════════════════════════════════"
    )?;
    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r  ✓ Received {}/{} responses in {:.2}s",
        terminal_state.received_responses,
        terminal_state.expected_responses,
        elapsed_ms as f64 / 1000.0
    )?;

    let missing_count =
        terminal_state.expected_responses - terminal_state.received_responses;
    if missing_count > 0 {
        writeln!(
            stdout,
            "\r  ⚠ {missing_count} mode(s) did not respond (terminal may not support them)",
        )?;
    }
    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r═══════════════════════════════════════════════════════════════════"
    )?;
    writeln!(stdout, "\r")?;

    // Reuse the Terminal State tab display
    draw_terminal_state_tab(stdout, keyboard_state, terminal_state)?;

    // Footer with prompt
    writeln!(
        stdout,
        "\r═══════════════════════════════════════════════════════════════════"
    )?;
    writeln!(stdout, "\r")?;
    writeln!(stdout, "\r  Press any key to continue...")?;
    writeln!(stdout, "\r")?;

    stdout.flush()
}

#[allow(clippy::too_many_arguments)]
fn draw_ui(
    stdout: &mut impl Write,
    current_tab: Tab,
    keyboard_state: &KeyboardState,
    mouse_state: &MouseState,
    terminal_state: &TerminalState,
    event_log: &EventLog,
    last_event: &str,
    last_bytes: &str,
) -> io::Result<()> {
    clear_screen(stdout)?;

    // Title with tabs
    writeln!(
        stdout,
        "\r\n═══════════════════════════════════════════════════════════════════"
    )?;
    writeln!(stdout, "\r  TERMINAL INPUT EVENT DEBUGGER")?;
    writeln!(
        stdout,
        "\r═══════════════════════════════════════════════════════════════════"
    )?;

    // Tab navigation
    write!(stdout, "\r  ")?;
    if current_tab == Tab::KeyEvents {
        write!(stdout, "▶ [F1] Key Events  ")?;
    } else {
        write!(stdout, "  [F1] Key Events  ")?;
    }
    if current_tab == Tab::TerminalState {
        write!(stdout, "▶ [F2] Terminal State  ")?;
    } else {
        write!(stdout, "  [F2] Terminal State  ")?;
    }
    if current_tab == Tab::EventLog {
        write!(stdout, "▶ [F3] Event Log")?;
    } else {
        write!(stdout, "  [F3] Event Log")?;
    }
    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r═══════════════════════════════════════════════════════════════════"
    )?;
    writeln!(stdout, "\r")?;

    match current_tab {
        Tab::KeyEvents => draw_key_events_tab(
            stdout,
            keyboard_state,
            mouse_state,
            last_event,
            last_bytes,
        )?,
        Tab::TerminalState => {
            draw_terminal_state_tab(stdout, keyboard_state, terminal_state)?
        }
        Tab::EventLog => draw_event_log_tab(stdout, event_log)?,
    }

    stdout.flush()
}

fn draw_key_events_tab(
    stdout: &mut impl Write,
    state: &KeyboardState,
    mouse_state: &MouseState,
    last_event: &str,
    last_bytes: &str,
) -> io::Result<()> {
    // Keyboard Enhancement Flags Status
    writeln!(stdout, "\r  Keyboard Enhancement Flags:")?;
    writeln!(
        stdout,
        "\r  ───────────────────────────────────────────────────────────────"
    )?;
    writeln!(
        stdout,
        "\r    [1] DISAMBIGUATE_ESCAPE_CODES         {}",
        if state.has_flag(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES) {
            "[ON]"
        } else {
            "[OFF]"
        }
    )?;
    writeln!(
        stdout,
        "\r    [2] REPORT_EVENT_TYPES                {}",
        if state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES) {
            "[ON]"
        } else {
            "[OFF]"
        }
    )?;
    writeln!(
        stdout,
        "\r    [3] REPORT_ALTERNATE_KEYS             {}",
        if state.has_flag(KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS) {
            "[ON]"
        } else {
            "[OFF]"
        }
    )?;
    writeln!(
        stdout,
        "\r    [4] REPORT_ALL_KEYS_AS_ESCAPE_CODES   {}",
        if state
            .has_flag(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES)
        {
            "[ON]"
        } else {
            "[OFF]"
        }
    )?;
    writeln!(
        stdout,
        "\r    [5] REPORT_ASSOCIATED_TEXT            {}",
        if state.has_flag(KeyboardEnhancementFlags::REPORT_ASSOCIATED_TEXT) {
            "[ON]"
        } else {
            "[OFF]"
        }
    )?;
    writeln!(
        stdout,
        "\r    [0] Toggle All                        [Current: 0b{:08b}]",
        state.get_flags().bits()
    )?;
    writeln!(stdout, "\r")?;

    // Mouse Mode Flags Status
    writeln!(stdout, "\r  Mouse Mode Flags:")?;
    writeln!(
        stdout,
        "\r  ───────────────────────────────────────────────────────────────"
    )?;
    writeln!(
        stdout,
        "\r    [6] DOWN_UP_TRACKING (1000)           {}",
        if mouse_state.has_flag(MouseModeFlags::DOWN_UP_TRACKING) {
            "[ON]"
        } else {
            "[OFF]"
        }
    )?;
    writeln!(
        stdout,
        "\r    [7] CLICK_DRAG_TRACKING (1002)        {}",
        if mouse_state.has_flag(MouseModeFlags::CLICK_DRAG_TRACKING) {
            "[ON]"
        } else {
            "[OFF]"
        }
    )?;
    writeln!(
        stdout,
        "\r    [8] ANY_EVENT_TRACKING (1003)         {}",
        if mouse_state.has_flag(MouseModeFlags::ANY_EVENT_TRACKING) {
            "[ON]"
        } else {
            "[OFF]"
        }
    )?;
    writeln!(
        stdout,
        "\r    [9] SGR_FORMAT (1006)                 {}",
        if mouse_state.has_flag(MouseModeFlags::SGR_FORMAT) {
            "[ON]"
        } else {
            "[OFF]"
        }
    )?;
    writeln!(
        stdout,
        "\r    [m] Toggle All Mouse Modes            [Current: 0b{:04b}]",
        mouse_state.get_flags().bits()
    )?;
    writeln!(stdout, "\r")?;

    // Last Event Display
    writeln!(stdout, "\r  Last Event:")?;
    writeln!(
        stdout,
        "\r  ───────────────────────────────────────────────────────────────"
    )?;
    writeln!(stdout, "\r    {last_event}")?;
    writeln!(stdout, "\r")?;

    // Raw Bytes Display
    writeln!(stdout, "\r  Raw Bytes:")?;
    writeln!(
        stdout,
        "\r  ───────────────────────────────────────────────────────────────"
    )?;
    writeln!(stdout, "\r    {last_bytes}")?;
    writeln!(stdout, "\r")?;

    // Help
    writeln!(stdout, "\r  Controls:")?;
    writeln!(
        stdout,
        "\r  ───────────────────────────────────────────────────────────────"
    )?;
    writeln!(stdout, "\r    [F1-F3] Switch tabs")?;
    writeln!(stdout, "\r    [1-5]   Toggle keyboard enhancement flags")?;
    writeln!(stdout, "\r    [0]     Toggle all keyboard enhancements")?;
    writeln!(stdout, "\r    [6-9]   Toggle individual mouse modes")?;
    writeln!(stdout, "\r    [m]     Toggle all mouse modes")?;
    writeln!(stdout, "\r    [q]     Quit (or Ctrl+C, Ctrl+D)")?;
    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r═══════════════════════════════════════════════════════════════════"
    )?;

    Ok(())
}

fn draw_terminal_state_tab(
    stdout: &mut impl Write,
    keyboard_state: &KeyboardState,
    terminal_state: &TerminalState,
) -> io::Result<()> {
    // Keyboard Enhancement Flags Status (compact)
    writeln!(stdout, "\r  Keyboard Enhancement Flags:")?;
    writeln!(
        stdout,
        "\r    [1]DisambiguateEscapeCodes:{} [2]ReportEventTypes:{} [3]ReportAlternateKeys:{}",
        if keyboard_state
            .has_flag(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        {
            "✓"
        } else {
            "✗"
        },
        if keyboard_state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
        {
            "✓"
        } else {
            "✗"
        },
        if keyboard_state
            .has_flag(KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS)
        {
            "✓"
        } else {
            "✗"
        }
    )?;
    writeln!(
        stdout,
        "\r    [4]ReportAllKeysAsEscapeCodes:{} [5]ReportAssociatedText:{} (bits:0b{:05b})",
        if keyboard_state
            .has_flag(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES)
        {
            "✓"
        } else {
            "✗"
        },
        if keyboard_state
            .has_flag(KeyboardEnhancementFlags::REPORT_ASSOCIATED_TEXT)
        {
            "✓"
        } else {
            "✗"
        },
        keyboard_state.get_flags().bits()
    )?;
    writeln!(stdout, "\r")?;

    // Terminal Modes (DECRQM/DECRPM) - Compact multi-column layout
    writeln!(
        stdout,
        "\r  Terminal Modes (✓=Set ✗=Reset ?=Unknown -=Ignored ~=NotSupported !=PermSet @=PermReset):"
    )?;
    writeln!(
        stdout,
        "\r  ───────────────────────────────────────────────────────────────"
    )?;

    // Helper to format mode state as single char
    let state_char = |mode: &TerminalModeStatus| -> &str {
        match mode.state {
            ModeResponseState::Unknown => "?",
            ModeResponseState::Ignored => "-",
            ModeResponseState::NotSupported => "~",
            ModeResponseState::Supported(TerminalModeState::Set) => "✓",
            ModeResponseState::Supported(TerminalModeState::Reset) => "✗",
            ModeResponseState::Supported(TerminalModeState::PermanentlySet) => {
                "!"
            }
            ModeResponseState::Supported(
                TerminalModeState::PermanentlyReset,
            ) => "@",
            ModeResponseState::Supported(TerminalModeState::NotRecognized) => {
                "~"
            }
        }
    };

    // Display modes in compact 2-column layout by category
    writeln!(
        stdout,
        "\r  Cursor:   RelativeCursorOrigin:{} CursorBlinking:{} CursorVisibility:{}",
        terminal_state
            .modes
            .get("RelativeCursorOrigin")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("CursorBlinking")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("CursorVisibility")
            .map(state_char)
            .unwrap_or("?"),
    )?;

    writeln!(stdout, "\r")?;

    writeln!(
        stdout,
        "\r  Keyboard: InputDisabled:{} CursorKeys:{} HeldKeysRepeat:{}",
        terminal_state
            .modes
            .get("KeyboardInputDisabled")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("CursorKeys")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("HeldKeysRepeat")
            .map(state_char)
            .unwrap_or("?"),
    )?;
    writeln!(
        stdout,
        "\r            ApplicationKeypad:{} BackspaceSendsDelete:{} AltKeyHighBitSet:{}",
        terminal_state
            .modes
            .get("ApplicationKeypad")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("BackspaceSendsDelete")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("AltKeyHighBitSet")
            .map(state_char)
            .unwrap_or("?"),
    )?;
    writeln!(
        stdout,
        "\r            AltKeySendsEscPrefix:{} DeleteKeySendsDEL:{}",
        terminal_state
            .modes
            .get("AltKeySendsEscPrefix")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("DeleteKeySendsDEL")
            .map(state_char)
            .unwrap_or("?"),
    )?;

    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r  Mouse:    X10Mode:{} DownUpTracking:{} Highlight:{} ClickAndDrag:{}",
        terminal_state
            .modes
            .get("MouseClickOnlyTracking")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("MouseDownUpTracking")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("MouseHighlight")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("MouseClickAndDragTracking")
            .map(state_char)
            .unwrap_or("?"),
    )?;
    writeln!(
        stdout,
        "\r            AnyEventTracking:{} ReportMultibyte:{} ReportSGR:{}",
        terminal_state
            .modes
            .get("MouseAnyEventTracking")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("MouseReportMultibyte")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("MouseReportSGR")
            .map(state_char)
            .unwrap_or("?"),
    )?;
    writeln!(
        stdout,
        "\r            ReportRxvt:{} WheelToCursorKeys:{}",
        terminal_state
            .modes
            .get("MouseReportUrxvt")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("MouseWheelToCursorKeys")
            .map(state_char)
            .unwrap_or("?"),
    )?;

    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r  Terminal: InsertMode:{} EchoMode:{} LinefeedMode:{} VT52Mode:{}",
        terminal_state
            .modes
            .get("Insert")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("Echo")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("Linefeed")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("VT52")
            .map(state_char)
            .unwrap_or("?"),
    )?;
    writeln!(
        stdout,
        "\r            Wraparound:{} ReverseDisplayColors:{} SmoothScroll:{}",
        terminal_state
            .modes
            .get("Wraparound")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("ReverseDisplayColors")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("SmoothScroll")
            .map(state_char)
            .unwrap_or("?"),
    )?;
    writeln!(
        stdout,
        "\r            AlternateScreen:{} AltScreenClearOnExit:{}",
        terminal_state
            .modes
            .get("AlternateScreen")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("AlternateScreenWithClearOnExit")
            .map(state_char)
            .unwrap_or("?"),
    )?;
    writeln!(
        stdout,
        "\r            BracketedPaste:{} SynchronizedUpdate:{} ReportFocusChange:{}",
        terminal_state
            .modes
            .get("BracketedPaste")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("SynchronizedUpdate")
            .map(state_char)
            .unwrap_or("?"),
        terminal_state
            .modes
            .get("ReportFocusChange")
            .map(state_char)
            .unwrap_or("?"),
    )?;
    writeln!(stdout, "\r")?;

    // Help
    writeln!(stdout, "\r  Controls:")?;
    writeln!(
        stdout,
        "\r  ───────────────────────────────────────────────────────────────"
    )?;
    writeln!(stdout, "\r    [F1-F3] Switch tabs")?;
    writeln!(stdout, "\r    [q]     Quit (or Ctrl+C, Ctrl+D)")?;
    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r═══════════════════════════════════════════════════════════════════"
    )?;

    Ok(())
}

fn draw_event_log_tab(
    stdout: &mut impl Write,
    event_log: &EventLog,
) -> io::Result<()> {
    writeln!(
        stdout,
        "\r  Event Log (last {} events):",
        event_log.max_entries
    )?;
    writeln!(
        stdout,
        "\r  ───────────────────────────────────────────────────────────────"
    )?;
    writeln!(stdout, "\r")?;

    if event_log.entries.is_empty() {
        writeln!(stdout, "\r    (No events logged yet)")?;
    } else {
        // Display events in reverse order (most recent first)
        for (idx, entry) in event_log.entries.iter().rev().enumerate() {
            let entry_num = event_log.entries.len() - idx;
            writeln!(
                stdout,
                "\r  #{}:  [{}] @ {}ms",
                entry_num, entry.source, entry.timestamp_ms
            )?;
            writeln!(stdout, "\r    Decoded: {}", entry.decoded)?;
            writeln!(stdout, "\r    Raw:     {}", entry.raw_bytes)?;
            writeln!(stdout, "\r")?;
        }
    }

    writeln!(stdout, "\r")?;

    // Help
    writeln!(stdout, "\r  Controls:")?;
    writeln!(
        stdout,
        "\r  ───────────────────────────────────────────────────────────────"
    )?;
    writeln!(stdout, "\r    [F1-F3] Switch tabs")?;
    writeln!(stdout, "\r    [q]     Quit (or Ctrl+C, Ctrl+D)")?;
    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r  Note: Events logged to keys.log, debug to debug.log"
    )?;
    writeln!(stdout, "\r")?;
    writeln!(
        stdout,
        "\r═══════════════════════════════════════════════════════════════════"
    )?;

    Ok(())
}

fn query_terminal_modes(stdout: &mut impl Write) -> io::Result<()> {
    // Uses send_all_mode_queries() which is generated by the define_terminal_modes! macro
    // This ensures the queries stay in sync with the mode names in init_modes()
    send_all_mode_queries(stdout)?;

    // Send a cursor position report request as a terminator.
    // This allows us to detect when the terminal has finished responding to mode queries
    // (or has ignored them) without waiting for the full timeout.
    // CPR is universally supported, even by terminals that don't support DECRQM.
    vtio::event::cursor::RequestCursorPosition.encode_ansi_into(stdout)?;

    stdout.flush()
}

fn format_key_event(key_event: &KeyEvent) -> String {
    let mut parts = vec![TerseDebugWrapper(key_event).to_string()];

    // Add detailed breakdown
    let mut details = Vec::new();

    // Key code
    details.push(format!("code: {:?}", key_event.code));

    // Modifiers
    if !key_event.modifiers.is_empty() {
        details.push(format!("modifiers: {:?}", key_event.modifiers));
    }

    // Kind
    details.push(format!("kind: {:?}", key_event.kind));

    // State
    if !key_event.state.is_empty() {
        details.push(format!("state: {:?}", key_event.state));
    }

    // Base layout key
    if let Some(base_key) = &key_event.base_layout_key {
        details.push(format!("base_layout_key: {base_key:?}"));
    }

    // Associated text
    if let Some(text) = &key_event.text {
        details.push(format!("text: {text:?}"));
    }

    parts.push(format!("    Details: {}", details.join(", ")));

    parts.join("\n\r    ")
}

fn format_bytes(bytes: &[u8]) -> String {
    let hex: Vec<String> = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let ascii: String = bytes
        .iter()
        .map(|&b| {
            if (0x20..=0x7e).contains(&b) {
                b as char
            } else {
                '·'
            }
        })
        .collect();

    format!(
        "{} bytes: [{}]  ASCII: \"{}\"",
        bytes.len(),
        hex.join(" "),
        ascii
    )
}

/// Check if both stdin and stdout are TTYs (interactive mode)
fn is_interactive() -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let stdin_tty = unsafe { libc::isatty(io::stdin().as_raw_fd()) == 1 };
        let stdout_tty = unsafe { libc::isatty(io::stdout().as_raw_fd()) == 1 };
        stdin_tty && stdout_tty
    }
    #[cfg(not(unix))]
    {
        // On non-Unix, assume interactive
        true
    }
}

/// Run in non-interactive mode: read all input, parse it, and print events
fn run_non_interactive(quiet: bool) -> io::Result<()> {
    if !quiet {
        eprintln!(
            "Running in non-interactive mode (stdin/stdout not connected to TTY)"
        );
    }

    let mut parser = TerminalInputParser::new();
    let mut stdin = io::stdin();
    let mut buffer = [0u8; 4096];
    let mut event_count = 0;

    loop {
        let n = match stdin.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };

        let input = &buffer[..n];
        if !quiet {
            eprintln!("Read {n} bytes: {input:02x?}");
        }

        parser.feed_with(input, &mut |event| {
            event_count += 1;

            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                let formatted = format_key_event(key_event);
                println!("KeyEvent #{event_count}: {formatted}");
            } else if let Some(mouse_event) =
                event.downcast_ref::<vtio::event::MouseEvent>()
            {
                let formatted = format_mouse_event(mouse_event);
                println!("MouseEvent #{event_count}: {formatted}");
            } else {
                // Handle terminal mode events and other events
                let type_name = std::any::type_name_of_val(event);
                if type_name.contains("Mode") {
                    println!("TerminalMode #{event_count}: {type_name}");
                } else {
                    println!("Event #{event_count}: {type_name}");
                }
            }
        });
    }

    // Call idle() to flush any remaining buffered sequences
    parser.idle(&mut |event| {
        event_count += 1;

        if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
            let formatted = format_key_event(key_event);
            println!("KeyEvent #{event_count} (from idle): {formatted}");
        } else if let Some(mouse_event) =
            event.downcast_ref::<vtio::event::MouseEvent>()
        {
            let formatted = format_mouse_event(mouse_event);
            println!("MouseEvent #{event_count} (from idle): {formatted}");
        } else {
            let type_name = std::any::type_name_of_val(event);
            if type_name.contains("Mode") {
                println!(
                    "TerminalMode #{event_count} (from idle): {type_name}"
                );
            } else {
                println!("Event #{event_count} (from idle): {type_name}");
            }
        }
    });

    if !quiet {
        eprintln!("Processed {event_count} total events");
    }
    Ok(())
}

fn main() -> io::Result<()> {
    // Check for command-line flags
    let args: Vec<String> = std::env::args().collect();
    let force_non_interactive = args
        .iter()
        .any(|arg| arg == "--non-interactive" || arg == "-n");
    let quiet = args.iter().any(|arg| arg == "--quiet" || arg == "-q");

    // Check if we're in interactive mode
    if force_non_interactive || !is_interactive() {
        return run_non_interactive(quiet);
    }

    // Interactive mode - proceed with full UI
    // Initialize tracing to debug.log
    let debug_file =
        File::create("debug.log").expect("Failed to create debug.log");
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::sync::Mutex::new(debug_file))
        .with_ansi(false)
        .init();

    tracing::info!("=== vtev started ===");

    let mut stdout = io::stdout();
    let mut keyboard_state = KeyboardState::new();
    let mut mouse_state = MouseState::new();
    let mut terminal_state = TerminalState::new();
    let expected_mode_count = terminal_state.init_modes();
    tracing::info!(
        "Initialized terminal state, expecting {} mode responses",
        expected_mode_count
    );
    let mut current_tab = Tab::KeyEvents;
    let mut last_event = String::from("(waiting for input...)");
    let mut last_bytes = String::from("(none)");
    let mut event_log = EventLog::new(100);

    // Enter raw mode
    let _guard = raw_mode::RawModeGuard::new()?;

    enter_alternate_screen(&mut stdout)?;

    // Query all terminal modes
    query_terminal_modes(&mut stdout)?;
    terminal_state.start_waiting_for_responses();

    // Show startup UI
    let startup_time = Instant::now();
    draw_startup_ui(&mut stdout, &terminal_state, 0)?;

    let mut parser = TerminalInputParser::new();
    let mut stdin = io::stdin();
    let mut buffer = [0u8; 1024];

    // Timeout tracking for idle() calls
    // Use 150ms to allow time for slow status report responses from terminal
    // This must be long enough that multi-sequence responses (like status reports)
    // can fully arrive before idle() is called, which would corrupt parser state
    let idle_timeout = Duration::from_millis(150);
    let mut last_input_time: Option<Instant> = None;
    let mut pending_idle = false;

    // Maximum time to wait for all terminal responses before giving up
    let max_startup_wait = Duration::from_millis(5000);
    let mut last_startup_ui_update = Instant::now();

    loop {
        // Handle startup phase - wait for all terminal mode responses
        if !terminal_state.is_startup_complete() {
            let elapsed = startup_time.elapsed();

            if terminal_state.is_showing_summary() {
                // We're showing the summary - wait for any key press
                // This is handled in the feed_with event processing below
            } else {
                // Check if we should show summary
                let should_show_summary = terminal_state
                    .all_responses_received()
                    || elapsed >= max_startup_wait;

                if should_show_summary {
                    // Mark any remaining unknown modes as ignored before finalizing
                    if !terminal_state.all_responses_received() {
                        terminal_state.finalize_remaining_modes();
                    }

                    // Skip summary screen if all responses received in under 500ms
                    let query_elapsed = terminal_state
                        .query_start_time
                        .map(|t| t.elapsed())
                        .unwrap_or(Duration::ZERO);

                    if terminal_state.all_responses_received()
                        && query_elapsed < Duration::from_millis(500)
                    {
                        terminal_state.complete_startup();
                        tracing::info!(
                            "Skipping summary screen: all {}/{} responses received in {:.1}ms",
                            terminal_state.received_responses,
                            terminal_state.expected_responses,
                            query_elapsed.as_secs_f64() * 1000.0
                        );

                        // Draw the normal UI for the first time
                        draw_ui(
                            &mut stdout,
                            current_tab,
                            &keyboard_state,
                            &mouse_state,
                            &terminal_state,
                            &event_log,
                            &last_event,
                            &last_bytes,
                        )?;
                    } else {
                        terminal_state.show_summary();
                        tracing::info!(
                            "Showing startup summary: received {}/{} responses in {:.1}s",
                            terminal_state.received_responses,
                            terminal_state.expected_responses,
                            elapsed.as_secs_f64()
                        );

                        // Draw the summary screen
                        draw_startup_summary(
                            &mut stdout,
                            &keyboard_state,
                            &terminal_state,
                            elapsed.as_millis(),
                        )?;
                    }
                } else {
                    // Update startup UI every 100ms to show progress
                    if last_startup_ui_update.elapsed()
                        >= Duration::from_millis(100)
                    {
                        draw_startup_ui(
                            &mut stdout,
                            &terminal_state,
                            elapsed.as_millis(),
                        )?;
                        last_startup_ui_update = Instant::now();
                    }
                }
            }
        }

        // Check if we should call idle() due to timeout
        // But suppress idle() calls during startup to allow status reports to arrive
        let startup_suppress = !terminal_state.is_startup_complete();

        if !startup_suppress
            && pending_idle
            && let Some(last_time) = last_input_time
            && last_time.elapsed() >= idle_timeout
        {
            tracing::info!(
                "Calling idle() after {}ms timeout (startup_suppress={})",
                last_time.elapsed().as_millis(),
                startup_suppress
            );
            let mut idle_redraw = false;
            let mut idle_event_count = 0;
            parser.idle(&mut |event| {
                idle_event_count += 1;
                tracing::debug!(
                    "Event #{} from idle(): type={}",
                    idle_event_count,
                    std::any::type_name_of_val(event)
                );
                if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                    tracing::trace!("KeyEvent from idle: {:?}", key_event);
                    last_event = format_key_event(key_event);
                    event_log.add_entry(
                        last_event.clone(),
                        last_bytes.clone(),
                        "idle()",
                        startup_time.elapsed().as_millis(),
                    );
                    idle_redraw = true;
                } else if let Some(mouse_event) =
                    event.downcast_ref::<vtio::event::MouseEvent>()
                {
                    tracing::trace!("MouseEvent from idle: {:?}", mouse_event);
                    last_event = format_mouse_event(mouse_event);
                    event_log.add_entry(
                        last_event.clone(),
                        last_bytes.clone(),
                        "idle()",
                        startup_time.elapsed().as_millis(),
                    );
                    idle_redraw = true;
                } else {
                    // Check if it's a terminal mode response and log it
                    tracing::trace!(
                        "Other event type from idle: {}",
                        std::any::type_name_of_val(event)
                    );
                    if let Some(mode_description) =
                        handle_mode_response(&mut terminal_state, event)
                    {
                        tracing::debug!(
                            "Terminal mode response from idle: {}",
                            mode_description
                        );
                        event_log.add_entry(
                            mode_description,
                            last_bytes.clone(),
                            "idle()",
                            startup_time.elapsed().as_millis(),
                        );
                    }
                }
            });
            tracing::debug!(
                "Completed idle(), processed {} events",
                idle_event_count
            );
            pending_idle = false;

            if idle_redraw {
                draw_ui(
                    &mut stdout,
                    current_tab,
                    &keyboard_state,
                    &mouse_state,
                    &terminal_state,
                    &event_log,
                    &last_event,
                    &last_bytes,
                )?;
            }
        }

        // Calculate remaining time until idle timeout
        let poll_timeout = if pending_idle {
            if let Some(last_time) = last_input_time {
                let elapsed = last_time.elapsed();
                if elapsed >= idle_timeout {
                    Duration::from_millis(100)
                } else {
                    idle_timeout - elapsed
                }
            } else {
                Duration::from_millis(100)
            }
        } else {
            Duration::from_millis(100)
        };

        // Poll stdin with timeout
        tracing::debug!(
            "Polling stdin with timeout: {}ms, pending_idle={}, startup_suppress={}",
            poll_timeout.as_millis(),
            pending_idle,
            startup_suppress
        );
        let data_available = raw_mode::poll_stdin(poll_timeout)?;
        tracing::debug!("Poll result: data_available={}", data_available);

        if !data_available {
            // Timeout - continue to check idle condition at top of loop
            tracing::debug!(
                "No data available, continuing to check idle condition"
            );
            continue;
        }

        // Read from stdin (blocking, but we know data is available)
        tracing::debug!("Reading from stdin...");
        let n = match stdin.read(&mut buffer) {
            Ok(0) => {
                // EOF (Ctrl+D)
                tracing::info!("EOF received, exiting");
                break;
            }
            Ok(n) => {
                tracing::debug!("Read {} bytes from stdin", n);
                n
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                tracing::debug!("Read interrupted, continuing");
                continue;
            }
            Err(e) => {
                tracing::warn!("Read error: {}", e);
                return Err(e);
            }
        };

        let input = &buffer[..n];
        last_bytes = format_bytes(input);
        tracing::debug!("Read data: {} bytes: {:02x?}", n, input);

        // Log raw bytes at info level when mouse capture is enabled to help debug
        if mouse_state.is_any_enabled() {
            tracing::info!(
                "Mouse capture ON - received {} bytes: {:02x?} (ASCII: {:?})",
                n,
                input,
                String::from_utf8_lossy(input)
            );
        }

        // Update timeout tracking - reset timer on new input
        last_input_time = Some(Instant::now());

        // Parse the input
        let mut should_exit = false;
        let mut should_redraw = false;
        let mut flags_changed = false;
        let old_mouse_flags = mouse_state.get_flags();

        tracing::debug!(
            "Starting parser.feed_with() with {} bytes",
            input.len()
        );
        let mut event_count = 0;
        parser.feed_with(input, &mut |event| {
            event_count += 1;
            tracing::debug!(
                "Event #{} from feed_with: type={}",
                event_count,
                std::any::type_name_of_val(event)
            );
            if let Some(key_event) = event.downcast_ref::<KeyEvent>() {
                tracing::trace!("KeyEvent: {:?}", key_event);

                // If we're showing the startup summary, any key press completes startup
                if terminal_state.is_showing_summary() {
                    terminal_state.complete_startup();
                    tracing::info!("User pressed key to exit startup summary");

                    // Draw the normal UI for the first time
                    draw_ui(
                        &mut stdout,
                        current_tab,
                        &keyboard_state,
                        &mouse_state,
                        &terminal_state,
                        &event_log,
                        &last_event,
                        &last_bytes,
                    ).ok();
                    return;
                }

                // Check for exit keys (Ctrl+C, Ctrl+D, or 'q')
                if (key_event.modifiers.contains(KeyModifiers::CONTROL)
                    && matches!(
                        key_event.code,
                        KeyCode::Char('c') | KeyCode::Char('d')
                    ))
                    || (key_event.code == KeyCode::Char('q')
                        && key_event.modifiers.is_empty())
                {
                    should_exit = true;
                    return;
                }

                // Handle tab switching and keyboard enhancement flag toggling
                match key_event.code {
                    KeyCode::F(1) => {
                        current_tab = Tab::KeyEvents;
                    }
                    KeyCode::F(2) => {
                        current_tab = Tab::TerminalState;
                    }
                    KeyCode::F(3) => {
                        current_tab = Tab::EventLog;
                    }
                    // Handle keyboard enhancement flag toggling (only on Key Events tab)
                    // Handle keyboard/mouse mode toggles
                    // When REPORT_EVENT_TYPES is enabled, only toggle on Release events
                    // to avoid double-toggling (once on press, once on release)
                    KeyCode::Char('0') if current_tab == Tab::KeyEvents => {
                        let should_toggle = !keyboard_state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
                            || key_event.is_release();
                        if should_toggle {
                            keyboard_state.toggle_all();
                            flags_changed = true;
                        }
                    }
                    KeyCode::Char('1') if current_tab == Tab::KeyEvents => {
                        let should_toggle = !keyboard_state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
                            || key_event.is_release();
                        if should_toggle {
                            keyboard_state.toggle_flag(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES);
                            flags_changed = true;
                        }
                    }
                    KeyCode::Char('2') if current_tab == Tab::KeyEvents => {
                        let should_toggle = !keyboard_state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
                            || key_event.is_release();
                        if should_toggle {
                            keyboard_state.toggle_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES);
                            flags_changed = true;
                        }
                    }
                    KeyCode::Char('3') if current_tab == Tab::KeyEvents => {
                        let should_toggle = !keyboard_state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
                            || key_event.is_release();
                        if should_toggle {
                            keyboard_state.toggle_flag(KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS);
                            flags_changed = true;
                        }
                    }
                    KeyCode::Char('4') if current_tab == Tab::KeyEvents => {
                        let should_toggle = !keyboard_state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
                            || key_event.is_release();
                        if should_toggle {
                            keyboard_state.toggle_flag(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES);
                            flags_changed = true;
                        }
                    }
                    KeyCode::Char('5') if current_tab == Tab::KeyEvents => {
                        let should_toggle = !keyboard_state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
                            || key_event.is_release();
                        if should_toggle {
                            keyboard_state.toggle_flag(KeyboardEnhancementFlags::REPORT_ASSOCIATED_TEXT);
                            flags_changed = true;
                        }
                    }
                    KeyCode::Char('6') if current_tab == Tab::KeyEvents => {
                        let should_toggle = !keyboard_state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
                            || key_event.is_release();
                        if should_toggle {
                            mouse_state.toggle_flag(MouseModeFlags::DOWN_UP_TRACKING);
                        }
                    }
                    KeyCode::Char('7') if current_tab == Tab::KeyEvents => {
                        let should_toggle = !keyboard_state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
                            || key_event.is_release();
                        if should_toggle {
                            mouse_state.toggle_flag(MouseModeFlags::CLICK_DRAG_TRACKING);
                        }
                    }
                    KeyCode::Char('8') if current_tab == Tab::KeyEvents => {
                        let should_toggle = !keyboard_state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
                            || key_event.is_release();
                        if should_toggle {
                            mouse_state.toggle_flag(MouseModeFlags::ANY_EVENT_TRACKING);
                        }
                    }
                    KeyCode::Char('9') if current_tab == Tab::KeyEvents => {
                        let should_toggle = !keyboard_state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
                            || key_event.is_release();
                        if should_toggle {
                            mouse_state.toggle_flag(MouseModeFlags::SGR_FORMAT);
                        }
                    }
                    KeyCode::Char('m') if current_tab == Tab::KeyEvents => {
                        let should_toggle = !keyboard_state.has_flag(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
                            || key_event.is_release();
                        if should_toggle {
                            mouse_state.toggle_all();
                            tracing::info!("Toggling all mouse modes");
                        }
                    }
                    _ => {}
                };

                // Format and display the key event
                last_event = format_key_event(key_event);
                event_log.add_entry(last_event.clone(), last_bytes.clone(), "feed_with()", startup_time.elapsed().as_millis());
                should_redraw = true;
            } else if let Some(mouse_event) =
                event.downcast_ref::<vtio::event::MouseEvent>()
            {
                tracing::info!("MouseEvent received: {:?}", mouse_event);
                last_event = format_mouse_event(mouse_event);
                event_log.add_entry(last_event.clone(), last_bytes.clone(), "feed_with()", startup_time.elapsed().as_millis());
                should_redraw = true;
            } else if let Some(_cpr) = event.downcast_ref::<vtio::event::cursor::CursorPositionReport>() {
                // Cursor Position Report - this is our terminator query
                // Mark all remaining unknown modes as ignored since the terminal responded to CPR
                // but not to those mode queries
                if !terminal_state.is_startup_complete() {
                    terminal_state.finalize_remaining_modes();
                    tracing::info!(
                        "Received CPR terminator: marked remaining unknown modes as ignored ({}/{})",
                        terminal_state.received_responses,
                        terminal_state.expected_responses
                    );
                }
            } else {
                // Check if it's a terminal mode response
                tracing::debug!("Other event type: {}", std::any::type_name_of_val(event));

                // Log unrecognized events when mouse capture is enabled
                if mouse_state.is_any_enabled()
                    && let Some(unrecognized) = event.downcast_ref::<vtio::event::UnrecognizedInputEvent>()
                {
                    tracing::info!(
                        "Unrecognized event while mouse capture enabled: {:?}",
                        unrecognized
                    );
                }
                if let Some(mode_description) = handle_mode_response(&mut terminal_state, event) {
                    tracing::debug!(
                        "Terminal mode response: {} (received {}/{})",
                        mode_description,
                        terminal_state.received_responses,
                        terminal_state.expected_responses
                    );
                    // Log terminal mode responses
                    last_event = mode_description;
                    event_log.add_entry(last_event.clone(), last_bytes.clone(), "feed_with()", startup_time.elapsed().as_millis());

                    // Only redraw if startup is complete
                    if terminal_state.is_startup_complete() {
                        should_redraw = true;
                    }
                } else {
                    // Show debug info for unrecognized events
                    tracing::debug!("Unrecognized event type: {}", std::any::type_name_of_val(event));
                    last_event = format!(
                        "Unrecognized: {}",
                        std::any::type_name_of_val(event)
                    );
                    event_log.add_entry(last_event.clone(), last_bytes.clone(), "feed_with()", startup_time.elapsed().as_millis());
                    should_redraw = true;
                }
            }
        });
        tracing::debug!(
            "Completed parser.feed_with(), processed {} events",
            event_count
        );

        if should_exit {
            tracing::info!("Exit requested");
            break;
        }

        // If keyboard enhancement flags changed, send them to the terminal
        if flags_changed {
            tracing::debug!(
                "Keyboard enhancement flags changed, sending to terminal"
            );
            send_keyboard_enhancement_flags(
                &mut stdout,
                keyboard_state.get_flags(),
            )?;
        }

        // If mouse modes changed, sync with terminal
        let new_mouse_flags = mouse_state.get_flags();
        if old_mouse_flags != new_mouse_flags {
            tracing::info!(
                "Mouse modes changed: {:?} -> {:?}",
                old_mouse_flags,
                new_mouse_flags
            );
            sync_mouse_modes(&mut stdout, old_mouse_flags, new_mouse_flags)?;
        }

        // Now that we've processed this data, check if more is available
        // ALWAYS check for more data immediately available, even during startup
        // This ensures we drain all buffered terminal responses without delay
        //
        // IMPORTANT: This fixes a critical bug where terminal mode query responses
        // would arrive but not be read immediately. The terminal sends multiple
        // responses that may exceed the buffer size, so we must check for more
        // data after each read and loop back immediately to read it, rather than
        // waiting for the next poll timeout. Without this, responses can sit in
        // the kernel buffer until user input triggers another read cycle.
        tracing::debug!("Checking if more data is immediately available");
        let more_data = raw_mode::poll_stdin(Duration::from_millis(0))?;
        tracing::debug!(
            "Checking for more data with 0ms timeout: {}",
            more_data
        );

        // Only enable idle timeout if:
        // 1. No more data is immediately available
        // 2. Startup is complete (don't call idle() during startup)
        let startup_suppress = !terminal_state.is_startup_complete();
        if !startup_suppress && !more_data {
            pending_idle = true;
            tracing::debug!(
                "Set pending_idle=true (no more data, startup complete)"
            );
        } else {
            pending_idle = false;
            if more_data {
                tracing::debug!("Set pending_idle=false (more data available)");
            } else if startup_suppress {
                tracing::debug!(
                    "Set pending_idle=false (startup still in progress)"
                );
            }
        }

        if should_redraw && terminal_state.is_startup_complete() {
            draw_ui(
                &mut stdout,
                current_tab,
                &keyboard_state,
                &mouse_state,
                &terminal_state,
                &event_log,
                &last_event,
                &last_bytes,
            )?;
        }
    }

    send_keyboard_enhancement_flags(
        &mut stdout,
        KeyboardEnhancementFlags::empty(),
    )?;
    // Disable all mouse modes if any were enabled
    if mouse_state.is_any_enabled() {
        sync_mouse_modes(
            &mut stdout,
            mouse_state.get_flags(),
            MouseModeFlags::empty(),
        )?;
    }
    exit_alternate_screen(&mut stdout)?;

    Ok(())
}

fn format_mouse_event(mouse_event: &vtio::event::MouseEvent) -> String {
    let mut parts = vec![format!("MouseEvent: {:?}", mouse_event.kind)];

    // Add detailed breakdown
    let mut details = Vec::new();

    // Event kind with button info
    match &mouse_event.kind {
        MouseEventKind::Down(button) => {
            details.push(format!("button: {button:?}"));
            details.push("action: Down".to_string());
        }
        MouseEventKind::Up(button) => {
            details.push(format!("button: {button:?}"));
            details.push("action: Up".to_string());
        }
        MouseEventKind::Drag(button) => {
            details.push(format!("button: {button:?}"));
            details.push("action: Drag".to_string());
        }
        MouseEventKind::Moved => {
            details.push("action: Moved".to_string());
        }
        MouseEventKind::ScrollDown => {
            details.push("action: ScrollDown".to_string());
        }
        MouseEventKind::ScrollUp => {
            details.push("action: ScrollUp".to_string());
        }
        MouseEventKind::ScrollLeft => {
            details.push("action: ScrollLeft".to_string());
        }
        MouseEventKind::ScrollRight => {
            details.push("action: ScrollRight".to_string());
        }
    }

    // Coordinates (0-based)
    details.push(format!(
        "position: ({}, {})",
        mouse_event.col(),
        mouse_event.row()
    ));

    // Modifiers
    if !mouse_event.modifiers.is_empty() {
        details.push(format!("modifiers: {:?}", *mouse_event.modifiers));
    }

    parts.push(format!("    Details: {}", details.join(", ")));

    parts.join("\n\r    ")
}
