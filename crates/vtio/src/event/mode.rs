//! Definitions and helpers for terminal modes.

/// Trait for DEC private terminal modes that support XTSAVE/XTRESTORE.
///
/// This trait is implemented by the `terminal_mode!` macro for all DEC private modes
/// (those with `private = '?'`). It provides the mode number for use with
/// [`SaveTerminalModes`] and [`RestoreTerminalModes`].
///
/// # Example
///
/// ```ignore
/// use vtio::event::mode::{TerminalMode, SaveTerminalModes};
/// use vtio::event::cursor::CursorVisibility;
///
/// // Get the mode number for cursor visibility
/// assert_eq!(<CursorVisibility as TerminalMode>::MODE_NUMBER, 25);
///
/// // Use with SaveTerminalModes
/// let save = SaveTerminalModes::from_modes::<(CursorVisibility,)>();
/// ```
pub trait TerminalMode {
    /// The DEC private mode number for this terminal mode.
    const MODE_NUMBER: u16;
}

/// Const function to parse a u16 from a string literal at compile time.
///
/// This is used by the `terminal_mode!` macro to convert the mode number
/// string parameter to a `u16` constant.
#[doc(hidden)]
#[must_use]
pub const fn parse_mode_number(s: &str) -> u16 {
    match u16::from_str_radix(s, 10) {
        Ok(n) => n,
        Err(_) => panic!("invalid mode number"),
    }
}

/// Represents state of terminal mode as reported in `DECRPM` responses.
///
/// See <https://vt100.net/docs/vt510-rm/DECRPM.html> for more information.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    num_enum::IntoPrimitive,
    num_enum::TryFromPrimitive,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
)]
#[repr(u8)]
pub enum TerminalModeState {
    NotRecognized = 0,
    Set = 1,
    Reset = 2,
    PermanentlySet = 3,
    PermanentlyReset = 4,
}

