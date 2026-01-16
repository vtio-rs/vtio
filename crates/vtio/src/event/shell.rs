//! # Shell Integration
//!
//! The `shell` module provides functionality for shell integration sequences.
//!
//! Shell integration sequences (OSC 133) enable terminal emulators to track
//! shell prompts, command input, and command output. This allows features
//! like:
//! - Jumping between prompts
//! - Selecting command output
//! - Tracking command execution status
//! - Recording command history with context
//!
//! These sequences are supported by modern terminal emulators including
//! `iTerm2`, `VSCode`, `WezTerm`, and others.
//!
//! ## Positional Parameters
//!
//! Some shell integration sequences support optional positional parameters.
//! Fields marked with `#[vtansi(positional)]` are encoded as semicolon-
//! separated values in the OSC data section. Optional positional parameters
//! (using `Option<T>`) must come after all required positional parameters.
//!
//! When encoding, optional parameters that are `None` are omitted, along with
//! any subsequent parameters.

/// Current location report (hostname and directory).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "7")]
pub struct CurrentLocation<'a>(pub &'a str);

/// A command that marks the beginning of a shell prompt.
///
/// This sequence (OSC 133;A) indicates where a new prompt starts. Terminal
/// emulators can use this to enable features like jumping between prompts.
///
/// # Notes
///
/// - This should be emitted at the very start of drawing the prompt.
/// - Must be paired with `PromptEnd` to mark where the prompt ends.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "133", data = "A")]
pub struct PromptStart;

/// A command that marks the end of a shell prompt and the beginning of user
/// input.
///
/// This sequence (OSC 133;B) indicates where the prompt ends and user input
/// begins. Terminal emulators can use this to distinguish between the
/// prompt and the user's command.
///
/// # Notes
///
/// - This should be emitted right before accepting user input.
/// - Should follow a `PromptStart` sequence.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "133", data = "B")]
pub struct PromptEnd;

/// A command that marks the start of command execution and output.
///
/// This sequence (OSC 133;C) indicates where the command output begins.
/// Terminal emulators can use this to enable features like selecting
/// command output or distinguishing input from output.
///
/// # Notes
///
/// - This should be emitted right before executing a command.
/// - Should follow a `PromptEnd` sequence.
/// - Must be paired with `CommandEnd` to mark where output ends.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "133", data = "C")]
pub struct CommandStart;

/// A command that marks the end of command output.
///
/// This sequence (`OSC 133;D` or OSC `133;D;exit_code`) indicates where the
/// command output ends. It can optionally include the command's exit code.
/// Terminal emulators can use this to track command execution status and
/// enable features like showing success/failure indicators.
///
/// # Notes
///
/// - This should be emitted after a command finishes execution.
/// - Should follow a `CommandStart` sequence.
/// - The exit code parameter is optional.
///
/// # Positional Parameters
///
/// The `exit_code` field is marked as a positional parameter. When encoded:
/// - `CommandEnd { exit_code: None }` produces `OSC 133;D ST`
/// - `CommandEnd { exit_code: Some(0) }` produces `OSC 133;D;0 ST`
/// - `CommandEnd { exit_code: Some(1) }` produces `OSC 133;D;1 ST`
///
/// # Example
///
/// ```
/// use vtio::event::shell::CommandEnd;
///
/// // Report command completion without exit code
/// let end = CommandEnd { exit_code: None };
///
/// // Report successful command completion
/// let end = CommandEnd { exit_code: Some(0) };
///
/// // Report command failure
/// let end = CommandEnd { exit_code: Some(1) };
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, vtansi::derive::AnsiOutput,
)]
#[vtansi(osc, number = "133", data = "D")]
pub struct CommandEnd {
    pub exit_code: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::{AnsiEncode, StaticAnsiEncode};

    #[test]
    fn test_prompt_start() {
        assert_eq!(PromptStart::BYTES, b"\x1b]133;A\x1b\\");
    }

    #[test]
    fn test_prompt_end() {
        assert_eq!(PromptEnd::BYTES, b"\x1b]133;B\x1b\\");
    }

    #[test]
    fn test_command_start() {
        assert_eq!(CommandStart::BYTES, b"\x1b]133;C\x1b\\");
    }

    #[test]
    fn test_command_end_without_exit_code() {
        let cmd = CommandEnd { exit_code: None };
        let mut buf = Vec::new();
        let result = cmd.encode_ansi_into(&mut buf);
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]133;D\x1b\\");
    }

    #[test]
    fn test_command_end_with_exit_code_zero() {
        let cmd = CommandEnd { exit_code: Some(0) };
        let mut buf = Vec::new();
        let result = cmd.encode_ansi_into(&mut buf);
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]133;D;0\x1b\\");
    }

    #[test]
    fn test_command_end_with_exit_code_nonzero() {
        let cmd = CommandEnd { exit_code: Some(1) };
        let mut buf = Vec::new();
        let result = cmd.encode_ansi_into(&mut buf);
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]133;D;1\x1b\\");
    }

    #[test]
    fn test_command_end_with_large_exit_code() {
        let cmd = CommandEnd {
            exit_code: Some(127),
        };
        let mut buf = Vec::new();
        let result = cmd.encode_ansi_into(&mut buf);
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buf).unwrap(), "\x1b]133;D;127\x1b\\");
    }
}