/// Generate terminal mode control sequences.
///
/// This macro generates control sequence structs for a terminal mode:
/// - `Enable{Name}`: CSI sequence with 'h' final byte to enable the mode
/// - `Disable{Name}`: CSI sequence with 'l' final byte to disable the mode
/// - `Request{Name}`: CSI sequence with '$' intermediate and 'p' final byte
///   to request the mode state (DECRQM)
/// - `{Name}`: CSI sequence with '$' intermediate and 'y' final byte
///   representing the mode state response (DECRPM, with `state` field)
///
/// For DEC private modes (those with `private = '?'`), it also generates:
/// - `Save{Name}`: CSI sequence with 's' final byte to save mode state (XTSAVE)
/// - `Restore{Name}`: CSI sequence with 'r' final byte to restore mode state (XTRESTORE)
/// - Implements [`TerminalMode`] trait for all generated structs
///
/// # Syntax
///
/// ```ignore
/// terminal_mode!(ModeName, params = ["param_value"]);
/// terminal_mode!(ModeName, private = '?', params = ["param_value"]);
/// ```
///
/// # Parameters
///
/// - `ModeName`: The base identifier for the mode
/// - `private`: Optional private parameter character (e.g., '?')
/// - `params`: Required parameter array for the CSI sequence
///
/// # Example
///
/// ```ignore
/// terminal_mode!(RelativeCursorOriginMode, private = '?', params = ["6"]);
/// ```
///
/// This generates:
/// - `EnableRelativeCursorOriginMode` → `CSI ? 6 h`
/// - `DisableRelativeCursorOriginMode` → `CSI ? 6 l`
/// - `RequestRelativeCursorOriginMode` → `CSI ? 6 $ p`
/// - `RelativeCursorOriginMode` → `CSI ? 6 ; Ps $ y` (with `state` field)
/// - `SaveRelativeCursorOriginMode` → `CSI ? 6 s` (XTSAVE)
/// - `RestoreRelativeCursorOriginMode` → `CSI ? 6 r` (XTRESTORE)
#[macro_export]
macro_rules! terminal_mode {
    // DEC private mode (with '?' prefix) - generates Save/Restore variants
    ($(#[$meta:meta])* $base_name:ident, private = $private:literal, params = [$param:literal]) => {
        $crate::__private::paste::paste! {
            $(#[$meta])*
            #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, ::vtansi::derive::AnsiInput)]
            #[vtansi(csi, private = $private, params = [$param], intermediate = "$", finalbyte = 'y')]
            pub struct [<$base_name>] {
                pub state: $crate::event::mode::TerminalModeState,
            }

            impl $crate::event::mode::TerminalMode for [<$base_name>] {
                const MODE_NUMBER: u16 = $crate::event::mode::parse_mode_number($param);
            }

            #[doc = concat!("Enable [`", stringify!($base_name), "`].")]
            #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, ::vtansi::derive::AnsiOutput)]
            #[vtansi(csi, private = $private, params = [$param], finalbyte = 'h')]
            pub struct [<Enable $base_name>];

            impl $crate::event::mode::TerminalMode for [<Enable $base_name>] {
                const MODE_NUMBER: u16 = $crate::event::mode::parse_mode_number($param);
            }

            #[doc = concat!("Disable [`", stringify!($base_name), "`].")]
            #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, ::vtansi::derive::AnsiOutput)]
            #[vtansi(csi, private = $private, params = [$param], finalbyte = 'l')]
            pub struct [<Disable $base_name>];

            impl $crate::event::mode::TerminalMode for [<Disable $base_name>] {
                const MODE_NUMBER: u16 = $crate::event::mode::parse_mode_number($param);
            }

            #[doc = concat!("Query state of [`", stringify!($base_name), "`] (DECRQM).")]
            #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, ::vtansi::derive::AnsiOutput)]
            #[vtansi(csi, private = $private, params = [$param], intermediate = "$", finalbyte = 'p')]
            pub struct [<Request $base_name>];

            impl $crate::event::mode::TerminalMode for [<Request $base_name>] {
                const MODE_NUMBER: u16 = $crate::event::mode::parse_mode_number($param);
            }

            #[doc = concat!("Save state of [`", stringify!($base_name), "`] (XTSAVE).")]
            ///
            /// Saves the current state of this DEC private mode to an internal stack.
            /// Use the corresponding `Restore` struct to restore the saved state.
            ///
            /// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-which-begin-with-CSI>
            /// for more information on XTSAVE.
            #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, ::vtansi::derive::AnsiOutput)]
            #[vtansi(csi, private = $private, params = [$param], finalbyte = 's')]
            pub struct [<Save $base_name>];

            impl $crate::event::mode::TerminalMode for [<Save $base_name>] {
                const MODE_NUMBER: u16 = $crate::event::mode::parse_mode_number($param);
            }

            #[doc = concat!("Restore state of [`", stringify!($base_name), "`] (XTRESTORE).")]
            ///
            /// Restores the previously saved state of this DEC private mode from the internal stack.
            /// Use the corresponding `Save` struct to save the state first.
            ///
            /// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-which-begin-with-CSI>
            /// for more information on XTRESTORE.
            #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, ::vtansi::derive::AnsiOutput)]
            #[vtansi(csi, private = $private, params = [$param], finalbyte = 'r')]
            pub struct [<Restore $base_name>];

            impl $crate::event::mode::TerminalMode for [<Restore $base_name>] {
                const MODE_NUMBER: u16 = $crate::event::mode::parse_mode_number($param);
            }
        }
    };

    // Standard ANSI mode (no private prefix) - no Save/Restore variants
    ($(#[$meta:meta])* $base_name:ident, params = [$($params:literal),* $(,)?]) => {
        $crate::__private::paste::paste! {
            $(#[$meta])*
            #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, ::vtansi::derive::AnsiInput)]
            #[vtansi(csi, params = [$($params),*], intermediate = "$", finalbyte = 'y')]
            pub struct [<$base_name>] {
                pub state: $crate::event::mode::TerminalModeState,
            }

            #[doc = concat!("Enable [`", stringify!($base_name), "`].")]
            #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, ::vtansi::derive::AnsiOutput)]
            #[vtansi(csi, params = [$($params),*], finalbyte = 'h')]
            pub struct [<Enable $base_name>];

            #[doc = concat!("Disable [`", stringify!($base_name), "`].")]
            #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, ::vtansi::derive::AnsiOutput)]
            #[vtansi(csi, params = [$($params),*], finalbyte = 'l')]
            pub struct [<Disable $base_name>];

            #[doc = concat!("Query state of [`", stringify!($base_name), "`] (DECRQM).")]
            #[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash, ::vtansi::derive::AnsiOutput)]
            #[vtansi(csi, params = [$($params),*], intermediate = "$", finalbyte = 'p')]
            pub struct [<Request $base_name>];
        }
    };

    // DEC private mode with flag specifier
    ($(#[$meta:meta])* $base_name:ident, private = $private:literal, params = [$param:literal], flag = $flag:expr) => {
        $crate::terminal_mode!($(#[$meta])* $base_name, private = $private, params = [$param]);

        impl $crate::event::keyboard::AsModeFlag for $base_name {
            fn as_mode_flag(&self) -> $crate::event::keyboard::KeyboardModeFlags {
                if self.state == $crate::event::mode::TerminalModeState::Set
                    || self.state == $crate::event::mode::TerminalModeState::PermanentlySet
                {
                    $flag
                } else {
                    $crate::event::keyboard::KeyboardModeFlags::empty()
                }
            }
        }
    };

    // Standard ANSI mode with flag specifier
    ($(#[$meta:meta])* $base_name:ident, params = [$($params:literal),* $(,)?], flag = $flag:expr) => {
        $crate::terminal_mode!($(#[$meta])* $base_name, params = [$($params),*]);

        impl $crate::event::keyboard::AsModeFlag for $base_name {
            fn as_mode_flag(&self) -> $crate::event::keyboard::KeyboardModeFlags {
                if self.state == $crate::event::mode::TerminalModeState::Set
                    || self.state == $crate::event::mode::TerminalModeState::PermanentlySet
                {
                    $flag
                } else {
                    $crate::event::keyboard::KeyboardModeFlags::empty()
                }
            }
        }
    };
}

// ============================================================================
// Generic Multi-Mode Save/Restore Operations (XTSAVE/XTRESTORE)
// ============================================================================

/// Wrapper for a list of DEC private mode numbers.
///
/// This type is used with [`SaveTerminalModes`] and [`RestoreTerminalModes`]
/// to save or restore multiple modes at once.
///
/// # Example
///
/// ```ignore
/// use vtio::event::mode::{TerminalModes, SaveTerminalModes};
/// use vtio::event::cursor::CursorVisibility;
///
/// // Type-safe construction from TerminalMode types
/// let save = SaveTerminalModes::from_modes::<(CursorVisibility,)>();
///
/// // Or using raw mode numbers
/// let modes = TerminalModes::new(vec![25, 2004, 1049]);
/// let save = SaveTerminalModes { modes };
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    Hash,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
)]
#[vtansi(transparent, delimiter = b';')]
pub struct TerminalModes(pub Vec<u16>);

impl TerminalModes {
    /// Create a new `TerminalModes` from a vector of mode numbers.
    #[must_use]
    pub fn new(modes: Vec<u16>) -> Self {
        Self(modes)
    }

    /// Create a new `TerminalModes` from a slice of mode numbers.
    #[must_use]
    pub fn from_slice(modes: &[u16]) -> Self {
        Self(modes.to_vec())
    }

    /// Returns `true` if the list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of modes in the list.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a slice of the mode numbers.
    #[must_use]
    pub fn as_slice(&self) -> &[u16] {
        &self.0
    }
}

impl From<Vec<u16>> for TerminalModes {
    fn from(modes: Vec<u16>) -> Self {
        Self(modes)
    }
}

impl From<&[u16]> for TerminalModes {
    fn from(modes: &[u16]) -> Self {
        Self(modes.to_vec())
    }
}

impl<const N: usize> From<[u16; N]> for TerminalModes {
    fn from(modes: [u16; N]) -> Self {
        Self(modes.to_vec())
    }
}

impl std::ops::Deref for TerminalModes {
    type Target = Vec<u16>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Helper trait for building `TerminalModes` from tuples of `TerminalMode` types.
///
/// This trait is implemented for tuples of up to 12 types implementing [`TerminalMode`].
pub trait IntoTerminalModes {
    /// Convert to a `TerminalModes` list.
    fn into_terminal_modes() -> TerminalModes;
}

impl IntoTerminalModes for () {
    fn into_terminal_modes() -> TerminalModes {
        TerminalModes(Vec::new())
    }
}

impl<T1: TerminalMode> IntoTerminalModes for (T1,) {
    fn into_terminal_modes() -> TerminalModes {
        TerminalModes(vec![T1::MODE_NUMBER])
    }
}

impl<T1: TerminalMode, T2: TerminalMode> IntoTerminalModes for (T1, T2) {
    fn into_terminal_modes() -> TerminalModes {
        TerminalModes(vec![T1::MODE_NUMBER, T2::MODE_NUMBER])
    }
}

impl<T1: TerminalMode, T2: TerminalMode, T3: TerminalMode> IntoTerminalModes
    for (T1, T2, T3)
{
    fn into_terminal_modes() -> TerminalModes {
        TerminalModes(vec![T1::MODE_NUMBER, T2::MODE_NUMBER, T3::MODE_NUMBER])
    }
}

impl<T1: TerminalMode, T2: TerminalMode, T3: TerminalMode, T4: TerminalMode>
    IntoTerminalModes for (T1, T2, T3, T4)
{
    fn into_terminal_modes() -> TerminalModes {
        TerminalModes(vec![
            T1::MODE_NUMBER,
            T2::MODE_NUMBER,
            T3::MODE_NUMBER,
            T4::MODE_NUMBER,
        ])
    }
}

impl<
    T1: TerminalMode,
    T2: TerminalMode,
    T3: TerminalMode,
    T4: TerminalMode,
    T5: TerminalMode,
> IntoTerminalModes for (T1, T2, T3, T4, T5)
{
    fn into_terminal_modes() -> TerminalModes {
        TerminalModes(vec![
            T1::MODE_NUMBER,
            T2::MODE_NUMBER,
            T3::MODE_NUMBER,
            T4::MODE_NUMBER,
            T5::MODE_NUMBER,
        ])
    }
}

impl<
    T1: TerminalMode,
    T2: TerminalMode,
    T3: TerminalMode,
    T4: TerminalMode,
    T5: TerminalMode,
    T6: TerminalMode,
> IntoTerminalModes for (T1, T2, T3, T4, T5, T6)
{
    fn into_terminal_modes() -> TerminalModes {
        TerminalModes(vec![
            T1::MODE_NUMBER,
            T2::MODE_NUMBER,
            T3::MODE_NUMBER,
            T4::MODE_NUMBER,
            T5::MODE_NUMBER,
            T6::MODE_NUMBER,
        ])
    }
}

impl<
    T1: TerminalMode,
    T2: TerminalMode,
    T3: TerminalMode,
    T4: TerminalMode,
    T5: TerminalMode,
    T6: TerminalMode,
    T7: TerminalMode,
> IntoTerminalModes for (T1, T2, T3, T4, T5, T6, T7)
{
    fn into_terminal_modes() -> TerminalModes {
        TerminalModes(vec![
            T1::MODE_NUMBER,
            T2::MODE_NUMBER,
            T3::MODE_NUMBER,
            T4::MODE_NUMBER,
            T5::MODE_NUMBER,
            T6::MODE_NUMBER,
            T7::MODE_NUMBER,
        ])
    }
}

impl<
    T1: TerminalMode,
    T2: TerminalMode,
    T3: TerminalMode,
    T4: TerminalMode,
    T5: TerminalMode,
    T6: TerminalMode,
    T7: TerminalMode,
    T8: TerminalMode,
> IntoTerminalModes for (T1, T2, T3, T4, T5, T6, T7, T8)
{
    fn into_terminal_modes() -> TerminalModes {
        TerminalModes(vec![
            T1::MODE_NUMBER,
            T2::MODE_NUMBER,
            T3::MODE_NUMBER,
            T4::MODE_NUMBER,
            T5::MODE_NUMBER,
            T6::MODE_NUMBER,
            T7::MODE_NUMBER,
            T8::MODE_NUMBER,
        ])
    }
}

/// Save multiple DEC private mode values at once (XTSAVE).
///
/// *Sequence*: `CSI ? Pm ; Pm ; ... s`
///
/// This sequence saves the current state of the specified DEC private modes
/// to an internal stack in the terminal. The saved states can later be
/// restored using [`RestoreTerminalModes`].
///
/// This is an xterm extension (XTSAVE) that allows saving multiple mode
/// states with a single sequence, which is more efficient than saving
/// modes individually.
///
/// # Example
///
/// ```ignore
/// use vtio::event::mode::SaveTerminalModes;
/// use vtio::event::cursor::CursorVisibility;
/// use vtio::event::terminal::BracketedPasteMode;
/// use vtansi::AnsiEncode;
///
/// // Type-safe: use from_modes with TerminalMode types
/// let save = SaveTerminalModes::from_modes::<(CursorVisibility, BracketedPasteMode)>();
///
/// // Or with raw mode numbers
/// let save = SaveTerminalModes::new(vec![25, 1000, 2004]);
///
/// let mut buf = Vec::new();
/// save.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(buf, b"\x1b[?25;1000;2004s");
/// ```
///
/// # See Also
///
/// - [`RestoreTerminalModes`] - Restore saved mode states
/// - Individual `Save{ModeName}` structs generated by [`terminal_mode!`] macro
///
/// # References
///
/// - <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-which-begin-with-CSI>
#[derive(
    Debug, Clone, PartialEq, Eq, Default, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', finalbyte = 's')]
pub struct SaveTerminalModes {
    /// The mode numbers to save.
    pub modes: TerminalModes,
}

impl SaveTerminalModes {
    /// Create a new `SaveTerminalModes` with the given mode numbers.
    #[must_use]
    pub fn new(modes: impl Into<TerminalModes>) -> Self {
        Self {
            modes: modes.into(),
        }
    }

    /// Create a new `SaveTerminalModes` from types implementing [`TerminalMode`].
    ///
    /// This provides type-safe construction ensuring only valid terminal modes are used.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use vtio::event::mode::SaveTerminalModes;
    /// use vtio::event::cursor::CursorVisibility;
    /// use vtio::event::terminal::BracketedPasteMode;
    ///
    /// let save = SaveTerminalModes::from_modes::<(CursorVisibility, BracketedPasteMode)>();
    /// ```
    #[must_use]
    pub fn from_modes<T: IntoTerminalModes>() -> Self {
        Self {
            modes: T::into_terminal_modes(),
        }
    }
}

/// Restore multiple DEC private mode values at once (XTRESTORE).
///
/// *Sequence*: `CSI ? Pm ; Pm ; ... r`
///
/// This sequence restores the previously saved state of the specified DEC
/// private modes from the terminal's internal stack. The states must have
/// been previously saved using [`SaveTerminalModes`].
///
/// This is an xterm extension (XTRESTORE) that allows restoring multiple mode
/// states with a single sequence, which is more efficient than restoring
/// modes individually.
///
/// # Example
///
/// ```ignore
/// use vtio::event::mode::RestoreTerminalModes;
/// use vtio::event::cursor::CursorVisibility;
/// use vtio::event::terminal::BracketedPasteMode;
/// use vtansi::AnsiEncode;
///
/// // Type-safe: use from_modes with TerminalMode types
/// let restore = RestoreTerminalModes::from_modes::<(CursorVisibility, BracketedPasteMode)>();
///
/// // Or with raw mode numbers
/// let restore = RestoreTerminalModes::new(vec![25, 1000, 2004]);
///
/// let mut buf = Vec::new();
/// restore.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(buf, b"\x1b[?25;1000;2004r");
/// ```
///
/// # See Also
///
/// - [`SaveTerminalModes`] - Save mode states
/// - Individual `Restore{ModeName}` structs generated by [`terminal_mode!`] macro
///
/// # References
///
/// - <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-which-begin-with-CSI>
#[derive(
    Debug, Clone, PartialEq, Eq, Default, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, private = '?', finalbyte = 'r')]
pub struct RestoreTerminalModes {
    /// The mode numbers to restore.
    pub modes: TerminalModes,
}

impl RestoreTerminalModes {
    /// Create a new `RestoreTerminalModes` with the given mode numbers.
    #[must_use]
    pub fn new(modes: impl Into<TerminalModes>) -> Self {
        Self {
            modes: modes.into(),
        }
    }

    /// Create a new `RestoreTerminalModes` from types implementing [`TerminalMode`].
    ///
    /// This provides type-safe construction ensuring only valid terminal modes are used.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use vtio::event::mode::RestoreTerminalModes;
    /// use vtio::event::cursor::CursorVisibility;
    /// use vtio::event::terminal::BracketedPasteMode;
    ///
    /// let restore = RestoreTerminalModes::from_modes::<(CursorVisibility, BracketedPasteMode)>();
    /// ```
    #[must_use]
    pub fn from_modes<T: IntoTerminalModes>() -> Self {
        Self {
            modes: T::into_terminal_modes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::AnsiEncode;

    #[test]
    fn test_save_terminal_modes_single() {
        let save = SaveTerminalModes::new(vec![25]);

        let mut buf = Vec::new();
        save.encode_ansi_into(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b[?25s");
    }

    #[test]
    fn test_save_terminal_modes_multiple() {
        let save = SaveTerminalModes::new(vec![25, 1000, 2004]);

        let mut buf = Vec::new();
        save.encode_ansi_into(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b[?25;1000;2004s");
    }

    #[test]
    fn test_save_terminal_modes_empty() {
        let save = SaveTerminalModes::new(Vec::<u16>::new());

        let mut buf = Vec::new();
        save.encode_ansi_into(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b[?s");
    }

    #[test]
    fn test_restore_terminal_modes_single() {
        let restore = RestoreTerminalModes::new(vec![25]);

        let mut buf = Vec::new();
        restore.encode_ansi_into(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b[?25r");
    }

    #[test]
    fn test_restore_terminal_modes_multiple() {
        let restore = RestoreTerminalModes::new(vec![25, 1000, 2004]);

        let mut buf = Vec::new();
        restore.encode_ansi_into(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b[?25;1000;2004r");
    }

    #[test]
    fn test_restore_terminal_modes_empty() {
        let restore = RestoreTerminalModes::new(Vec::<u16>::new());

        let mut buf = Vec::new();
        restore.encode_ansi_into(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b[?r");
    }

    #[test]
    fn test_terminal_modes_from_array() {
        let modes: TerminalModes = [1, 25, 2004].into();
        assert_eq!(modes.0, vec![1, 25, 2004]);
    }

    #[test]
    fn test_terminal_modes_from_slice() {
        let arr = [1u16, 25, 2004];
        let modes = TerminalModes::from_slice(&arr);
        assert_eq!(modes.0, vec![1, 25, 2004]);
    }

    #[test]
    fn test_terminal_modes_parse() {
        let modes = <TerminalModes as vtansi::TryFromAnsi>::try_from_ansi(
            b"25;1000;2004",
        )
        .unwrap();
        assert_eq!(modes.0, vec![25, 1000, 2004]);
    }

    #[test]
    fn test_terminal_modes_parse_single() {
        let modes =
            <TerminalModes as vtansi::TryFromAnsi>::try_from_ansi(b"25")
                .unwrap();
        assert_eq!(modes.0, vec![25]);
    }

    #[test]
    fn test_terminal_modes_parse_empty() {
        let modes =
            <TerminalModes as vtansi::TryFromAnsi>::try_from_ansi(b"").unwrap();
        assert!(modes.0.is_empty());
    }

    #[test]
    fn test_terminal_modes_accessors() {
        let modes = TerminalModes::new(vec![1, 2, 3]);
        assert!(!modes.is_empty());
        assert_eq!(modes.len(), 3);
        assert_eq!(modes.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_terminal_mode_trait() {
        use crate::event::cursor::{
            CursorVisibility, DisableCursorVisibility, EnableCursorVisibility,
            RequestCursorVisibility, RestoreCursorVisibility,
            SaveCursorVisibility,
        };

        // All generated structs should implement TerminalMode with the same MODE_NUMBER
        assert_eq!(CursorVisibility::MODE_NUMBER, 25);
        assert_eq!(EnableCursorVisibility::MODE_NUMBER, 25);
        assert_eq!(DisableCursorVisibility::MODE_NUMBER, 25);
        assert_eq!(RequestCursorVisibility::MODE_NUMBER, 25);
        assert_eq!(SaveCursorVisibility::MODE_NUMBER, 25);
        assert_eq!(RestoreCursorVisibility::MODE_NUMBER, 25);
    }

    #[test]
    fn test_save_terminal_modes_from_modes_single() {
        use crate::event::cursor::CursorVisibility;

        let save = SaveTerminalModes::from_modes::<(CursorVisibility,)>();

        let mut buf = Vec::new();
        save.encode_ansi_into(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b[?25s");
    }

    #[test]
    fn test_save_terminal_modes_from_modes_multiple() {
        use crate::event::cursor::{CursorBlinking, CursorVisibility};

        let save = SaveTerminalModes::from_modes::<(
            CursorVisibility,
            CursorBlinking,
        )>();

        let mut buf = Vec::new();
        save.encode_ansi_into(&mut buf).unwrap();

        // CursorVisibility = 25, CursorBlinking = 12
        assert_eq!(buf, b"\x1b[?25;12s");
    }

    #[test]
    fn test_restore_terminal_modes_from_modes() {
        use crate::event::cursor::{CursorBlinking, CursorVisibility};

        let restore = RestoreTerminalModes::from_modes::<(
            CursorVisibility,
            CursorBlinking,
        )>();

        let mut buf = Vec::new();
        restore.encode_ansi_into(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b[?25;12r");
    }

    #[test]
    fn test_into_terminal_modes_empty() {
        let modes = <() as IntoTerminalModes>::into_terminal_modes();
        assert!(modes.is_empty());
    }

    #[test]
    fn test_into_terminal_modes_tuple() {
        use crate::event::cursor::{
            CursorBlinking, CursorVisibility, RelativeCursorOriginMode,
        };

        let modes = <(
            CursorVisibility,
            CursorBlinking,
            RelativeCursorOriginMode,
        ) as IntoTerminalModes>::into_terminal_modes();

        // CursorVisibility = 25, CursorBlinking = 12, RelativeCursorOriginMode = 6
        assert_eq!(modes.as_slice(), &[25, 12, 6]);
    }
}
