//! SGR (Select Graphic Rendition) sequences.
use std::io::Write;

use vtansi::{ParseError, TryFromAnsi, TryFromAnsiIter, bitflags, write_csi};

// =============================================================================
// Basic ANSI Colors
// =============================================================================

/// Standard 8 ANSI colors used with SGR 30-37 (foreground) and 40-47 (background).
///
/// These colors were defined in ECMA-48 2nd edition (1979) and are universally
/// supported by terminals. The actual RGB values displayed depend on the
/// terminal's color scheme configuration.
///
/// # Standard Color Indices
///
/// | Index | Color | Typical Dark Theme | Typical Light Theme |
/// |-------|-------|-------------------|---------------------|
/// | 0 | Black | Dark gray/black | Black |
/// | 1 | Red | Light red | Dark red |
/// | 2 | Green | Light green | Dark green |
/// | 3 | Yellow | Light yellow/orange | Dark yellow/brown |
/// | 4 | Blue | Light blue | Dark blue |
/// | 5 | Magenta | Light magenta | Dark magenta |
/// | 6 | Cyan | Light cyan | Dark cyan |
/// | 7 | White | White/light gray | Dark gray |
///
/// # Usage
///
/// ```
/// use vtio::event::sgr::{AnsiColor, SgrAttr};
///
/// let red_fg = SgrAttr::foreground(AnsiColor::Red);
/// let blue_bg = SgrAttr::background(AnsiColor::Blue);
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u8)]
pub enum AnsiColor {
    /// Black (color index 0).
    #[default]
    Black = 0,
    /// Red (color index 1).
    Red = 1,
    /// Green (color index 2).
    Green = 2,
    /// Yellow (color index 3).
    Yellow = 3,
    /// Blue (color index 4).
    Blue = 4,
    /// Magenta (color index 5).
    Magenta = 5,
    /// Cyan (color index 6).
    Cyan = 6,
    /// White (color index 7).
    White = 7,
}

// =============================================================================
// Extended Underline Styles (kitty extension)
// =============================================================================

/// Extended underline styles using the `SGR 4:Ps` sub-parameter syntax.
///
/// This is a modern extension originally introduced by `kitty` and now supported
/// by `iTerm2`, `WezTerm`, `Alacritty`, and other modern terminal emulators.
///
/// # Encoding
///
/// Unlike the simple `SGR 4` (single underline), extended styles use
/// colon-separated sub-parameters:
///
/// ```text
/// CSI 4:3 m    # Curly underline
/// CSI 4:0 m    # Remove underline (equivalent to SGR 24)
/// ```
///
/// # Terminal Support
///
/// | Terminal | Support |
/// |----------|---------|
/// | `kitty` | Full (originator) |
/// | `iTerm2` | Full |
/// | `WezTerm` | Full |
/// | `Alacritty` | Partial (no dotted/dashed) |
/// | `VTE`-based | Partial |
/// | `xterm` | Not supported |
/// | `Windows Terminal` | Partial |
///
/// # References
///
/// - `kitty` documentation: <https://sw.kovidgoyal.net/kitty/underlines/>
/// - `st` undercurl patch: <https://st.suckless.org/patches/undercurl/>
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u8)]
pub enum UnderlineStyle {
    /// No underline (`4:0`).
    ///
    /// Equivalent to `SGR 24` (not underlined).
    #[default]
    None = 0,

    /// Single underline (`4:1`).
    ///
    /// Equivalent to `SGR 4`. This is the traditional underline style
    /// supported since VT100.
    Single = 1,

    /// Double underline (`4:2`).
    ///
    /// Equivalent to `SGR 21`. Supported since VT300 series.
    /// Renders as two parallel horizontal lines beneath the text.
    Double = 2,

    /// Curly/wavy underline (`4:3`), also known as "undercurl".
    ///
    /// Renders as a wavy line beneath the text. Commonly used by
    /// text editors to indicate spelling errors or warnings.
    Curly = 3,

    /// Dotted underline (`4:4`).
    ///
    /// Renders as a dotted line beneath the text.
    Dotted = 4,

    /// Dashed underline (`4:5`).
    ///
    /// Renders as a dashed line beneath the text.
    Dashed = 5,
}

// =============================================================================
// Simple SGR Code - Single Parameter Codes
// =============================================================================

/// Simple SGR codes that map directly to a single parameter value.
///
/// This enum contains all SGR codes that are encoded as a single integer
/// parameter without sub-parameters. It uses `#[repr(u16)]` to allow for
/// codes above 255 (bright colors use 90-107).
///
/// # When to Use
///
/// Use `SimpleSgrCode` when you need direct access to the numeric SGR codes,
/// such as for low-level protocol implementation or when interfacing with
/// systems that expect raw SGR values.
///
/// For most applications, prefer [`SgrAttr`] which provides a higher-level
/// API including extended colors and underline styles.
///
/// # Excluded Codes
///
/// The following codes require additional parameters and are handled by
/// [`SgrAttr`] instead:
///
/// - **11-19**: Alternative fonts (encoded as 10 + font number)
/// - **38**: Extended foreground color
/// - **48**: Extended background color
/// - **58**: Extended underline color
/// - **4:x**: Extended underline styles
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[repr(u16)]
#[non_exhaustive]
pub enum SimpleSgrCode {
    // =========================================================================
    // Reset (0)
    // =========================================================================
    /// Reset all attributes to default (`SGR 0`).
    ///
    /// Restores all text attributes to their default values, including:
    /// intensity, italic, underline, blink, inverse, hidden, strikethrough,
    /// foreground color, background color, and underline color.
    ///
    /// - **ECMA-48**: 2nd edition (1979)
    /// - **VT Support**: VT100+
    Reset = 0,

    // =========================================================================
    // Intensity (1, 2, 22)
    // =========================================================================
    /// Bold or increased intensity (`SGR 1`).
    ///
    /// Renders text with increased intensity. Many terminals render this
    /// as bold text, bright colors, or both.
    ///
    /// - **ECMA-48**: 2nd edition (1979)
    /// - **VT Support**: VT100+
    /// - **Reset**: [`NormalIntensity`](Self::NormalIntensity) (22) or [`Reset`](Self::Reset) (0)
    Bold = 1,

    /// Faint or decreased intensity (`SGR 2`).
    ///
    /// Renders text with decreased intensity (dimmed). Not supported on
    /// VT100/VT220; available on VT300+ and most modern terminals.
    ///
    /// - **ECMA-48**: 2nd edition (1979)
    /// - **VT Support**: VT300+
    /// - **Reset**: [`NormalIntensity`](Self::NormalIntensity) (22) or [`Reset`](Self::Reset) (0)
    Faint = 2,

    // =========================================================================
    // Italic (3)
    // =========================================================================
    /// Italic text (`SGR 3`).
    ///
    /// Renders text in italic style. Not supported by DEC VT terminals;
    /// widely supported by modern terminal emulators.
    ///
    /// - **ECMA-48**: 2nd edition (1979)
    /// - **VT Support**: Not supported (DEC terminals used this for "standout")
    /// - **Reset**: [`NotItalic`](Self::NotItalic) (23) or [`Reset`](Self::Reset) (0)
    Italic = 3,

    // =========================================================================
    // Underline (4)
    // =========================================================================
    /// Single underline (`SGR 4`).
    ///
    /// Renders text with a single underline. For extended underline styles
    /// (curly, dotted, dashed), see [`SgrAttr::UnderlineStyle`].
    ///
    /// - **ECMA-48**: 2nd edition (1979)
    /// - **VT Support**: VT100+
    /// - **Reset**: [`NotUnderlined`](Self::NotUnderlined) (24) or [`Reset`](Self::Reset) (0)
    Underline = 4,

    // =========================================================================
    // Blink (5, 6)
    // =========================================================================
    /// Slow blink (`SGR 5`), less than 150 blinks per minute.
    ///
    /// Causes text to blink slowly. Many modern terminals disable or
    /// ignore blinking for accessibility reasons.
    ///
    /// - **ECMA-48**: 2nd edition (1979)
    /// - **VT Support**: VT100+ (often rendered as bold in xterm)
    /// - **Reset**: [`NotBlinking`](Self::NotBlinking) (25) or [`Reset`](Self::Reset) (0)
    Blink = 5,

    /// Rapid blink (`SGR 6`), 150+ blinks per minute.
    ///
    /// Causes text to blink rapidly. Rarely supported by terminals;
    /// most treat it the same as slow blink or ignore it entirely.
    ///
    /// - **ECMA-48**: 2nd edition (1979)
    /// - **VT Support**: Not widely supported
    /// - **Reset**: [`NotBlinking`](Self::NotBlinking) (25) or [`Reset`](Self::Reset) (0)
    RapidBlink = 6,

    // =========================================================================
    // Inverse/Reverse (7)
    // =========================================================================
    /// Inverse/reverse video (`SGR 7`).
    ///
    /// Swaps foreground and background colors. If custom colors are set,
    /// those colors are swapped; otherwise the terminal's default
    /// foreground and background are swapped.
    ///
    /// - **ECMA-48**: 2nd edition (1979)
    /// - **VT Support**: VT100+
    /// - **Reset**: [`NotInverse`](Self::NotInverse) (27) or [`Reset`](Self::Reset) (0)
    Inverse = 7,

    // =========================================================================
    // Hidden/Invisible (8)
    // =========================================================================
    /// Hidden/invisible text (`SGR 8`).
    ///
    /// Renders text invisibly (typically by setting foreground color
    /// equal to background color). The text is still selectable and
    /// will appear when copied. Useful for password entry prompts.
    ///
    /// - **ECMA-48**: 2nd edition (1979)
    /// - **VT Support**: VT300+
    /// - **Reset**: [`NotHidden`](Self::NotHidden) (28) or [`Reset`](Self::Reset) (0)
    Hidden = 8,

    // =========================================================================
    // Strikethrough (9)
    // =========================================================================
    /// Crossed out/strikethrough text (`SGR 9`).
    ///
    /// Renders text with a horizontal line through the middle.
    /// Not supported by DEC VT terminals.
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **VT Support**: Not supported by DEC terminals
    /// - **Reset**: [`NotStrikethrough`](Self::NotStrikethrough) (29) or [`Reset`](Self::Reset) (0)
    Strikethrough = 9,

    // =========================================================================
    // Fonts (10, 20)
    // =========================================================================
    /// Primary (default) font (`SGR 10`).
    ///
    /// Selects the primary/default font. Resets any alternative font
    /// selection (codes 11-19).
    ///
    /// - **ECMA-48**: 2nd edition (1979)
    /// - **VT Support**: Not widely supported
    PrimaryFont = 10,

    // Note: Alternative fonts 11-19 are handled as SgrAttr::AlternativeFont
    /// Fraktur/Gothic font (`SGR 20`).
    ///
    /// Selects a Fraktur (blackletter/Gothic) font. Rarely supported
    /// by any terminal. Reset with [`NotItalic`](Self::NotItalic) (23).
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **VT Support**: Not supported
    /// - **Reset**: [`NotItalic`](Self::NotItalic) (23) or [`Reset`](Self::Reset) (0)
    Fraktur = 20,

    // =========================================================================
    // Attribute Reset Codes (21-29)
    // =========================================================================
    /// Double underline (`SGR 21`).
    ///
    /// Renders text with a double underline (two parallel lines).
    /// In ECMA-48 3rd edition, this was specified as "doubly underlined".
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **VT Support**: VT300+
    /// - **Reset**: [`NotUnderlined`](Self::NotUnderlined) (24) or [`Reset`](Self::Reset) (0)
    DoubleUnderline = 21,

    /// Normal intensity (`SGR 22`) - neither bold nor faint.
    ///
    /// Resets both [`Bold`](Self::Bold) (1) and [`Faint`](Self::Faint) (2).
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **VT Support**: VT300+
    NormalIntensity = 22,

    /// Not italic, not fraktur (`SGR 23`).
    ///
    /// Resets both [`Italic`](Self::Italic) (3) and [`Fraktur`](Self::Fraktur) (20).
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **VT Support**: Not widely supported by DEC terminals
    NotItalic = 23,

    /// Not underlined (`SGR 24`).
    ///
    /// Resets all underline styles including single (4), double (21),
    /// and extended styles (4:1-4:5).
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **VT Support**: VT300+
    NotUnderlined = 24,

    /// Not blinking (`SGR 25`).
    ///
    /// Resets both [`Blink`](Self::Blink) (5) and [`RapidBlink`](Self::RapidBlink) (6).
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **VT Support**: VT300+
    NotBlinking = 25,

    /// Proportional spacing (`SGR 26`).
    ///
    /// Enables proportional (variable-width) spacing. Rarely supported
    /// as most terminals use fixed-width character cells.
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **VT Support**: Not supported
    /// - **Reset**: [`NotProportionalSpacing`](Self::NotProportionalSpacing) (50)
    ProportionalSpacing = 26,

    /// Not inverse/positive image (`SGR 27`).
    ///
    /// Resets [`Inverse`](Self::Inverse) (7).
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **VT Support**: VT300+
    NotInverse = 27,

    /// Revealed/not hidden (`SGR 28`).
    ///
    /// Resets [`Hidden`](Self::Hidden) (8), making text visible again.
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **VT Support**: VT300+
    NotHidden = 28,

    /// Not crossed out (`SGR 29`).
    ///
    /// Resets [`Strikethrough`](Self::Strikethrough) (9).
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **VT Support**: Not widely supported by DEC terminals
    NotStrikethrough = 29,

    // =========================================================================
    // Standard Foreground Colors (30-37, 39) - ECMA-48 2nd Edition
    // =========================================================================
    /// Foreground black (`SGR 30`).
    ///
    /// Sets the foreground (text) color to black (ANSI color 0).
    ForegroundBlack = 30,
    /// Foreground red (`SGR 31`).
    ///
    /// Sets the foreground (text) color to red (ANSI color 1).
    ForegroundRed = 31,
    /// Foreground green (`SGR 32`).
    ///
    /// Sets the foreground (text) color to green (ANSI color 2).
    ForegroundGreen = 32,
    /// Foreground yellow (`SGR 33`).
    ///
    /// Sets the foreground (text) color to yellow (ANSI color 3).
    ForegroundYellow = 33,
    /// Foreground blue (`SGR 34`).
    ///
    /// Sets the foreground (text) color to blue (ANSI color 4).
    ForegroundBlue = 34,
    /// Foreground magenta (`SGR 35`).
    ///
    /// Sets the foreground (text) color to magenta (ANSI color 5).
    ForegroundMagenta = 35,
    /// Foreground cyan (`SGR 36`).
    ///
    /// Sets the foreground (text) color to cyan (ANSI color 6).
    ForegroundCyan = 36,
    /// Foreground white (`SGR 37`).
    ///
    /// Sets the foreground (text) color to white (ANSI color 7).
    ForegroundWhite = 37,
    // Note: 38 is extended foreground color, handled by SgrAttr::Foreground
    /// Default foreground color (`SGR 39`).
    ///
    /// Resets the foreground color to the terminal's default.
    /// The actual color depends on the terminal's color scheme.
    ///
    /// - **ECMA-48**: 3rd edition (1984), "implementation defined"
    ForegroundDefault = 39,

    // =========================================================================
    // Standard Background Colors (40-47, 49) - ECMA-48 2nd Edition
    // =========================================================================
    /// Background black (`SGR 40`).
    ///
    /// Sets the background color to black (ANSI color 0).
    BackgroundBlack = 40,
    /// Background red (`SGR 41`).
    ///
    /// Sets the background color to red (ANSI color 1).
    BackgroundRed = 41,
    /// Background green (`SGR 42`).
    ///
    /// Sets the background color to green (ANSI color 2).
    BackgroundGreen = 42,
    /// Background yellow (`SGR 43`).
    ///
    /// Sets the background color to yellow (ANSI color 3).
    BackgroundYellow = 43,
    /// Background blue (`SGR 44`).
    ///
    /// Sets the background color to blue (ANSI color 4).
    BackgroundBlue = 44,
    /// Background magenta (`SGR 45`).
    ///
    /// Sets the background color to magenta (ANSI color 5).
    BackgroundMagenta = 45,
    /// Background cyan (`SGR 46`).
    ///
    /// Sets the background color to cyan (ANSI color 6).
    BackgroundCyan = 46,
    /// Background white (`SGR 47`).
    ///
    /// Sets the background color to white (ANSI color 7).
    BackgroundWhite = 47,
    // Note: 48 is extended background color, handled by SgrAttr::Background
    /// Default background color (`SGR 49`).
    ///
    /// Resets the background color to the terminal's default.
    /// The actual color depends on the terminal's color scheme.
    ///
    /// - **ECMA-48**: 3rd edition (1984), "implementation defined"
    BackgroundDefault = 49,

    // =========================================================================
    // Additional Attributes (50-55) - ECMA-48 3rd Edition
    // =========================================================================
    /// Disable proportional spacing (`SGR 50`).
    ///
    /// Resets [`ProportionalSpacing`](Self::ProportionalSpacing) (26).
    /// Rarely supported.
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    NotProportionalSpacing = 50,

    /// Framed text (`SGR 51`).
    ///
    /// Draws a frame/border around the text. Rarely supported by terminals.
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **Reset**: [`NotFramedNotEncircled`](Self::NotFramedNotEncircled) (54)
    Framed = 51,

    /// Encircled text (`SGR 52`).
    ///
    /// Draws a circle around the text (for CJK characters).
    /// Rarely supported by terminals.
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **Reset**: [`NotFramedNotEncircled`](Self::NotFramedNotEncircled) (54)
    Encircled = 52,

    /// Overlined text (`SGR 53`).
    ///
    /// Renders text with a line above it. Supported by many modern
    /// terminal emulators but not by DEC VT terminals.
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **Reset**: [`NotOverline`](Self::NotOverline) (55)
    Overline = 53,

    /// Not framed, not encircled (`SGR 54`).
    ///
    /// Resets both [`Framed`](Self::Framed) (51) and [`Encircled`](Self::Encircled) (52).
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    NotFramedNotEncircled = 54,

    /// Not overlined (`SGR 55`).
    ///
    /// Resets [`Overline`](Self::Overline) (53).
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    NotOverline = 55,

    // Note: 56-57 are reserved/undefined in ECMA-48
    // Note: 58 is extended underline color, handled by SgrAttr::UnderlineColor
    /// Default underline color (`SGR 59`).
    ///
    /// Resets the underline color to the terminal's default (typically
    /// the same as the foreground color). This is a kitty extension
    /// that has been adopted by other terminals.
    ///
    /// - **Origin**: kitty terminal extension
    /// - **Reset for**: Extended underline color (58:...)
    UnderlineColorDefault = 59,

    // =========================================================================
    // Ideogram Attributes (60-65) - ECMA-48 3rd Edition
    //
    // These are intended for CJK (Chinese, Japanese, Korean) text rendering.
    // Rarely supported by Western terminal emulators.
    // =========================================================================
    /// Ideogram underline or right side line (`SGR 60`).
    ///
    /// For vertical text: draws a line on the right side.
    /// For horizontal text: draws an underline.
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **Reset**: [`IdeogramAttributesOff`](Self::IdeogramAttributesOff) (65)
    IdeogramUnderline = 60,

    /// Ideogram double underline or double right side line (`SGR 61`).
    ///
    /// For vertical text: draws double lines on the right side.
    /// For horizontal text: draws a double underline.
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **Reset**: [`IdeogramAttributesOff`](Self::IdeogramAttributesOff) (65)
    IdeogramDoubleUnderline = 61,

    /// Ideogram overline or left side line (`SGR 62`).
    ///
    /// For vertical text: draws a line on the left side.
    /// For horizontal text: draws an overline.
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **Reset**: [`IdeogramAttributesOff`](Self::IdeogramAttributesOff) (65)
    IdeogramOverline = 62,

    /// Ideogram double overline or double left side line (`SGR 63`).
    ///
    /// For vertical text: draws double lines on the left side.
    /// For horizontal text: draws a double overline.
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **Reset**: [`IdeogramAttributesOff`](Self::IdeogramAttributesOff) (65)
    IdeogramDoubleOverline = 63,
    /// Ideogram stress marking (`SGR 64`).
    ///
    /// Adds emphasis/stress marking to ideogram characters.
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    /// - **Reset**: [`IdeogramAttributesOff`](Self::IdeogramAttributesOff) (65)
    IdeogramStressMarking = 64,

    /// Reset all ideogram attributes (`SGR 65`).
    ///
    /// Resets all ideogram-related attributes (60-64).
    ///
    /// - **ECMA-48**: 3rd edition (1984)
    IdeogramAttributesOff = 65,

    // =========================================================================
    // Superscript/Subscript (73-75) - mintty extension
    //
    // These codes originated in mintty and have been adopted by other
    // terminal emulators including VTE-based terminals and Windows Terminal.
    // Not part of ECMA-48.
    // =========================================================================
    /// Superscript text (`SGR 73`).
    ///
    /// Renders text in a smaller font positioned above the baseline.
    /// Useful for mathematical notation, footnote markers, and ordinals.
    ///
    /// - **Origin**: mintty terminal extension
    /// - **Support**: mintty, VTE-based terminals, Windows Terminal
    /// - **Reset**: [`NotSuperscriptSubscript`](Self::NotSuperscriptSubscript) (75)
    Superscript = 73,

    /// Subscript text (`SGR 74`).
    ///
    /// Renders text in a smaller font positioned below the baseline.
    /// Useful for chemical formulas and mathematical notation.
    ///
    /// - **Origin**: mintty terminal extension
    /// - **Support**: mintty, VTE-based terminals, Windows Terminal
    /// - **Reset**: [`NotSuperscriptSubscript`](Self::NotSuperscriptSubscript) (75)
    Subscript = 74,

    /// Neither superscript nor subscript (`SGR 75`).
    ///
    /// Resets both [`Superscript`](Self::Superscript) (73) and
    /// [`Subscript`](Self::Subscript) (74).
    ///
    /// - **Origin**: mintty terminal extension
    NotSuperscriptSubscript = 75,

    // =========================================================================
    // Bright/Aixterm Foreground Colors (90-97) - Non-standard
    //
    // These "bright" or "high-intensity" colors originated in IBM's AIXterm
    // and have become a de-facto standard. They are NOT part of ECMA-48.
    // The codes 90-97 correspond to bright versions of colors 0-7.
    //
    // Reset with ForegroundDefault (39) or Reset (0).
    // =========================================================================
    /// Bright foreground black (`SGR 90`).
    ///
    /// Sets foreground to bright black (often displayed as dark gray).
    /// Equivalent to 256-color index 8.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`ForegroundDefault`](Self::ForegroundDefault) (39)
    ForegroundBrightBlack = 90,

    /// Bright foreground red (`SGR 91`).
    ///
    /// Sets foreground to bright red. Equivalent to 256-color index 9.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`ForegroundDefault`](Self::ForegroundDefault) (39)
    ForegroundBrightRed = 91,

    /// Bright foreground green (`SGR 92`).
    ///
    /// Sets foreground to bright green. Equivalent to 256-color index 10.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`ForegroundDefault`](Self::ForegroundDefault) (39)
    ForegroundBrightGreen = 92,

    /// Bright foreground yellow (`SGR 93`).
    ///
    /// Sets foreground to bright yellow. Equivalent to 256-color index 11.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`ForegroundDefault`](Self::ForegroundDefault) (39)
    ForegroundBrightYellow = 93,

    /// Bright foreground blue (`SGR 94`).
    ///
    /// Sets foreground to bright blue. Equivalent to 256-color index 12.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`ForegroundDefault`](Self::ForegroundDefault) (39)
    ForegroundBrightBlue = 94,

    /// Bright foreground magenta (`SGR 95`).
    ///
    /// Sets foreground to bright magenta. Equivalent to 256-color index 13.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`ForegroundDefault`](Self::ForegroundDefault) (39)
    ForegroundBrightMagenta = 95,

    /// Bright foreground cyan (`SGR 96`).
    ///
    /// Sets foreground to bright cyan. Equivalent to 256-color index 14.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`ForegroundDefault`](Self::ForegroundDefault) (39)
    ForegroundBrightCyan = 96,

    /// Bright foreground white (`SGR 97`).
    ///
    /// Sets foreground to bright white. Equivalent to 256-color index 15.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`ForegroundDefault`](Self::ForegroundDefault) (39)
    ForegroundBrightWhite = 97,

    // =========================================================================
    // Bright/Aixterm Background Colors (100-107) - Non-standard
    //
    // These are the bright versions of background colors, corresponding to
    // the bright foreground colors 90-97. Also originated in aixterm.
    //
    // Note: In some terminals (rxvt) with 16-color support disabled,
    // SGR 100 is used to reset both foreground and background colors.
    // This usage is rare and conflicts with bright black background.
    //
    // Reset with BackgroundDefault (49) or Reset (0).
    // =========================================================================
    /// Bright background black (`SGR 100`).
    ///
    /// Sets background to bright black (often displayed as dark gray).
    /// Equivalent to 256-color index 8.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`BackgroundDefault`](Self::BackgroundDefault) (49)
    BackgroundBrightBlack = 100,

    /// Bright background red (`SGR 101`).
    ///
    /// Sets background to bright red. Equivalent to 256-color index 9.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`BackgroundDefault`](Self::BackgroundDefault) (49)
    BackgroundBrightRed = 101,

    /// Bright background green (`SGR 102`).
    ///
    /// Sets background to bright green. Equivalent to 256-color index 10.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`BackgroundDefault`](Self::BackgroundDefault) (49)
    BackgroundBrightGreen = 102,

    /// Bright background yellow (`SGR 103`).
    ///
    /// Sets background to bright yellow. Equivalent to 256-color index 11.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`BackgroundDefault`](Self::BackgroundDefault) (49)
    BackgroundBrightYellow = 103,

    /// Bright background blue (`SGR 104`).
    ///
    /// Sets background to bright blue. Equivalent to 256-color index 12.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`BackgroundDefault`](Self::BackgroundDefault) (49)
    BackgroundBrightBlue = 104,

    /// Bright background magenta (`SGR 105`).
    ///
    /// Sets background to bright magenta. Equivalent to 256-color index 13.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`BackgroundDefault`](Self::BackgroundDefault) (49)
    BackgroundBrightMagenta = 105,

    /// Bright background cyan (`SGR 106`).
    ///
    /// Sets background to bright cyan. Equivalent to 256-color index 14.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`BackgroundDefault`](Self::BackgroundDefault) (49)
    BackgroundBrightCyan = 106,

    /// Bright background white (`SGR 107`).
    ///
    /// Sets background to bright white. Equivalent to 256-color index 15.
    ///
    /// - **Origin**: aixterm
    /// - **Reset**: [`BackgroundDefault`](Self::BackgroundDefault) (49)
    BackgroundBrightWhite = 107,
}

impl Default for SimpleSgrCode {
    fn default() -> Self {
        Self::Reset
    }
}

impl SimpleSgrCode {
    /// Create a foreground color from an [`AnsiColor`].
    #[must_use]
    pub const fn foreground(color: AnsiColor) -> Self {
        match color {
            AnsiColor::Black => Self::ForegroundBlack,
            AnsiColor::Red => Self::ForegroundRed,
            AnsiColor::Green => Self::ForegroundGreen,
            AnsiColor::Yellow => Self::ForegroundYellow,
            AnsiColor::Blue => Self::ForegroundBlue,
            AnsiColor::Magenta => Self::ForegroundMagenta,
            AnsiColor::Cyan => Self::ForegroundCyan,
            AnsiColor::White => Self::ForegroundWhite,
        }
    }

    /// Create a background color from an [`AnsiColor`].
    #[must_use]
    pub const fn background(color: AnsiColor) -> Self {
        match color {
            AnsiColor::Black => Self::BackgroundBlack,
            AnsiColor::Red => Self::BackgroundRed,
            AnsiColor::Green => Self::BackgroundGreen,
            AnsiColor::Yellow => Self::BackgroundYellow,
            AnsiColor::Blue => Self::BackgroundBlue,
            AnsiColor::Magenta => Self::BackgroundMagenta,
            AnsiColor::Cyan => Self::BackgroundCyan,
            AnsiColor::White => Self::BackgroundWhite,
        }
    }

    /// Create a bright foreground color from an [`AnsiColor`].
    #[must_use]
    pub const fn foreground_bright(color: AnsiColor) -> Self {
        match color {
            AnsiColor::Black => Self::ForegroundBrightBlack,
            AnsiColor::Red => Self::ForegroundBrightRed,
            AnsiColor::Green => Self::ForegroundBrightGreen,
            AnsiColor::Yellow => Self::ForegroundBrightYellow,
            AnsiColor::Blue => Self::ForegroundBrightBlue,
            AnsiColor::Magenta => Self::ForegroundBrightMagenta,
            AnsiColor::Cyan => Self::ForegroundBrightCyan,
            AnsiColor::White => Self::ForegroundBrightWhite,
        }
    }

    /// Create a bright background color from an [`AnsiColor`].
    #[must_use]
    pub const fn background_bright(color: AnsiColor) -> Self {
        match color {
            AnsiColor::Black => Self::BackgroundBrightBlack,
            AnsiColor::Red => Self::BackgroundBrightRed,
            AnsiColor::Green => Self::BackgroundBrightGreen,
            AnsiColor::Yellow => Self::BackgroundBrightYellow,
            AnsiColor::Blue => Self::BackgroundBrightBlue,
            AnsiColor::Magenta => Self::BackgroundBrightMagenta,
            AnsiColor::Cyan => Self::BackgroundBrightCyan,
            AnsiColor::White => Self::BackgroundBrightWhite,
        }
    }
}

// =============================================================================
// Extended Color
// =============================================================================

/// Extended color specification for SGR 38 (foreground), 48 (background), and 58 (underline).
///
/// Extended colors use a sub-parameter syntax defined in **ISO-8613-6** (ITU-T Rec. T.416).
/// The base code (38, 48, or 58) is followed by a color type and type-specific parameters.
///
/// # Color Types (ISO-8613-6)
///
/// | Type | Format | Description | Terminal Support |
/// |------|--------|-------------|------------------|
/// | 1 | `38:1` | Transparent/default | Limited |
/// | 2 | `38:2::r:g:b` | RGB truecolor | Widely supported |
/// | 3 | `38:3:c:m:y` | CMY | Rarely supported |
/// | 4 | `38:4:c:m:y:k` | CMYK | Rarely supported |
/// | 5 | `38:5:n` | 256-color palette | Widely supported |
///
/// # Encoding Formats
///
/// ## ISO-8613-6 (Colon-separated, Recommended)
///
/// The standard format uses colons to separate sub-parameters. This format
/// is unambiguous because colons create a sub-parameter group that cannot
/// be confused with separate SGR codes.
///
/// ```text
/// CSI 38:5:196 m           # Foreground palette color 196
/// CSI 38:2::255:128:64 m   # Foreground RGB (empty colorspace)
/// CSI 38:2:0:255:128:64 m  # Foreground RGB with colorspace 0
/// CSI 48:5:21 m            # Background palette color 21
/// CSI 58:2::255:0:0 m      # Underline color RGB red
/// ```
///
/// ## Legacy (Semicolon-separated, Compatibility)
///
/// The legacy xterm/konsole format uses semicolons. This format is ambiguous
/// because the parameters could be parsed as separate SGR codes. For example,
/// `38;5;196` could theoretically be parsed as three separate codes.
///
/// ```text
/// CSI 38;5;196 m           # Foreground palette color 196
/// CSI 38;2;255;128;64 m    # Foreground RGB
/// ```
///
/// Most modern terminals handle this correctly, but the colon format is preferred.
///
/// # 256-Color Palette Layout
///
/// The 256-color palette is organized as follows:
///
/// | Range | Description |
/// |-------|-------------|
/// | 0-7 | Standard ANSI colors (same as SGR 30-37) |
/// | 8-15 | Bright ANSI colors (same as SGR 90-97) |
/// | 16-231 | 6×6×6 color cube (216 colors) |
/// | 232-255 | Grayscale (24 shades, black to white) |
///
/// The 6×6×6 color cube formula: `16 + 36*r + 6*g + b` where r, g, b ∈ {0..5}
///
/// # Terminal Support
///
/// | Feature | xterm | kitty | iTerm2 | VTE | Windows Terminal |
/// |---------|-------|-------|--------|-----|------------------|
/// | 256-color (5) | ✓ | ✓ | ✓ | ✓ | ✓ |
/// | RGB (2) | ✓ | ✓ | ✓ | ✓ | ✓ |
/// | Default (1) | ✓ | ✓ | ? | ? | ? |
/// | Colon format | ✓ | ✓ | ✓ | ✓ | ✓ |
/// | Semicolon format | ✓ | ✓ | ✓ | ✓ | ✓ |
/// | CMY/CMYK (3, 4) | No | No | No | No | No |
///
/// # References
///
/// - ISO/IEC 8613-6 (ITU-T Rec. T.416)
/// - xterm ctlseqs: <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html>
/// - termstandard/colors: <https://github.com/termstandard/colors>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtendedColor {
    /// Transparent or default color (ISO-8613-6 color type 1).
    ///
    /// Encoded as `38:1` (foreground), `48:1` (background), or `58:1` (underline).
    ///
    /// This resets the color to the terminal's default. The interpretation
    /// varies by terminal:
    /// - Some treat it as "transparent" (inherit from parent)
    /// - Others treat it as the terminal's configured default color
    ///
    /// This is functionally similar to SGR 39/49/59 but uses the extended
    /// color syntax.
    Default,

    /// 256-color palette index (ISO-8613-6 color type 5).
    ///
    /// Encoded as `38:5:n` where n is 0-255.
    ///
    /// The palette is divided into:
    /// - **0-7**: Standard colors (black, red, green, yellow, blue, magenta, cyan, white)
    /// - **8-15**: Bright/bold colors
    /// - **16-231**: 6×6×6 RGB color cube
    /// - **232-255**: 24-step grayscale
    Palette(u8),

    /// RGB truecolor (ISO-8613-6 color type 2).
    ///
    /// Encoded as `38:2::r:g:b` (with empty colorspace) or `38:2:cs:r:g:b`
    /// (with colorspace identifier, which is typically ignored).
    ///
    /// Each component (r, g, b) is in the range 0-255, providing 16.7 million
    /// possible colors (24-bit color).
    ///
    /// Note: Terminals without direct color support may map RGB values to
    /// the nearest color in their palette.
    Rgb { r: u8, g: u8, b: u8 },

    /// CMY color (ISO-8613-6 color type 3).
    ///
    /// Encoded as `38:3:c:m:y` where c, m, y are in the range 0-255.
    ///
    /// CMY (Cyan, Magenta, Yellow) is a subtractive color model used in
    /// printing. Values represent ink amounts: 0 = no ink, 255 = maximum ink.
    ///
    /// **Warning**: This color type is defined in ISO-8613-6 but is
    /// virtually never supported by terminal emulators.
    Cmy { c: u8, m: u8, y: u8 },

    /// CMYK color (ISO-8613-6 color type 4).
    ///
    /// Encoded as `38:4:c:m:y:k` where c, m, y, k are in the range 0-255.
    ///
    /// CMYK (Cyan, Magenta, Yellow, Key/Black) is a subtractive color model
    /// used in printing. Values represent ink amounts: 0 = no ink, 255 = maximum ink.
    ///
    /// **Warning**: This color type is defined in ISO-8613-6 but is
    /// virtually never supported by terminal emulators.
    Cmyk { c: u8, m: u8, y: u8, k: u8 },
}

impl ExtendedColor {
    /// Create a transparent/default color (type 1).
    ///
    /// This creates a color that resets to the terminal's default.
    /// Encoded as `38:1`, `48:1`, or `58:1` depending on context.
    #[must_use]
    pub const fn default_color() -> Self {
        Self::Default
    }

    /// Create a 256-color palette color (type 5).
    ///
    /// # Arguments
    ///
    /// * `index` - Palette index (0-255)
    ///
    /// # Palette Layout
    ///
    /// - `0-7`: Standard ANSI colors
    /// - `8-15`: Bright ANSI colors
    /// - `16-231`: 6×6×6 color cube (`16 + 36*r + 6*g + b`)
    /// - `232-255`: Grayscale ramp
    #[must_use]
    pub const fn palette(index: u8) -> Self {
        Self::Palette(index)
    }

    /// Create an RGB truecolor (type 2).
    ///
    /// # Arguments
    ///
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    ///
    /// # Example
    ///
    /// ```
    /// use vtio::event::sgr::ExtendedColor;
    ///
    /// let orange = ExtendedColor::rgb(255, 165, 0);
    /// let purple = ExtendedColor::rgb(128, 0, 128);
    /// ```
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb { r, g, b }
    }

    /// Create a CMY color (type 3).
    ///
    /// **Warning**: CMY colors are rarely supported by terminal emulators.
    ///
    /// # Arguments
    ///
    /// * `c` - Cyan component (0-255, where 0 = no ink, 255 = maximum ink)
    /// * `m` - Magenta component (0-255)
    /// * `y` - Yellow component (0-255)
    #[must_use]
    pub const fn cmy(c: u8, m: u8, y: u8) -> Self {
        Self::Cmy { c, m, y }
    }

    /// Create a CMYK color (type 4).
    ///
    /// **Warning**: CMYK colors are rarely supported by terminal emulators.
    ///
    /// # Arguments
    ///
    /// * `c` - Cyan component (0-255, where 0 = no ink, 255 = maximum ink)
    /// * `m` - Magenta component (0-255)
    /// * `y` - Yellow component (0-255)
    /// * `k` - Key/black component (0-255)
    #[must_use]
    pub const fn cmyk(c: u8, m: u8, y: u8, k: u8) -> Self {
        Self::Cmyk { c, m, y, k }
    }

    /// Encode this color with the given base code (38, 48, or 58).
    /// Encode extended color using colon-separated format (ISO-8613-6 standard).
    ///
    /// This is the preferred encoding format as it's unambiguous and supports
    /// all color types including colorspace identifiers.
    ///
    /// Examples:
    /// - `38:1` (default/transparent)
    /// - `38:5:196` (256-color palette)
    /// - `38:2::255:128:64` (RGB, empty colorspace)
    /// - `38:3:100:150:200` (CMY)
    /// - `38:4:10:20:30:40` (CMYK)
    fn encode_with_base<W: Write + ?Sized>(
        self,
        sink: &mut W,
        base: u16,
    ) -> Result<usize, vtansi::EncodeError> {
        use vtansi::encode::write_int;

        let mut written = write_int(sink, base)?;
        match self {
            Self::Default => {
                written +=
                    sink.write(b":1").map_err(vtansi::EncodeError::IOError)?;
            }
            Self::Palette(n) => {
                written +=
                    sink.write(b":5:").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, n)?;
            }
            Self::Rgb { r, g, b } => {
                // Use empty colorspace (::) for compatibility
                written += sink
                    .write(b":2::")
                    .map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, r)?;
                written +=
                    sink.write(b":").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, g)?;
                written +=
                    sink.write(b":").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, b)?;
            }
            Self::Cmy { c, m, y } => {
                written +=
                    sink.write(b":3:").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, c)?;
                written +=
                    sink.write(b":").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, m)?;
                written +=
                    sink.write(b":").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, y)?;
            }
            Self::Cmyk { c, m, y, k } => {
                written +=
                    sink.write(b":4:").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, c)?;
                written +=
                    sink.write(b":").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, m)?;
                written +=
                    sink.write(b":").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, y)?;
                written +=
                    sink.write(b":").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, k)?;
            }
        }
        Ok(written)
    }

    /// Encode extended color using semicolon-separated format (legacy/xterm).
    ///
    /// This format is supported by older terminals but is technically ambiguous
    /// as the parameters could be parsed as separate SGR codes.
    ///
    /// Note: Default, CMY, and CMYK colors are not supported in this format
    /// and will fall back to colon encoding.
    ///
    /// Examples:
    /// - `38;5;196` (256-color palette)
    /// - `38;2;255;128;64` (RGB)
    fn encode_with_base_semicolons<W: Write + ?Sized>(
        self,
        sink: &mut W,
        base: u16,
    ) -> Result<usize, vtansi::EncodeError> {
        use vtansi::encode::write_int;

        let mut written = write_int(sink, base)?;
        match self {
            Self::Palette(n) => {
                written +=
                    sink.write(b";5;").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, n)?;
            }
            Self::Rgb { r, g, b } => {
                written +=
                    sink.write(b";2;").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, r)?;
                written +=
                    sink.write(b";").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, g)?;
                written +=
                    sink.write(b";").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, b)?;
            }
            // Default, CMY, and CMYK are only defined in ISO standard (colon format)
            // We already wrote the base, so just add the colon-format suffix
            Self::Default => {
                written +=
                    sink.write(b":1").map_err(vtansi::EncodeError::IOError)?;
            }
            Self::Cmy { c, m, y } => {
                written +=
                    sink.write(b":3:").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, c)?;
                written +=
                    sink.write(b":").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, m)?;
                written +=
                    sink.write(b":").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, y)?;
            }
            Self::Cmyk { c, m, y, k } => {
                written +=
                    sink.write(b":4:").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, c)?;
                written +=
                    sink.write(b":").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, m)?;
                written +=
                    sink.write(b":").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, y)?;
                written +=
                    sink.write(b":").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, k)?;
            }
        }
        Ok(written)
    }

    /// Parse extended color from colon-separated parts.
    ///
    /// Handles formats like:
    /// - `38:1` (transparent/default)
    /// - `38:5:n` (256-color)
    /// - `38:2:r:g:b` (RGB without colorspace)
    /// - `38:2:cs:r:g:b` (RGB with colorspace, cs may be empty)
    /// - `38:3:c:m:y` (CMY)
    /// - `38:4:c:m:y:k` (CMYK)
    fn parse_from_parts(parts: &[&str]) -> Option<Self> {
        let color_type: u8 = parts.get(1).and_then(|p| p.parse().ok())?;
        match color_type {
            1 => {
                // Transparent/default: base:1
                Some(Self::Default)
            }
            2 => {
                // RGB: base:2:r:g:b or base:2:colorspace:r:g:b
                let (r, g, b) = if parts.len() >= 6 {
                    // Format: base:2:colorspace:r:g:b (colorspace may be empty)
                    let r: u8 = parts.get(3).and_then(|p| p.parse().ok())?;
                    let g: u8 = parts.get(4).and_then(|p| p.parse().ok())?;
                    let b: u8 = parts.get(5).and_then(|p| p.parse().ok())?;
                    (r, g, b)
                } else if parts.len() >= 5 {
                    // Format: base:2:r:g:b (no colorspace)
                    let r: u8 = parts.get(2).and_then(|p| p.parse().ok())?;
                    let g: u8 = parts.get(3).and_then(|p| p.parse().ok())?;
                    let b: u8 = parts.get(4).and_then(|p| p.parse().ok())?;
                    (r, g, b)
                } else {
                    return None;
                };
                Some(Self::Rgb { r, g, b })
            }
            3 => {
                // CMY: base:3:c:m:y
                if parts.len() < 5 {
                    return None;
                }
                let c: u8 = parts.get(2).and_then(|p| p.parse().ok())?;
                let m: u8 = parts.get(3).and_then(|p| p.parse().ok())?;
                let y: u8 = parts.get(4).and_then(|p| p.parse().ok())?;
                Some(Self::Cmy { c, m, y })
            }
            4 => {
                // CMYK: base:4:c:m:y:k
                if parts.len() < 6 {
                    return None;
                }
                let c: u8 = parts.get(2).and_then(|p| p.parse().ok())?;
                let m: u8 = parts.get(3).and_then(|p| p.parse().ok())?;
                let y: u8 = parts.get(4).and_then(|p| p.parse().ok())?;
                let k: u8 = parts.get(5).and_then(|p| p.parse().ok())?;
                Some(Self::Cmyk { c, m, y, k })
            }
            5 => {
                // 256-color: base:5:n
                let n: u8 = parts.get(2).and_then(|p| p.parse().ok())?;
                Some(Self::Palette(n))
            }
            _ => None,
        }
    }

    /// Parse extended color from semicolon-separated iterator parameters.
    ///
    /// Handles formats like:
    /// - `38;1` (transparent/default)
    /// - `38;5;n` (256-color, xterm style)
    /// - `38;2;r;g;b` (RGB, xterm style)
    /// - `38;3;c;m;y` (CMY)
    /// - `38;4;c;m;y;k` (CMYK)
    ///
    /// This is called after the base code (38, 48, 58) has been consumed.
    /// The iterator should be positioned at the color type parameter.
    fn parse_from_iter<'a, I>(iter: &mut I) -> Option<Self>
    where
        I: Iterator<Item = &'a [u8]>,
    {
        let color_type_bytes = iter.next()?;
        let color_type: u8 = std::str::from_utf8(color_type_bytes)
            .ok()
            .and_then(|s| s.parse().ok())?;

        match color_type {
            1 => {
                // Transparent/default: 1
                Some(Self::Default)
            }
            2 => {
                // RGB: 2;r;g;b
                let r_bytes = iter.next()?;
                let g_bytes = iter.next()?;
                let b_bytes = iter.next()?;
                let r: u8 = std::str::from_utf8(r_bytes)
                    .ok()
                    .and_then(|s| s.parse().ok())?;
                let g: u8 = std::str::from_utf8(g_bytes)
                    .ok()
                    .and_then(|s| s.parse().ok())?;
                let b: u8 = std::str::from_utf8(b_bytes)
                    .ok()
                    .and_then(|s| s.parse().ok())?;
                Some(Self::Rgb { r, g, b })
            }
            3 => {
                // CMY: 3;c;m;y
                let c_bytes = iter.next()?;
                let m_bytes = iter.next()?;
                let y_bytes = iter.next()?;
                let c: u8 = std::str::from_utf8(c_bytes)
                    .ok()
                    .and_then(|s| s.parse().ok())?;
                let m: u8 = std::str::from_utf8(m_bytes)
                    .ok()
                    .and_then(|s| s.parse().ok())?;
                let y: u8 = std::str::from_utf8(y_bytes)
                    .ok()
                    .and_then(|s| s.parse().ok())?;
                Some(Self::Cmy { c, m, y })
            }
            4 => {
                // CMYK: 4;c;m;y;k
                let c_bytes = iter.next()?;
                let m_bytes = iter.next()?;
                let y_bytes = iter.next()?;
                let k_bytes = iter.next()?;
                let c: u8 = std::str::from_utf8(c_bytes)
                    .ok()
                    .and_then(|s| s.parse().ok())?;
                let m: u8 = std::str::from_utf8(m_bytes)
                    .ok()
                    .and_then(|s| s.parse().ok())?;
                let y: u8 = std::str::from_utf8(y_bytes)
                    .ok()
                    .and_then(|s| s.parse().ok())?;
                let k: u8 = std::str::from_utf8(k_bytes)
                    .ok()
                    .and_then(|s| s.parse().ok())?;
                Some(Self::Cmyk { c, m, y, k })
            }
            5 => {
                // 256-color: 5;n
                let n_bytes = iter.next()?;
                let n: u8 = std::str::from_utf8(n_bytes)
                    .ok()
                    .and_then(|s| s.parse().ok())?;
                Some(Self::Palette(n))
            }
            _ => None,
        }
    }
}

// =============================================================================
// SGR Attribute Enum
// =============================================================================

/// A single SGR (Select Graphic Rendition) attribute.
///
/// This is the primary type for representing SGR attributes. It unifies:
///
/// - **Simple codes** ([`SimpleSgrCode`]): Single-parameter attributes like bold, italic, colors
/// - **Extended colors** ([`ExtendedColor`]): 256-color palette and RGB truecolor
/// - **Underline styles** ([`UnderlineStyle`]): Curly, dotted, dashed underlines (kitty extension)
/// - **Alternative fonts**: Font selection (SGR 11-19)
///
/// # Usage Patterns
///
/// ## Basic Attributes
///
/// Use the associated constants for common attributes:
///
/// ```
/// use vtio::event::sgr::SgrAttr;
///
/// let bold = SgrAttr::Bold;           // SGR 1
/// let italic = SgrAttr::Italic;       // SGR 3
/// let underline = SgrAttr::Underline; // SGR 4
/// let reset = SgrAttr::Reset;         // SGR 0
/// ```
///
/// ## Standard Colors
///
/// Use the color constants or helper methods:
///
/// ```
/// use vtio::event::sgr::{SgrAttr, AnsiColor};
///
/// // Using constants
/// let red_fg = SgrAttr::ForegroundRed;
/// let blue_bg = SgrAttr::BackgroundBlue;
///
/// // Using helper methods
/// let green_fg = SgrAttr::foreground(AnsiColor::Green);
/// let bright_yellow = SgrAttr::foreground_bright(AnsiColor::Yellow);
/// ```
///
/// ## Extended Colors (256-color and RGB)
///
/// ```
/// use vtio::event::sgr::SgrAttr;
///
/// // 256-color palette
/// let orange_fg = SgrAttr::foreground_256(208);  // CSI 38:5:208 m
/// let purple_bg = SgrAttr::background_256(93);   // CSI 48:5:93 m
///
/// // RGB truecolor
/// let coral_fg = SgrAttr::foreground_rgb(255, 127, 80);  // CSI 38:2::255:127:80 m
/// let navy_bg = SgrAttr::background_rgb(0, 0, 128);      // CSI 48:2::0:0:128 m
/// ```
///
/// ## Underline Styles (kitty extension)
///
/// ```
/// use vtio::event::sgr::{SgrAttr, UnderlineStyle, ExtendedColor};
///
/// // Curly underline (undercurl) - commonly used for spell checking
/// let wavy = SgrAttr::UnderlineStyle(UnderlineStyle::Curly);  // CSI 4:3 m
///
/// // Colored underline
/// let red_underline = SgrAttr::UnderlineColor(ExtendedColor::rgb(255, 0, 0));
/// ```
///
/// # Encoding
///
/// By default, `SgrAttr` encodes extended colors using the ISO-8613-6
/// colon-separated format (e.g., `38:5:196`). For legacy semicolon format
/// (e.g., `38;5;196`), wrap in [`LegacySgrAttr`]:
///
/// ```
/// use vtio::event::sgr::{SgrAttr, LegacySgrAttr};
/// use vtansi::AnsiEncode;
///
/// let attr = SgrAttr::foreground_256(196);
///
/// // ISO-8613-6 format (recommended)
/// let mut buf = Vec::new();
/// attr.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"38:5:196");
///
/// // Legacy xterm format
/// let mut buf = Vec::new();
/// LegacySgrAttr(attr).encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"38;5;196");
/// ```
///
/// # Variant Categories
///
/// | Variant | SGR Codes | Description |
/// |---------|-----------|-------------|
/// | `Simple` | 0-10, 20-29, 30-37, 39-55, 59-65, 73-75, 90-107 | Single-parameter codes |
/// | `AlternativeFont` | 11-19 | Alternative font selection |
/// | `Foreground` | 38:... | Extended foreground colors |
/// | `Background` | 48:... | Extended background colors |
/// | `UnderlineStyle` | 4:0-5 | Extended underline styles |
/// | `UnderlineColor` | 58:... | Underline color |
/// | `Unknown` | Any | Unrecognized SGR codes |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SgrAttr {
    /// A simple single-parameter SGR code.
    ///
    /// This covers all SGR codes that are encoded as a single integer value
    /// without sub-parameters. See [`SimpleSgrCode`] for the full list.
    ///
    /// Most common attributes (bold, italic, colors, etc.) have corresponding
    /// associated constants on `SgrAttr` for convenience.
    Simple(SimpleSgrCode),

    /// Alternative font selection (`SGR 11-19`).
    ///
    /// Selects alternative font 1-9, where the value is the font number (1-9).
    /// Font 1 corresponds to SGR 11, font 2 to SGR 12, etc.
    ///
    /// Reset with [`SimpleSgrCode::PrimaryFont`] (SGR 10).
    ///
    /// **Note**: Alternative fonts are rarely supported by terminal emulators.
    AlternativeFont(u8),

    /// Extended foreground color (`SGR 38:...`).
    ///
    /// Supports 256-color palette, RGB truecolor, and other ISO-8613-6 color types.
    /// See [`ExtendedColor`] for details.
    ///
    /// Reset with [`SimpleSgrCode::ForegroundDefault`] (SGR 39).
    Foreground(ExtendedColor),

    /// Extended background color (`SGR 48:...`).
    ///
    /// Supports 256-color palette, RGB truecolor, and other ISO-8613-6 color types.
    /// See [`ExtendedColor`] for details.
    ///
    /// Reset with [`SimpleSgrCode::BackgroundDefault`] (SGR 49).
    Background(ExtendedColor),

    /// Extended underline style (`SGR 4:Ps`).
    ///
    /// Provides underline style variants beyond the basic single underline:
    /// curly (undercurl), dotted, dashed, and double underlines.
    ///
    /// This is a **`kitty` terminal extension** now supported by many modern
    /// terminal emulators including `iTerm2`, `WezTerm`, and `Alacritty`.
    ///
    /// Reset with [`SimpleSgrCode::NotUnderlined`] (SGR 24).
    UnderlineStyle(UnderlineStyle),

    /// Extended underline color (`SGR 58:...`).
    ///
    /// Sets the color of the underline independently from the text color.
    /// Supports the same color types as foreground/background colors.
    ///
    /// This is a **kitty terminal extension**.
    ///
    /// Reset with [`SimpleSgrCode::UnderlineColorDefault`] (SGR 59).
    UnderlineColor(ExtendedColor),

    /// Unknown or unsupported SGR parameter.
    ///
    /// Used for SGR codes that are not recognized. The value is the
    /// raw numeric parameter. This allows round-tripping of unknown
    /// codes without data loss.
    ///
    /// Unknown codes in ECMA-48 include: 56-57, 66-72, 76-89.
    Unknown(u16),
}

// Re-export common variants for convenience
#[allow(non_upper_case_globals)]
impl SgrAttr {
    // =========================================================================
    // Simple code constants (most common ones)
    // =========================================================================

    /// Reset all attributes to default (`SGR 0`).
    pub const Reset: Self = Self::Simple(SimpleSgrCode::Reset);
    /// Bold or increased intensity (`SGR 1`).
    pub const Bold: Self = Self::Simple(SimpleSgrCode::Bold);
    /// Faint or decreased intensity (`SGR 2`).
    pub const Faint: Self = Self::Simple(SimpleSgrCode::Faint);
    /// Normal intensity - neither bold nor faint (`SGR 22`).
    pub const NormalIntensity: Self =
        Self::Simple(SimpleSgrCode::NormalIntensity);
    /// Italic (`SGR 3`).
    pub const Italic: Self = Self::Simple(SimpleSgrCode::Italic);
    /// Not italic (`SGR 23`).
    pub const NotItalic: Self = Self::Simple(SimpleSgrCode::NotItalic);
    /// Single underline (`SGR 4`).
    pub const Underline: Self = Self::Simple(SimpleSgrCode::Underline);
    /// Double underline (`SGR 21`).
    pub const DoubleUnderline: Self =
        Self::Simple(SimpleSgrCode::DoubleUnderline);
    /// Not underlined (`SGR 24`).
    pub const NotUnderlined: Self = Self::Simple(SimpleSgrCode::NotUnderlined);
    /// Slow blink (`SGR 5`).
    pub const Blink: Self = Self::Simple(SimpleSgrCode::Blink);
    /// Rapid blink (`SGR 6`).
    pub const RapidBlink: Self = Self::Simple(SimpleSgrCode::RapidBlink);
    /// Not blinking (`SGR 25`).
    pub const NotBlinking: Self = Self::Simple(SimpleSgrCode::NotBlinking);
    /// Inverse/reverse video (`SGR 7`).
    pub const Inverse: Self = Self::Simple(SimpleSgrCode::Inverse);
    /// Not inverse (`SGR 27`).
    pub const NotInverse: Self = Self::Simple(SimpleSgrCode::NotInverse);
    /// Hidden/invisible (`SGR 8`).
    pub const Hidden: Self = Self::Simple(SimpleSgrCode::Hidden);
    /// Not hidden/visible (`SGR 28`).
    pub const NotHidden: Self = Self::Simple(SimpleSgrCode::NotHidden);
    /// Crossed out/strikethrough (`SGR 9`).
    pub const Strikethrough: Self = Self::Simple(SimpleSgrCode::Strikethrough);
    /// Not crossed out (`SGR 29`).
    pub const NotStrikethrough: Self =
        Self::Simple(SimpleSgrCode::NotStrikethrough);
    /// Primary (default) font (`SGR 10`).
    pub const PrimaryFont: Self = Self::Simple(SimpleSgrCode::PrimaryFont);
    /// Fraktur/Gothic font (`SGR 20`).
    pub const Fraktur: Self = Self::Simple(SimpleSgrCode::Fraktur);
    /// Proportional spacing (`SGR 26`).
    pub const ProportionalSpacing: Self =
        Self::Simple(SimpleSgrCode::ProportionalSpacing);
    /// Disable proportional spacing (`SGR 50`).
    pub const NotProportionalSpacing: Self =
        Self::Simple(SimpleSgrCode::NotProportionalSpacing);
    /// Framed (`SGR 51`).
    pub const Framed: Self = Self::Simple(SimpleSgrCode::Framed);
    /// Encircled (`SGR 52`).
    pub const Encircled: Self = Self::Simple(SimpleSgrCode::Encircled);
    /// Not framed, not encircled (`SGR 54`).
    pub const NotFramedNotEncircled: Self =
        Self::Simple(SimpleSgrCode::NotFramedNotEncircled);
    /// Overlined (`SGR 53`).
    pub const Overline: Self = Self::Simple(SimpleSgrCode::Overline);
    /// Not overlined (`SGR 55`).
    pub const NotOverline: Self = Self::Simple(SimpleSgrCode::NotOverline);
    /// Default underline color (`SGR 59`).
    pub const UnderlineColorDefault: Self =
        Self::Simple(SimpleSgrCode::UnderlineColorDefault);
    /// Ideogram underline (`SGR 60`).
    pub const IdeogramUnderline: Self =
        Self::Simple(SimpleSgrCode::IdeogramUnderline);
    /// Ideogram double underline (`SGR 61`).
    pub const IdeogramDoubleUnderline: Self =
        Self::Simple(SimpleSgrCode::IdeogramDoubleUnderline);
    /// Ideogram overline (`SGR 62`).
    pub const IdeogramOverline: Self =
        Self::Simple(SimpleSgrCode::IdeogramOverline);
    /// Ideogram double overline (`SGR 63`).
    pub const IdeogramDoubleOverline: Self =
        Self::Simple(SimpleSgrCode::IdeogramDoubleOverline);
    /// Ideogram stress marking (`SGR 64`).
    pub const IdeogramStressMarking: Self =
        Self::Simple(SimpleSgrCode::IdeogramStressMarking);
    /// Reset ideogram attributes (`SGR 65`).
    pub const IdeogramAttributesOff: Self =
        Self::Simple(SimpleSgrCode::IdeogramAttributesOff);
    /// Superscript (`SGR 73`).
    pub const Superscript: Self = Self::Simple(SimpleSgrCode::Superscript);
    /// Subscript (`SGR 74`).
    pub const Subscript: Self = Self::Simple(SimpleSgrCode::Subscript);
    /// Neither superscript nor subscript (`SGR 75`).
    pub const NotSuperscriptSubscript: Self =
        Self::Simple(SimpleSgrCode::NotSuperscriptSubscript);

    // =========================================================================
    // Standard foreground colors
    // =========================================================================

    /// Foreground black (`SGR 30`).
    pub const ForegroundBlack: Self =
        Self::Simple(SimpleSgrCode::ForegroundBlack);
    /// Foreground red (`SGR 31`).
    pub const ForegroundRed: Self = Self::Simple(SimpleSgrCode::ForegroundRed);
    /// Foreground green (`SGR 32`).
    pub const ForegroundGreen: Self =
        Self::Simple(SimpleSgrCode::ForegroundGreen);
    /// Foreground yellow (`SGR 33`).
    pub const ForegroundYellow: Self =
        Self::Simple(SimpleSgrCode::ForegroundYellow);
    /// Foreground blue (`SGR 34`).
    pub const ForegroundBlue: Self =
        Self::Simple(SimpleSgrCode::ForegroundBlue);
    /// Foreground magenta (`SGR 35`).
    pub const ForegroundMagenta: Self =
        Self::Simple(SimpleSgrCode::ForegroundMagenta);
    /// Foreground cyan (`SGR 36`).
    pub const ForegroundCyan: Self =
        Self::Simple(SimpleSgrCode::ForegroundCyan);
    /// Foreground white (`SGR 37`).
    pub const ForegroundWhite: Self =
        Self::Simple(SimpleSgrCode::ForegroundWhite);
    /// Default foreground color (`SGR 39`).
    pub const ForegroundDefault: Self =
        Self::Simple(SimpleSgrCode::ForegroundDefault);

    // =========================================================================
    // Standard background colors
    // =========================================================================

    /// Background black (`SGR 40`).
    pub const BackgroundBlack: Self =
        Self::Simple(SimpleSgrCode::BackgroundBlack);
    /// Background red (`SGR 41`).
    pub const BackgroundRed: Self = Self::Simple(SimpleSgrCode::BackgroundRed);
    /// Background green (`SGR 42`).
    pub const BackgroundGreen: Self =
        Self::Simple(SimpleSgrCode::BackgroundGreen);
    /// Background yellow (`SGR 43`).
    pub const BackgroundYellow: Self =
        Self::Simple(SimpleSgrCode::BackgroundYellow);
    /// Background blue (`SGR 44`).
    pub const BackgroundBlue: Self =
        Self::Simple(SimpleSgrCode::BackgroundBlue);
    /// Background magenta (`SGR 45`).
    pub const BackgroundMagenta: Self =
        Self::Simple(SimpleSgrCode::BackgroundMagenta);
    /// Background cyan (`SGR 46`).
    pub const BackgroundCyan: Self =
        Self::Simple(SimpleSgrCode::BackgroundCyan);
    /// Background white (`SGR 47`).
    pub const BackgroundWhite: Self =
        Self::Simple(SimpleSgrCode::BackgroundWhite);
    /// Default background color (`SGR 49`).
    pub const BackgroundDefault: Self =
        Self::Simple(SimpleSgrCode::BackgroundDefault);

    // =========================================================================
    // Bright foreground colors
    // =========================================================================

    /// Bright foreground black (`SGR 90`).
    pub const ForegroundBrightBlack: Self =
        Self::Simple(SimpleSgrCode::ForegroundBrightBlack);
    /// Bright foreground red (`SGR 91`).
    pub const ForegroundBrightRed: Self =
        Self::Simple(SimpleSgrCode::ForegroundBrightRed);
    /// Bright foreground green (`SGR 92`).
    pub const ForegroundBrightGreen: Self =
        Self::Simple(SimpleSgrCode::ForegroundBrightGreen);
    /// Bright foreground yellow (`SGR 93`).
    pub const ForegroundBrightYellow: Self =
        Self::Simple(SimpleSgrCode::ForegroundBrightYellow);
    /// Bright foreground blue (`SGR 94`).
    pub const ForegroundBrightBlue: Self =
        Self::Simple(SimpleSgrCode::ForegroundBrightBlue);
    /// Bright foreground magenta (`SGR 95`).
    pub const ForegroundBrightMagenta: Self =
        Self::Simple(SimpleSgrCode::ForegroundBrightMagenta);
    /// Bright foreground cyan (`SGR 96`).
    pub const ForegroundBrightCyan: Self =
        Self::Simple(SimpleSgrCode::ForegroundBrightCyan);
    /// Bright foreground white (`SGR 97`).
    pub const ForegroundBrightWhite: Self =
        Self::Simple(SimpleSgrCode::ForegroundBrightWhite);

    // =========================================================================
    // Bright background colors
    // =========================================================================

    /// Bright background black (`SGR 100`).
    pub const BackgroundBrightBlack: Self =
        Self::Simple(SimpleSgrCode::BackgroundBrightBlack);
    /// Bright background red (`SGR 101`).
    pub const BackgroundBrightRed: Self =
        Self::Simple(SimpleSgrCode::BackgroundBrightRed);
    /// Bright background green (`SGR 102`).
    pub const BackgroundBrightGreen: Self =
        Self::Simple(SimpleSgrCode::BackgroundBrightGreen);
    /// Bright background yellow (`SGR 103`).
    pub const BackgroundBrightYellow: Self =
        Self::Simple(SimpleSgrCode::BackgroundBrightYellow);
    /// Bright background blue (`SGR 104`).
    pub const BackgroundBrightBlue: Self =
        Self::Simple(SimpleSgrCode::BackgroundBrightBlue);
    /// Bright background magenta (`SGR 105`).
    pub const BackgroundBrightMagenta: Self =
        Self::Simple(SimpleSgrCode::BackgroundBrightMagenta);
    /// Bright background cyan (`SGR 106`).
    pub const BackgroundBrightCyan: Self =
        Self::Simple(SimpleSgrCode::BackgroundBrightCyan);
    /// Bright background white (`SGR 107`).
    pub const BackgroundBrightWhite: Self =
        Self::Simple(SimpleSgrCode::BackgroundBrightWhite);

    // =========================================================================
    // Helper constructors for colors
    // =========================================================================

    /// Create a standard foreground color from an [`AnsiColor`].
    ///
    /// This creates one of the 8 basic ANSI foreground colors (SGR 30-37).
    ///
    /// # Example
    ///
    /// ```
    /// use vtio::event::sgr::{SgrAttr, AnsiColor};
    ///
    /// let red = SgrAttr::foreground(AnsiColor::Red);    // SGR 31
    /// let blue = SgrAttr::foreground(AnsiColor::Blue);  // SGR 34
    /// ```
    #[must_use]
    pub const fn foreground(color: AnsiColor) -> Self {
        Self::Simple(SimpleSgrCode::foreground(color))
    }

    /// Create a standard background color from an [`AnsiColor`].
    ///
    /// This creates one of the 8 basic ANSI background colors (SGR 40-47).
    ///
    /// # Example
    ///
    /// ```
    /// use vtio::event::sgr::{SgrAttr, AnsiColor};
    ///
    /// let red_bg = SgrAttr::background(AnsiColor::Red);    // SGR 41
    /// let blue_bg = SgrAttr::background(AnsiColor::Blue);  // SGR 44
    /// ```
    #[must_use]
    pub const fn background(color: AnsiColor) -> Self {
        Self::Simple(SimpleSgrCode::background(color))
    }

    /// Create a bright foreground color from an [`AnsiColor`].
    ///
    /// This creates one of the 8 bright/bold ANSI foreground colors (SGR 90-97).
    /// These are the aixterm high-intensity color extension.
    ///
    /// # Example
    ///
    /// ```
    /// use vtio::event::sgr::{SgrAttr, AnsiColor};
    ///
    /// let bright_red = SgrAttr::foreground_bright(AnsiColor::Red);    // SGR 91
    /// let bright_blue = SgrAttr::foreground_bright(AnsiColor::Blue);  // SGR 94
    /// ```
    #[must_use]
    pub const fn foreground_bright(color: AnsiColor) -> Self {
        Self::Simple(SimpleSgrCode::foreground_bright(color))
    }

    /// Create a bright background color from an [`AnsiColor`].
    ///
    /// This creates one of the 8 bright/bold ANSI background colors (SGR 100-107).
    /// These are the aixterm high-intensity color extension.
    ///
    /// # Example
    ///
    /// ```
    /// use vtio::event::sgr::{SgrAttr, AnsiColor};
    ///
    /// let bright_red_bg = SgrAttr::background_bright(AnsiColor::Red);    // SGR 101
    /// let bright_blue_bg = SgrAttr::background_bright(AnsiColor::Blue);  // SGR 104
    /// ```
    #[must_use]
    pub const fn background_bright(color: AnsiColor) -> Self {
        Self::Simple(SimpleSgrCode::background_bright(color))
    }

    /// Create a 256-color palette foreground.
    ///
    /// This creates an extended foreground color using the 256-color palette
    /// (SGR 38:5:n in ISO-8613-6 format).
    ///
    /// # Arguments
    ///
    /// * `index` - Palette index (0-255). See [`ExtendedColor::Palette`] for
    ///   the palette layout.
    ///
    /// # Example
    ///
    /// ```
    /// use vtio::event::sgr::SgrAttr;
    ///
    /// let orange = SgrAttr::foreground_256(208); // Orange from color cube
    /// let gray = SgrAttr::foreground_256(244);     // Light gray from grayscale
    /// ```
    #[must_use]
    pub const fn foreground_256(index: u8) -> Self {
        Self::Foreground(ExtendedColor::Palette(index))
    }

    /// Create a 256-color palette background.
    ///
    /// This creates an extended background color using the 256-color palette
    /// (SGR 48:5:n in ISO-8613-6 format).
    ///
    /// # Arguments
    ///
    /// * `index` - Palette index (0-255). See [`ExtendedColor::Palette`] for
    ///   the palette layout.
    ///
    /// # Example
    ///
    /// ```
    /// use vtio::event::sgr::SgrAttr;
    ///
    /// let purple_bg = SgrAttr::background_256(93);    // Purple from color cube
    /// let dark_gray_bg = SgrAttr::background_256(236); // Dark gray from grayscale
    /// ```
    #[must_use]
    pub const fn background_256(index: u8) -> Self {
        Self::Background(ExtendedColor::Palette(index))
    }

    /// Create an RGB truecolor foreground (24-bit color).
    ///
    /// This creates an extended foreground color using RGB values
    /// (`SGR 38:2::r:g:b` in `ISO-8613-6` format).
    ///
    /// # Arguments
    ///
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    ///
    /// # Example
    ///
    /// ```
    /// use vtio::event::sgr::SgrAttr;
    ///
    /// let coral = SgrAttr::foreground_rgb(255, 127, 80);
    /// let teal = SgrAttr::foreground_rgb(0, 128, 128);
    /// ```
    #[must_use]
    pub const fn foreground_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Foreground(ExtendedColor::Rgb { r, g, b })
    }

    /// Create an RGB truecolor background (24-bit color).
    ///
    /// This creates an extended background color using RGB values
    /// (`SGR 48:2::r:g:b` in `ISO-8613-6` format).
    ///
    /// # Arguments
    ///
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    ///
    /// # Example
    ///
    /// ```
    /// use vtio::event::sgr::SgrAttr;
    ///
    /// let navy_bg = SgrAttr::background_rgb(0, 0, 128);
    /// let salmon_bg = SgrAttr::background_rgb(250, 128, 114);
    /// ```
    #[must_use]
    pub const fn background_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Background(ExtendedColor::Rgb { r, g, b })
    }
}

impl Default for SgrAttr {
    fn default() -> Self {
        Self::Reset
    }
}

impl From<SimpleSgrCode> for SgrAttr {
    fn from(code: SimpleSgrCode) -> Self {
        Self::Simple(code)
    }
}

/// SGR attribute with legacy semicolon-separated encoding for extended colors.
///
/// This newtype wrapper changes how extended colors (256-color and RGB) are
/// encoded when serializing to ANSI escape sequences.
///
/// # Background
///
/// There are two formats for encoding extended colors:
///
/// | Format | Example | Standard | Compatibility |
/// |--------|---------|----------|---------------|
/// | Colon (ISO-8613-6) | `38:5:196` | Yes | Modern terminals |
/// | Semicolon (xterm) | `38;5;196` | No | All terminals |
///
/// The semicolon format is technically ambiguous because the parameters
/// could be parsed as separate SGR codes. However, it has wider compatibility
/// with older terminals and terminal emulators.
///
/// # When to Use
///
/// - Use `SgrAttr` (colon format) for modern terminals and when correctness matters
/// - Use `LegacySgrAttr` (semicolon format) for maximum compatibility
///
/// # Behavior
///
/// - **Palette colors** (type 5): `38;5;n` instead of `38:5:n`
/// - **RGB colors** (type 2): `38;2;r;g;b` instead of `38:2::r:g:b`
/// - **Default/CMY/CMYK** (types 1, 3, 4): Still use colons (no semicolon equivalent)
/// - **Underline styles**: Always use colons (`4:3` for curly)
/// - **Simple codes**: Unchanged (single integer)
///
/// # Example
///
/// ```
/// use vtio::event::sgr::{SgrAttr, LegacySgrAttr};
/// use vtansi::AnsiEncode;
///
/// let attr = SgrAttr::foreground_rgb(255, 128, 64);
///
/// // ISO-8613-6 colon format (default, recommended)
/// let mut buf = Vec::new();
/// attr.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"38:2::255:128:64");
///
/// // Legacy xterm semicolon format (wider compatibility)
/// let mut buf = Vec::new();
/// LegacySgrAttr(attr).encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"38;2;255;128;64");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LegacySgrAttr(pub SgrAttr);

impl From<SgrAttr> for LegacySgrAttr {
    #[inline]
    fn from(attr: SgrAttr) -> Self {
        Self(attr)
    }
}

impl From<LegacySgrAttr> for SgrAttr {
    #[inline]
    fn from(wrapper: LegacySgrAttr) -> Self {
        wrapper.0
    }
}

impl vtansi::AnsiEncode for LegacySgrAttr {
    fn encode_ansi_into<W: Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        use vtansi::encode::write_int;

        match &self.0 {
            SgrAttr::Simple(code) => code.encode_ansi_into(sink),
            SgrAttr::AlternativeFont(n) => {
                write_int(sink, 10u16 + u16::from(*n))
            }
            SgrAttr::Foreground(color) => {
                color.encode_with_base_semicolons(sink, 38)
            }
            SgrAttr::Background(color) => {
                color.encode_with_base_semicolons(sink, 48)
            }
            SgrAttr::UnderlineStyle(style) => {
                // Underline style always uses colon format (4:Ps)
                let mut written =
                    sink.write(b"4:").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, u8::from(*style))?;
                Ok(written)
            }
            SgrAttr::UnderlineColor(color) => {
                color.encode_with_base_semicolons(sink, 58)
            }
            SgrAttr::Unknown(code) => write_int(sink, *code),
        }
    }
}

// Implement AnsiEncode for SgrAttr
impl vtansi::AnsiEncode for SgrAttr {
    fn encode_ansi_into<W: Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        use vtansi::encode::write_int;

        match self {
            Self::Simple(code) => code.encode_ansi_into(sink),
            Self::AlternativeFont(n) => write_int(sink, 10u16 + u16::from(*n)),
            Self::Foreground(color) => color.encode_with_base(sink, 38),
            Self::Background(color) => color.encode_with_base(sink, 48),
            Self::UnderlineStyle(style) => {
                let mut written =
                    sink.write(b"4:").map_err(vtansi::EncodeError::IOError)?;
                written += write_int(sink, u8::from(*style))?;
                Ok(written)
            }
            Self::UnderlineColor(color) => color.encode_with_base(sink, 58),
            Self::Unknown(code) => write_int(sink, *code),
        }
    }
}

impl<'a> TryFromAnsiIter<'a> for SgrAttr {
    /// Parse an SGR attribute from an iterator of parameter byte slices.
    ///
    /// This handles both simple single-parameter codes and multi-parameter
    /// extended color sequences like `38;5;196` or `38;2;255;128;0`.
    ///
    /// The iterator is advanced past all consumed parameters.
    fn try_from_ansi_iter<I>(iter: &mut I) -> Result<Self, ParseError>
    where
        I: Iterator<Item = &'a [u8]>,
    {
        let bytes = iter.next().ok_or(ParseError::Empty)?;

        let s = std::str::from_utf8(bytes).map_err(|_| {
            ParseError::InvalidValue(
                "invalid UTF-8 in SGR parameter".to_string(),
            )
        })?;

        // Check for colon-separated subparams (e.g., "4:3" for curly underline)
        // These are self-contained in a single parameter
        if s.contains(':') {
            let parts: Vec<&str> = s.split(':').collect();
            let base_num: u16 =
                parts.first().and_then(|p| p.parse().ok()).unwrap_or(0);

            return match base_num {
                4 => {
                    // Extended underline style: 4:Ps
                    let sub_num: u8 =
                        parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(1);
                    Ok(Self::UnderlineStyle(
                        UnderlineStyle::try_from(sub_num)
                            .unwrap_or(UnderlineStyle::Single),
                    ))
                }
                38 => {
                    // Extended foreground color (colon format)
                    ExtendedColor::parse_from_parts(&parts)
                        .map(Self::Foreground)
                        .ok_or_else(|| {
                            ParseError::InvalidValue(format!(
                                "invalid extended foreground color: {s}"
                            ))
                        })
                }
                48 => {
                    // Extended background color (colon format)
                    ExtendedColor::parse_from_parts(&parts)
                        .map(Self::Background)
                        .ok_or_else(|| {
                            ParseError::InvalidValue(format!(
                                "invalid extended background color: {s}"
                            ))
                        })
                }
                58 => {
                    // Extended underline color (colon format)
                    ExtendedColor::parse_from_parts(&parts)
                        .map(Self::UnderlineColor)
                        .ok_or_else(|| {
                            ParseError::InvalidValue(format!(
                                "invalid extended underline color: {s}"
                            ))
                        })
                }
                _ => Ok(Self::Unknown(base_num)),
            };
        }

        // Simple single parameter - parse the code number
        let code: u16 = s.parse().map_err(|_| {
            ParseError::InvalidValue(format!("invalid SGR parameter: {s}"))
        })?;

        // Check for extended color codes that need additional parameters
        // These use semicolon-separated format: 38;5;n or 38;2;r;g;b
        match code {
            38 => {
                // Extended foreground color (semicolon format)
                ExtendedColor::parse_from_iter(iter)
                    .map(Self::Foreground)
                    .ok_or_else(|| {
                        ParseError::InvalidValue(
                            "invalid extended foreground color parameters"
                                .to_string(),
                        )
                    })
            }
            48 => {
                // Extended background color (semicolon format)
                ExtendedColor::parse_from_iter(iter)
                    .map(Self::Background)
                    .ok_or_else(|| {
                        ParseError::InvalidValue(
                            "invalid extended background color parameters"
                                .to_string(),
                        )
                    })
            }
            58 => {
                // Extended underline color (semicolon format)
                ExtendedColor::parse_from_iter(iter)
                    .map(Self::UnderlineColor)
                    .ok_or_else(|| {
                        ParseError::InvalidValue(
                            "invalid extended underline color parameters"
                                .to_string(),
                        )
                    })
            }
            _ => {
                // Try to parse as SimpleSgrCode
                if let Ok(simple) = SimpleSgrCode::try_from(code) {
                    return Ok(Self::Simple(simple));
                }

                // Handle alternative fonts (11-19)
                #[allow(clippy::cast_possible_truncation)]
                if (11..=19).contains(&code) {
                    return Ok(Self::AlternativeFont((code - 10) as u8));
                }

                // Unknown code
                Ok(Self::Unknown(code))
            }
        }
    }
}

impl TryFromAnsi<'_> for SgrAttr {
    fn try_from_ansi(bytes: &[u8]) -> Result<Self, ParseError> {
        // Delegate to the iterator implementation
        <SgrAttr as TryFromAnsiIter>::try_from_ansi_iter(
            &mut bytes.split(|&c| c == b';'),
        )
    }
}

/// Parse multiple SGR attributes from a semicolon-separated parameter string.
///
/// This handles sequences like `1;31;48;5;196` which contain multiple
/// attributes including extended colors.
///
/// # Errors
///
/// Returns a [`ParseError`] if any of the SGR parameters are invalid,
/// such as non-numeric values or malformed extended color sequences.
pub fn parse_sgr_params(bytes: &[u8]) -> Result<Vec<SgrAttr>, ParseError> {
    let mut attrs = Vec::new();
    let mut iter = bytes.split(|&c| c == b';').peekable();

    while iter.peek().is_some() {
        let attr = SgrAttr::try_from_ansi_iter(&mut iter)?;
        attrs.push(attr);
    }

    Ok(attrs)
}

// =============================================================================
// Sgr - Main SGR Sequence
// =============================================================================

/// Select Graphic Rendition sequence (`CSI Pm m`).
///
/// *Sequence*: `CSI Pm m`
///
/// SGR is the standard ANSI escape sequence for controlling text attributes
/// and colors in terminal output. This struct wraps a single [`SgrAttr`]
/// and handles the CSI prefix and `m` final byte when encoding.
///
/// # Sequence Format
///
/// The encoded format is: `ESC [ <params> m`
///
/// Where `<params>` is the encoded attribute. For extended colors, the
/// ISO-8613-6 colon-separated format is used by default.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use vtio::event::sgr::{Sgr, SgrAttr};
/// use vtansi::AnsiEncode;
///
/// // Reset all attributes
/// let reset = Sgr::reset();
/// let mut buf = Vec::new();
/// reset.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[0m");
///
/// // Bold text
/// let bold = Sgr::new(SgrAttr::Bold);
/// let mut buf = Vec::new();
/// bold.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[1m");
///
/// // Foreground color
/// let red = Sgr::from(SgrAttr::ForegroundRed);
/// let mut buf = Vec::new();
/// red.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[31m");
/// ```
///
/// ## Extended Colors
///
/// ```
/// use vtio::event::sgr::{Sgr, SgrAttr};
/// use vtansi::AnsiEncode;
///
/// // 256-color foreground (uses colon format)
/// let orange = Sgr::new(SgrAttr::foreground_256(208));
/// let mut buf = Vec::new();
/// orange.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[38:5:208m");
///
/// // RGB truecolor background
/// let navy = Sgr::new(SgrAttr::background_rgb(0, 0, 128));
/// let mut buf = Vec::new();
/// navy.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[48:2::0:0:128m");
/// ```
///
/// # Multiple Attributes
///
/// To apply multiple attributes in a single escape sequence, you'll need
/// to encode them separately or concatenate their parameter strings.
/// Each `Sgr` instance represents a single attribute.
///
/// # Legacy Format
///
/// For compatibility with older terminals that don't support the colon-separated
/// format, use [`LegacySgr`] which encodes extended colors with semicolons:
///
/// ```
/// use vtio::event::sgr::{Sgr, LegacySgr, SgrAttr};
/// use vtansi::AnsiEncode;
///
/// let sgr = Sgr::new(SgrAttr::foreground_256(196));
///
/// // Standard format (colon-separated)
/// let mut buf = Vec::new();
/// sgr.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[38:5:196m");
///
/// // Legacy format (semicolon-separated)
/// let mut buf = Vec::new();
/// LegacySgr::from(sgr).encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[38;5;196m");
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Default, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, finalbyte = 'm')]
pub struct Sgr {
    /// The SGR attribute to apply.
    pub attr: SgrAttr,
}

impl Sgr {
    /// Create a new SGR sequence with the given attribute.
    #[must_use]
    pub const fn new(attr: SgrAttr) -> Self {
        Self { attr }
    }

    /// Create an SGR reset sequence (`CSI 0 m`).
    #[must_use]
    pub const fn reset() -> Self {
        Self {
            attr: SgrAttr::Reset,
        }
    }
}

impl From<SgrAttr> for Sgr {
    fn from(attr: SgrAttr) -> Self {
        Self::new(attr)
    }
}

// =============================================================================
// LegacySgr - Semicolon Encoding Wrapper
// =============================================================================

/// Select Graphic Rendition with legacy semicolon-separated encoding.
///
/// This wrapper type changes how extended colors are encoded in the SGR
/// sequence, using the legacy xterm-style semicolon format instead of the
/// ISO-8613-6 colon format.
///
/// *Sequence*: `CSI Pm m`
///
/// # Format Comparison
///
/// | Color Type | Standard (`Sgr`) | Legacy (`LegacySgr`) |
/// |------------|------------------|----------------------|
/// | 256-color | `\x1b[38:5:196m` | `\x1b[38;5;196m` |
/// | RGB | `\x1b[38:2::255:0:0m` | `\x1b[38;2;255;0;0m` |
/// | Default | `\x1b[38:1m` | `\x1b[38:1m` (unchanged) |
///
/// # When to Use
///
/// Use `LegacySgr` when:
/// - Targeting older terminal emulators (pre-2010)
/// - Maximum compatibility is required
/// - The target terminal is unknown
///
/// Use `Sgr` (standard) when:
/// - Targeting modern terminals (xterm 282+, VTE 0.36+, etc.)
/// - Standards compliance is important
/// - You need CMY/CMYK colors (only available in colon format)
///
/// # Limitations
///
/// The following color types only exist in ISO-8613-6 and always use
/// colon separators, even with `LegacySgr`:
/// - Default/transparent color (type 1)
/// - CMY color (type 3)
/// - CMYK color (type 4)
///
/// Underline styles (`4:x`) also always use colons as there is no
/// semicolon equivalent.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use vtio::event::sgr::{Sgr, LegacySgr, SgrAttr};
/// use vtansi::AnsiEncode;
///
/// // Create an SGR with RGB color
/// let sgr = Sgr::new(SgrAttr::foreground_rgb(255, 128, 64));
///
/// // Standard ISO-8613-6 format (colons)
/// let mut buf = Vec::new();
/// sgr.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[38:2::255:128:64m");
///
/// // Legacy xterm format (semicolons)
/// let mut buf = Vec::new();
/// LegacySgr::from(sgr).encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[38;2;255;128;64m");
/// ```
///
/// ## Direct Construction
///
/// ```
/// use vtio::event::sgr::{LegacySgr, SgrAttr};
/// use vtansi::AnsiEncode;
///
/// // Create directly with an attribute
/// let legacy = LegacySgr::new(SgrAttr::background_256(21));
/// let mut buf = Vec::new();
/// legacy.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[48;5;21m");
///
/// // Reset sequence (same format either way)
/// let reset = LegacySgr::reset();
/// let mut buf = Vec::new();
/// reset.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[0m");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct LegacySgr(pub Sgr);

impl LegacySgr {
    /// Create a new legacy SGR sequence with the given attribute.
    #[must_use]
    pub const fn new(attr: SgrAttr) -> Self {
        Self(Sgr::new(attr))
    }

    /// Create a legacy SGR reset sequence (`CSI 0 m`).
    #[must_use]
    pub const fn reset() -> Self {
        Self(Sgr::reset())
    }
}

impl From<Sgr> for LegacySgr {
    #[inline]
    fn from(sgr: Sgr) -> Self {
        Self(sgr)
    }
}

impl From<LegacySgr> for Sgr {
    #[inline]
    fn from(wrapper: LegacySgr) -> Self {
        wrapper.0
    }
}

impl From<SgrAttr> for LegacySgr {
    #[inline]
    fn from(attr: SgrAttr) -> Self {
        Self::new(attr)
    }
}

impl vtansi::AnsiEncode for LegacySgr {
    #[inline]
    fn encode_ansi_into<W: Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<usize, vtansi::EncodeError> {
        write_csi!(sink; LegacySgrAttr(self.0.attr), 'm')
    }
}

// =============================================================================
// SGR Video Attribute Stack (XTPUSHSGR/XTPOPSGR)
// =============================================================================

bitflags! {
    /// Attributes that can be selectively pushed to the video attribute stack.
    ///
    /// When pushing video attributes with [`PushVideoAttributes`], you can specify
    /// which attributes to save. If no attributes are specified (empty flags),
    /// all current SGR attributes are saved.
    ///
    /// These flags correspond to the parameter values for `CSI Pm # {`:
    ///
    /// | Value | Attribute |
    /// |-------|-----------|
    /// | 1 | Bold/faint |
    /// | 2 | Underline |
    /// | 4 | Blink |
    /// | 8 | Inverse |
    /// | 16 | Invisible |
    /// | 32 | Foreground color |
    /// | 64 | Background color |
    /// | 128 | Underline color |
    /// | 256 | Strikethrough |
    /// | 512 | Overline |
    /// | 1024 | Font |
    ///
    /// # Example
    ///
    /// ```
    /// use vtio::event::sgr::{PushVideoAttributes, SgrStackAttribute};
    ///
    /// // Push only foreground and background colors
    /// let push = PushVideoAttributes::selective(
    ///     SgrStackAttribute::FOREGROUND | SgrStackAttribute::BACKGROUND
    /// );
    /// ```
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct SgrStackAttribute: u16 {
        /// Bold and faint intensity attributes.
        const BOLD_FAINT = 1;
        /// Underline attribute.
        const UNDERLINE = 2;
        /// Blink attribute.
        const BLINK = 4;
        /// Inverse/reverse video attribute.
        const INVERSE = 8;
        /// Invisible/hidden attribute.
        const INVISIBLE = 16;
        /// Foreground color.
        const FOREGROUND = 32;
        /// Background color.
        const BACKGROUND = 64;
        /// Underline color.
        const UNDERLINE_COLOR = 128;
        /// Strikethrough attribute.
        const STRIKETHROUGH = 256;
        /// Overline attribute.
        const OVERLINE = 512;
        /// Font selection.
        const FONT = 1024;
    }
}

/// Push video attributes onto the stack (`XTPUSHSGR`).
///
/// *Sequence*: `CSI Pm # {`
///
/// Push the current SGR attributes onto an internal stack. If specific
/// attributes are provided via the parameter, only those attributes are saved.
/// If no parameters are given, all attributes are saved.
///
/// The stack has a maximum depth of 10 levels in xterm. Pushing beyond this
/// limit may cause older entries to be lost.
///
/// Use [`PopVideoAttributes`] to restore the saved attributes.
///
/// # Parameters
///
/// The `attributes` field specifies which attributes to push. See
/// [`SgrStackAttribute`] for the available flags. If `None` (default),
/// all attributes are pushed.
///
/// # Example
///
/// ```
/// use vtio::event::sgr::{PushVideoAttributes, SgrStackAttribute};
/// use vtansi::AnsiEncode;
///
/// // Push all attributes
/// let push_all = PushVideoAttributes::all();
/// let mut buf = Vec::new();
/// push_all.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[#{");
///
/// // Push only colors
/// let push_colors = PushVideoAttributes::selective(
///     SgrStackAttribute::FOREGROUND | SgrStackAttribute::BACKGROUND
/// );
/// let mut buf = Vec::new();
/// push_colors.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[96#{");
/// ```
///
/// # See Also
///
/// - [`PopVideoAttributes`] - Restore saved attributes
/// - <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-which-begin-with-CSI>
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, intermediate = "#", finalbyte = '{')]
pub struct PushVideoAttributes {
    /// Which attributes to push. `None` means push all attributes.
    pub attributes: Option<SgrStackAttribute>,
}

impl PushVideoAttributes {
    /// Create a push command for all attributes.
    #[must_use]
    pub const fn all() -> Self {
        Self { attributes: None }
    }

    /// Create a push command for specific attributes.
    #[must_use]
    pub const fn selective(attributes: SgrStackAttribute) -> Self {
        Self {
            attributes: Some(attributes),
        }
    }
}

/// Pop video attributes from the stack (`XTPOPSGR`).
///
/// *Sequence*: `CSI # }` or `CSI # q`
///
/// Pop and restore SGR attributes from the internal stack. The attributes
/// that were saved by the most recent [`PushVideoAttributes`] are restored.
///
/// If the stack is empty, this command has no effect.
///
/// # Example
///
/// ```
/// use vtio::event::sgr::PopVideoAttributes;
/// use vtansi::AnsiEncode;
///
/// let pop = PopVideoAttributes;
/// let mut buf = Vec::new();
/// pop.encode_ansi_into(&mut buf).unwrap();
/// assert_eq!(&buf, b"\x1b[#}");
/// ```
///
/// # See Also
///
/// - [`PushVideoAttributes`] - Save attributes to stack
/// - <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-which-begin-with-CSI>
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, vtansi::derive::AnsiOutput,
)]
#[vtansi(csi, intermediate = "#", finalbyte = '}')]
pub struct PopVideoAttributes;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::AnsiEncode;
    use vtansi::TryFromAnsi;

    #[test]
    fn test_sgr_reset() {
        let sgr = Sgr::reset();
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[0m");
    }

    #[test]
    fn test_sgr_bold() {
        let sgr = Sgr::new(SgrAttr::Bold);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[1m");
    }

    #[test]
    fn test_sgr_foreground_red() {
        let sgr = Sgr::new(SgrAttr::ForegroundRed);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[31m");
    }

    #[test]
    fn test_sgr_foreground_256() {
        let sgr = Sgr::new(SgrAttr::foreground_256(196));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38:5:196m");
    }

    #[test]
    fn test_sgr_background_256() {
        let sgr = Sgr::new(SgrAttr::background_256(21));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[48:5:21m");
    }

    #[test]
    fn test_sgr_foreground_rgb() {
        let sgr = Sgr::new(SgrAttr::foreground_rgb(255, 128, 0));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38:2::255:128:0m");
    }

    #[test]
    fn test_sgr_background_rgb() {
        let sgr = Sgr::new(SgrAttr::background_rgb(0, 0, 128));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[48:2::0:0:128m");
    }

    #[test]
    fn test_sgr_underline_style_curly() {
        let sgr = Sgr::new(SgrAttr::UnderlineStyle(UnderlineStyle::Curly));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[4:3m");
    }

    #[test]
    fn test_sgr_bright_colors() {
        let sgr = Sgr::new(SgrAttr::ForegroundBrightCyan);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[96m");
    }

    #[test]
    fn test_ansi_color_foreground() {
        let attr = SgrAttr::foreground(AnsiColor::Red);
        assert_eq!(attr, SgrAttr::ForegroundRed);
    }

    #[test]
    fn test_ansi_color_background_bright() {
        let attr = SgrAttr::background_bright(AnsiColor::Blue);
        assert_eq!(attr, SgrAttr::BackgroundBrightBlue);
    }

    #[test]
    fn test_sgr_from_attribute() {
        let sgr: Sgr = SgrAttr::Italic.into();
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[3m");
    }

    #[test]
    fn test_sgr_alternative_font() {
        let sgr = Sgr::new(SgrAttr::AlternativeFont(3));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[13m");
    }

    #[test]
    fn test_sgr_fraktur() {
        let sgr = Sgr::new(SgrAttr::Fraktur);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[20m");
    }

    #[test]
    fn test_sgr_framed() {
        let sgr = Sgr::new(SgrAttr::Framed);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[51m");
    }

    #[test]
    fn test_sgr_encircled() {
        let sgr = Sgr::new(SgrAttr::Encircled);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[52m");
    }

    #[test]
    fn test_sgr_overline() {
        let sgr = Sgr::new(SgrAttr::Overline);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[53m");
    }

    #[test]
    fn test_sgr_superscript() {
        let sgr = Sgr::new(SgrAttr::Superscript);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[73m");
    }

    #[test]
    fn test_sgr_subscript() {
        let sgr = Sgr::new(SgrAttr::Subscript);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[74m");
    }

    #[test]
    fn test_sgr_ideogram_underline() {
        let sgr = Sgr::new(SgrAttr::IdeogramUnderline);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[60m");
    }

    #[test]
    fn test_sgr_parse_alternative_font() {
        let attr = SgrAttr::try_from_ansi(b"15").unwrap();
        assert_eq!(attr, SgrAttr::AlternativeFont(5));
    }

    #[test]
    fn test_sgr_parse_superscript() {
        let attr = SgrAttr::try_from_ansi(b"73").unwrap();
        assert_eq!(attr, SgrAttr::Superscript);
    }

    #[test]
    fn test_sgr_parse_framed() {
        let attr = SgrAttr::try_from_ansi(b"51").unwrap();
        assert_eq!(attr, SgrAttr::Framed);
    }

    #[test]
    fn test_simple_sgr_code_derives() {
        // Test that SimpleSgrCode encodes correctly via the derive
        let code = SimpleSgrCode::Bold;
        let mut buf = Vec::new();
        code.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"1");
    }

    #[test]
    fn test_extended_color_palette() {
        let color = ExtendedColor::palette(196);
        let mut buf = Vec::new();
        color.encode_with_base(&mut buf, 38).unwrap();
        assert_eq!(buf, b"38:5:196");
    }

    #[test]
    fn test_extended_color_rgb() {
        let color = ExtendedColor::rgb(255, 128, 0);
        let mut buf = Vec::new();
        color.encode_with_base(&mut buf, 48).unwrap();
        assert_eq!(buf, b"48:2::255:128:0");
    }

    #[test]
    fn test_from_simple_sgr_code() {
        let attr: SgrAttr = SimpleSgrCode::Italic.into();
        assert_eq!(attr, SgrAttr::Italic);
    }

    // =========================================================================
    // Parsing tests for colon-separated extended colors
    // =========================================================================

    #[test]
    fn test_parse_colon_foreground_256() {
        // 38:5:n format (ITU T.416 / kitty style)
        let attr = SgrAttr::try_from_ansi(b"38:5:196").unwrap();
        assert_eq!(attr, SgrAttr::Foreground(ExtendedColor::Palette(196)));
    }

    #[test]
    fn test_parse_colon_background_256() {
        // 48:5:n format
        let attr = SgrAttr::try_from_ansi(b"48:5:21").unwrap();
        assert_eq!(attr, SgrAttr::Background(ExtendedColor::Palette(21)));
    }

    #[test]
    fn test_parse_colon_foreground_rgb() {
        // 38:2:r:g:b format (no colorspace)
        let attr = SgrAttr::try_from_ansi(b"38:2:255:128:0").unwrap();
        assert_eq!(
            attr,
            SgrAttr::Foreground(ExtendedColor::Rgb {
                r: 255,
                g: 128,
                b: 0
            })
        );
    }

    #[test]
    fn test_parse_colon_background_rgb() {
        // 48:2:r:g:b format (no colorspace)
        let attr = SgrAttr::try_from_ansi(b"48:2:0:0:128").unwrap();
        assert_eq!(
            attr,
            SgrAttr::Background(ExtendedColor::Rgb { r: 0, g: 0, b: 128 })
        );
    }

    #[test]
    fn test_parse_colon_rgb_with_colorspace() {
        // 38:2:colorspace:r:g:b format (colorspace is ignored but parsed)
        let attr = SgrAttr::try_from_ansi(b"38:2::12:34:56").unwrap();
        assert_eq!(
            attr,
            SgrAttr::Foreground(ExtendedColor::Rgb {
                r: 12,
                g: 34,
                b: 56
            })
        );
    }

    #[test]
    fn test_parse_colon_rgb_with_explicit_colorspace() {
        // 38:2:1:r:g:b format (colorspace=1)
        let attr = SgrAttr::try_from_ansi(b"38:2:1:100:150:200").unwrap();
        assert_eq!(
            attr,
            SgrAttr::Foreground(ExtendedColor::Rgb {
                r: 100,
                g: 150,
                b: 200
            })
        );
    }

    // =========================================================================
    // Parsing tests for underline color (SGR 58)
    // =========================================================================

    #[test]
    fn test_parse_colon_underline_color_256() {
        // 58:5:n format
        let attr = SgrAttr::try_from_ansi(b"58:5:208").unwrap();
        assert_eq!(attr, SgrAttr::UnderlineColor(ExtendedColor::Palette(208)));
    }

    #[test]
    fn test_parse_colon_underline_color_rgb() {
        // 58:2:r:g:b format
        let attr = SgrAttr::try_from_ansi(b"58:2:255:0:0").unwrap();
        assert_eq!(
            attr,
            SgrAttr::UnderlineColor(ExtendedColor::Rgb { r: 255, g: 0, b: 0 })
        );
    }

    #[test]
    fn test_parse_colon_underline_color_rgb_with_colorspace() {
        // 58:2::r:g:b format (empty colorspace)
        let attr = SgrAttr::try_from_ansi(b"58:2::200:100:50").unwrap();
        assert_eq!(
            attr,
            SgrAttr::UnderlineColor(ExtendedColor::Rgb {
                r: 200,
                g: 100,
                b: 50
            })
        );
    }

    // =========================================================================
    // Parsing tests for underline styles (SGR 4:x)
    // =========================================================================

    #[test]
    fn test_parse_underline_style_none() {
        let attr = SgrAttr::try_from_ansi(b"4:0").unwrap();
        assert_eq!(attr, SgrAttr::UnderlineStyle(UnderlineStyle::None));
    }

    #[test]
    fn test_parse_underline_style_single() {
        let attr = SgrAttr::try_from_ansi(b"4:1").unwrap();
        assert_eq!(attr, SgrAttr::UnderlineStyle(UnderlineStyle::Single));
    }

    #[test]
    fn test_parse_underline_style_double() {
        let attr = SgrAttr::try_from_ansi(b"4:2").unwrap();
        assert_eq!(attr, SgrAttr::UnderlineStyle(UnderlineStyle::Double));
    }

    #[test]
    fn test_parse_underline_style_curly() {
        let attr = SgrAttr::try_from_ansi(b"4:3").unwrap();
        assert_eq!(attr, SgrAttr::UnderlineStyle(UnderlineStyle::Curly));
    }

    #[test]
    fn test_parse_underline_style_dotted() {
        let attr = SgrAttr::try_from_ansi(b"4:4").unwrap();
        assert_eq!(attr, SgrAttr::UnderlineStyle(UnderlineStyle::Dotted));
    }

    #[test]
    fn test_parse_underline_style_dashed() {
        let attr = SgrAttr::try_from_ansi(b"4:5").unwrap();
        assert_eq!(attr, SgrAttr::UnderlineStyle(UnderlineStyle::Dashed));
    }

    #[test]
    fn test_parse_underline_style_invalid_defaults_to_single() {
        // Unknown underline style value should default to single
        let attr = SgrAttr::try_from_ansi(b"4:99").unwrap();
        assert_eq!(attr, SgrAttr::UnderlineStyle(UnderlineStyle::Single));
    }

    // =========================================================================
    // Encoding tests for underline styles
    // =========================================================================

    #[test]
    fn test_encode_underline_style_none() {
        let sgr = Sgr::new(SgrAttr::UnderlineStyle(UnderlineStyle::None));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[4:0m");
    }

    #[test]
    fn test_encode_underline_style_double() {
        let sgr = Sgr::new(SgrAttr::UnderlineStyle(UnderlineStyle::Double));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[4:2m");
    }

    #[test]
    fn test_encode_underline_style_dotted() {
        let sgr = Sgr::new(SgrAttr::UnderlineStyle(UnderlineStyle::Dotted));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[4:4m");
    }

    #[test]
    fn test_encode_underline_style_dashed() {
        let sgr = Sgr::new(SgrAttr::UnderlineStyle(UnderlineStyle::Dashed));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[4:5m");
    }

    // =========================================================================
    // Encoding tests for underline colors
    // =========================================================================

    #[test]
    fn test_encode_underline_color_256() {
        let sgr =
            Sgr::new(SgrAttr::UnderlineColor(ExtendedColor::Palette(208)));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[58:5:208m");
    }

    #[test]
    fn test_encode_underline_color_rgb() {
        let sgr = Sgr::new(SgrAttr::UnderlineColor(ExtendedColor::Rgb {
            r: 255,
            g: 128,
            b: 64,
        }));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[58:2::255:128:64m");
    }

    #[test]
    fn test_encode_underline_color_default() {
        let sgr = Sgr::new(SgrAttr::UnderlineColorDefault);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[59m");
    }

    // =========================================================================
    // Unknown code handling
    // =========================================================================

    #[test]
    fn test_parse_unknown_code() {
        // Code 56 is not standardized
        let attr = SgrAttr::try_from_ansi(b"56").unwrap();
        assert_eq!(attr, SgrAttr::Unknown(56));
    }

    #[test]
    fn test_parse_unknown_colon_code() {
        // Unknown base code with colon separator
        let attr = SgrAttr::try_from_ansi(b"99:1:2:3").unwrap();
        assert_eq!(attr, SgrAttr::Unknown(99));
    }

    #[test]
    fn test_encode_unknown_code() {
        let sgr = Sgr::new(SgrAttr::Unknown(56));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[56m");
    }

    // =========================================================================
    // Edge cases and boundary values
    // =========================================================================

    #[test]
    fn test_parse_simple_codes_boundary() {
        // Test first and last simple codes
        assert_eq!(SgrAttr::try_from_ansi(b"0").unwrap(), SgrAttr::Reset);
        assert_eq!(
            SgrAttr::try_from_ansi(b"107").unwrap(),
            SgrAttr::BackgroundBrightWhite
        );
    }

    #[test]
    fn test_parse_alternative_font_range() {
        // Test all alternative fonts (11-19)
        for i in 1..=9u8 {
            let code = format!("{}", 10 + i);
            let attr = SgrAttr::try_from_ansi(code.as_bytes()).unwrap();
            assert_eq!(attr, SgrAttr::AlternativeFont(i));
        }
    }

    #[test]
    fn test_extended_color_palette_boundaries() {
        // Test palette color boundaries (0 and 255)
        let attr_0 = SgrAttr::try_from_ansi(b"38:5:0").unwrap();
        assert_eq!(attr_0, SgrAttr::Foreground(ExtendedColor::Palette(0)));

        let attr_255 = SgrAttr::try_from_ansi(b"38:5:255").unwrap();
        assert_eq!(attr_255, SgrAttr::Foreground(ExtendedColor::Palette(255)));
    }

    #[test]
    fn test_extended_color_rgb_boundaries() {
        // Test RGB color boundaries
        let attr_black = SgrAttr::try_from_ansi(b"38:2:0:0:0").unwrap();
        assert_eq!(
            attr_black,
            SgrAttr::Foreground(ExtendedColor::Rgb { r: 0, g: 0, b: 0 })
        );

        let attr_white = SgrAttr::try_from_ansi(b"38:2:255:255:255").unwrap();
        assert_eq!(
            attr_white,
            SgrAttr::Foreground(ExtendedColor::Rgb {
                r: 255,
                g: 255,
                b: 255
            })
        );
    }

    #[test]
    fn test_roundtrip_extended_colors() {
        // Verify that encoding then parsing produces equivalent output
        // Default encoding uses colons (ISO-8613-6 standard)

        let original = SgrAttr::foreground_256(123);
        let sgr = Sgr::new(original);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38:5:123m");
    }

    #[test]
    fn test_roundtrip_rgb() {
        let original = SgrAttr::foreground_rgb(10, 20, 30);
        let sgr = Sgr::new(original);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38:2::10:20:30m");
    }

    #[test]
    fn test_roundtrip_default_color() {
        let original = SgrAttr::Foreground(ExtendedColor::Default);
        let sgr = Sgr::new(original);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38:1m");
    }

    #[test]
    fn test_roundtrip_cmy() {
        let original = SgrAttr::Foreground(ExtendedColor::Cmy {
            c: 100,
            m: 150,
            y: 200,
        });
        let sgr = Sgr::new(original);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38:3:100:150:200m");
    }

    #[test]
    fn test_roundtrip_cmyk() {
        let original = SgrAttr::Foreground(ExtendedColor::Cmyk {
            c: 10,
            m: 20,
            y: 30,
            k: 40,
        });
        let sgr = Sgr::new(original);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38:4:10:20:30:40m");
    }

    // =========================================================================
    // Error handling tests
    // =========================================================================

    #[test]
    fn test_parse_invalid_non_numeric() {
        let result = SgrAttr::try_from_ansi(b"abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_extended_color_missing_parts() {
        // 38:5 without the color index
        let result = SgrAttr::try_from_ansi(b"38:5");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_rgb_missing_components() {
        // 38:2:r:g without b
        let result = SgrAttr::try_from_ansi(b"38:2:10:20");
        assert!(result.is_err());
    }

    // =========================================================================
    // SimpleSgrCode comprehensive tests
    // =========================================================================

    #[test]
    fn test_simple_sgr_code_all_intensity() {
        assert_eq!(SgrAttr::try_from_ansi(b"1").unwrap(), SgrAttr::Bold);
        assert_eq!(SgrAttr::try_from_ansi(b"2").unwrap(), SgrAttr::Faint);
        assert_eq!(
            SgrAttr::try_from_ansi(b"22").unwrap(),
            SgrAttr::NormalIntensity
        );
    }

    #[test]
    fn test_simple_sgr_code_all_blink() {
        assert_eq!(SgrAttr::try_from_ansi(b"5").unwrap(), SgrAttr::Blink);
        assert_eq!(SgrAttr::try_from_ansi(b"6").unwrap(), SgrAttr::RapidBlink);
        assert_eq!(
            SgrAttr::try_from_ansi(b"25").unwrap(),
            SgrAttr::NotBlinking
        );
    }

    #[test]
    fn test_simple_sgr_code_ideogram_attributes() {
        assert_eq!(
            SgrAttr::try_from_ansi(b"60").unwrap(),
            SgrAttr::IdeogramUnderline
        );
        assert_eq!(
            SgrAttr::try_from_ansi(b"61").unwrap(),
            SgrAttr::IdeogramDoubleUnderline
        );
        assert_eq!(
            SgrAttr::try_from_ansi(b"62").unwrap(),
            SgrAttr::IdeogramOverline
        );
        assert_eq!(
            SgrAttr::try_from_ansi(b"63").unwrap(),
            SgrAttr::IdeogramDoubleOverline
        );
        assert_eq!(
            SgrAttr::try_from_ansi(b"64").unwrap(),
            SgrAttr::IdeogramStressMarking
        );
        assert_eq!(
            SgrAttr::try_from_ansi(b"65").unwrap(),
            SgrAttr::IdeogramAttributesOff
        );
    }

    #[test]
    fn test_simple_sgr_code_proportional_spacing() {
        assert_eq!(
            SgrAttr::try_from_ansi(b"26").unwrap(),
            SgrAttr::ProportionalSpacing
        );
        assert_eq!(
            SgrAttr::try_from_ansi(b"50").unwrap(),
            SgrAttr::NotProportionalSpacing
        );
    }

    #[test]
    fn test_simple_sgr_code_frame_and_encircle() {
        assert_eq!(SgrAttr::try_from_ansi(b"51").unwrap(), SgrAttr::Framed);
        assert_eq!(SgrAttr::try_from_ansi(b"52").unwrap(), SgrAttr::Encircled);
        assert_eq!(
            SgrAttr::try_from_ansi(b"54").unwrap(),
            SgrAttr::NotFramedNotEncircled
        );
    }

    #[test]
    fn test_simple_sgr_code_overline() {
        assert_eq!(SgrAttr::try_from_ansi(b"53").unwrap(), SgrAttr::Overline);
        assert_eq!(
            SgrAttr::try_from_ansi(b"55").unwrap(),
            SgrAttr::NotOverline
        );
    }

    #[test]
    fn test_simple_sgr_code_superscript_subscript() {
        assert_eq!(
            SgrAttr::try_from_ansi(b"73").unwrap(),
            SgrAttr::Superscript
        );
        assert_eq!(SgrAttr::try_from_ansi(b"74").unwrap(), SgrAttr::Subscript);
        assert_eq!(
            SgrAttr::try_from_ansi(b"75").unwrap(),
            SgrAttr::NotSuperscriptSubscript
        );
    }

    #[test]
    fn test_simple_sgr_code_default_colors() {
        assert_eq!(
            SgrAttr::try_from_ansi(b"39").unwrap(),
            SgrAttr::ForegroundDefault
        );
        assert_eq!(
            SgrAttr::try_from_ansi(b"49").unwrap(),
            SgrAttr::BackgroundDefault
        );
        assert_eq!(
            SgrAttr::try_from_ansi(b"59").unwrap(),
            SgrAttr::UnderlineColorDefault
        );
    }

    // =========================================================================
    // UnderlineStyle enum tests
    // =========================================================================

    #[test]
    fn test_underline_style_default() {
        assert_eq!(UnderlineStyle::default(), UnderlineStyle::None);
    }

    #[test]
    fn test_underline_style_from_u8() {
        assert_eq!(
            UnderlineStyle::try_from(0u8).unwrap(),
            UnderlineStyle::None
        );
        assert_eq!(
            UnderlineStyle::try_from(1u8).unwrap(),
            UnderlineStyle::Single
        );
        assert_eq!(
            UnderlineStyle::try_from(2u8).unwrap(),
            UnderlineStyle::Double
        );
        assert_eq!(
            UnderlineStyle::try_from(3u8).unwrap(),
            UnderlineStyle::Curly
        );
        assert_eq!(
            UnderlineStyle::try_from(4u8).unwrap(),
            UnderlineStyle::Dotted
        );
        assert_eq!(
            UnderlineStyle::try_from(5u8).unwrap(),
            UnderlineStyle::Dashed
        );
    }

    #[test]
    fn test_underline_style_to_u8() {
        assert_eq!(u8::from(UnderlineStyle::None), 0);
        assert_eq!(u8::from(UnderlineStyle::Single), 1);
        assert_eq!(u8::from(UnderlineStyle::Double), 2);
        assert_eq!(u8::from(UnderlineStyle::Curly), 3);
        assert_eq!(u8::from(UnderlineStyle::Dotted), 4);
        assert_eq!(u8::from(UnderlineStyle::Dashed), 5);
    }

    // =========================================================================
    // AnsiColor enum tests
    // =========================================================================

    #[test]
    fn test_ansi_color_default() {
        assert_eq!(AnsiColor::default(), AnsiColor::Black);
    }

    #[test]
    fn test_ansi_color_all_foreground() {
        assert_eq!(
            SgrAttr::foreground(AnsiColor::Black),
            SgrAttr::ForegroundBlack
        );
        assert_eq!(SgrAttr::foreground(AnsiColor::Red), SgrAttr::ForegroundRed);
        assert_eq!(
            SgrAttr::foreground(AnsiColor::Green),
            SgrAttr::ForegroundGreen
        );
        assert_eq!(
            SgrAttr::foreground(AnsiColor::Yellow),
            SgrAttr::ForegroundYellow
        );
        assert_eq!(
            SgrAttr::foreground(AnsiColor::Blue),
            SgrAttr::ForegroundBlue
        );
        assert_eq!(
            SgrAttr::foreground(AnsiColor::Magenta),
            SgrAttr::ForegroundMagenta
        );
        assert_eq!(
            SgrAttr::foreground(AnsiColor::Cyan),
            SgrAttr::ForegroundCyan
        );
        assert_eq!(
            SgrAttr::foreground(AnsiColor::White),
            SgrAttr::ForegroundWhite
        );
    }

    #[test]
    fn test_ansi_color_all_background() {
        assert_eq!(
            SgrAttr::background(AnsiColor::Black),
            SgrAttr::BackgroundBlack
        );
        assert_eq!(SgrAttr::background(AnsiColor::Red), SgrAttr::BackgroundRed);
        assert_eq!(
            SgrAttr::background(AnsiColor::Green),
            SgrAttr::BackgroundGreen
        );
        assert_eq!(
            SgrAttr::background(AnsiColor::Yellow),
            SgrAttr::BackgroundYellow
        );
        assert_eq!(
            SgrAttr::background(AnsiColor::Blue),
            SgrAttr::BackgroundBlue
        );
        assert_eq!(
            SgrAttr::background(AnsiColor::Magenta),
            SgrAttr::BackgroundMagenta
        );
        assert_eq!(
            SgrAttr::background(AnsiColor::Cyan),
            SgrAttr::BackgroundCyan
        );
        assert_eq!(
            SgrAttr::background(AnsiColor::White),
            SgrAttr::BackgroundWhite
        );
    }

    #[test]
    fn test_ansi_color_all_bright_foreground() {
        assert_eq!(
            SgrAttr::foreground_bright(AnsiColor::Black),
            SgrAttr::ForegroundBrightBlack
        );
        assert_eq!(
            SgrAttr::foreground_bright(AnsiColor::White),
            SgrAttr::ForegroundBrightWhite
        );
    }

    #[test]
    fn test_ansi_color_all_bright_background() {
        assert_eq!(
            SgrAttr::background_bright(AnsiColor::Black),
            SgrAttr::BackgroundBrightBlack
        );
        assert_eq!(
            SgrAttr::background_bright(AnsiColor::White),
            SgrAttr::BackgroundBrightWhite
        );
    }

    // =========================================================================
    // ExtendedColor tests
    // =========================================================================

    #[test]
    fn test_extended_color_constructors() {
        assert_eq!(ExtendedColor::palette(42), ExtendedColor::Palette(42));
        assert_eq!(
            ExtendedColor::rgb(1, 2, 3),
            ExtendedColor::Rgb { r: 1, g: 2, b: 3 }
        );
    }

    #[test]
    fn test_extended_color_equality() {
        assert_eq!(ExtendedColor::Palette(42), ExtendedColor::Palette(42));
        assert_ne!(ExtendedColor::Palette(42), ExtendedColor::Palette(43));
        assert_eq!(
            ExtendedColor::Rgb { r: 1, g: 2, b: 3 },
            ExtendedColor::Rgb { r: 1, g: 2, b: 3 }
        );
        assert_ne!(
            ExtendedColor::Rgb { r: 1, g: 2, b: 3 },
            ExtendedColor::Rgb { r: 1, g: 2, b: 4 }
        );
        assert_eq!(ExtendedColor::Default, ExtendedColor::Default);
        assert_eq!(
            ExtendedColor::Cmy {
                c: 10,
                m: 20,
                y: 30
            },
            ExtendedColor::Cmy {
                c: 10,
                m: 20,
                y: 30
            }
        );
        assert_eq!(
            ExtendedColor::Cmyk {
                c: 10,
                m: 20,
                y: 30,
                k: 40
            },
            ExtendedColor::Cmyk {
                c: 10,
                m: 20,
                y: 30,
                k: 40
            }
        );
    }

    // =========================================================================
    // Extended color type tests (default, CMY, CMYK)
    // =========================================================================

    #[test]
    fn test_parse_colon_default_foreground() {
        let attr = SgrAttr::try_from_ansi(b"38:1").unwrap();
        assert_eq!(attr, SgrAttr::Foreground(ExtendedColor::Default));
    }

    #[test]
    fn test_parse_colon_default_background() {
        let attr = SgrAttr::try_from_ansi(b"48:1").unwrap();
        assert_eq!(attr, SgrAttr::Background(ExtendedColor::Default));
    }

    #[test]
    fn test_parse_colon_default_underline() {
        let attr = SgrAttr::try_from_ansi(b"58:1").unwrap();
        assert_eq!(attr, SgrAttr::UnderlineColor(ExtendedColor::Default));
    }

    #[test]
    fn test_parse_semicolon_default_foreground() {
        let attr = SgrAttr::try_from_ansi(b"38;1").unwrap();
        assert_eq!(attr, SgrAttr::Foreground(ExtendedColor::Default));
    }

    #[test]
    fn test_parse_colon_cmy_foreground() {
        let attr = SgrAttr::try_from_ansi(b"38:3:100:150:200").unwrap();
        assert_eq!(
            attr,
            SgrAttr::Foreground(ExtendedColor::Cmy {
                c: 100,
                m: 150,
                y: 200
            })
        );
    }

    #[test]
    fn test_parse_colon_cmy_background() {
        let attr = SgrAttr::try_from_ansi(b"48:3:50:100:150").unwrap();
        assert_eq!(
            attr,
            SgrAttr::Background(ExtendedColor::Cmy {
                c: 50,
                m: 100,
                y: 150
            })
        );
    }

    #[test]
    fn test_parse_semicolon_cmy_foreground() {
        let attr = SgrAttr::try_from_ansi(b"38;3;100;150;200").unwrap();
        assert_eq!(
            attr,
            SgrAttr::Foreground(ExtendedColor::Cmy {
                c: 100,
                m: 150,
                y: 200
            })
        );
    }

    #[test]
    fn test_parse_colon_cmyk_foreground() {
        let attr = SgrAttr::try_from_ansi(b"38:4:100:150:200:50").unwrap();
        assert_eq!(
            attr,
            SgrAttr::Foreground(ExtendedColor::Cmyk {
                c: 100,
                m: 150,
                y: 200,
                k: 50
            })
        );
    }

    #[test]
    fn test_parse_colon_cmyk_background() {
        let attr = SgrAttr::try_from_ansi(b"48:4:10:20:30:40").unwrap();
        assert_eq!(
            attr,
            SgrAttr::Background(ExtendedColor::Cmyk {
                c: 10,
                m: 20,
                y: 30,
                k: 40
            })
        );
    }

    #[test]
    fn test_parse_semicolon_cmyk_foreground() {
        let attr = SgrAttr::try_from_ansi(b"38;4;100;150;200;50").unwrap();
        assert_eq!(
            attr,
            SgrAttr::Foreground(ExtendedColor::Cmyk {
                c: 100,
                m: 150,
                y: 200,
                k: 50
            })
        );
    }

    #[test]
    fn test_encode_default_foreground() {
        let mut buf = Vec::new();
        SgrAttr::Foreground(ExtendedColor::Default)
            .encode_ansi_into(&mut buf)
            .unwrap();
        assert_eq!(buf, b"38:1");
    }

    #[test]
    fn test_encode_default_background() {
        let mut buf = Vec::new();
        SgrAttr::Background(ExtendedColor::Default)
            .encode_ansi_into(&mut buf)
            .unwrap();
        assert_eq!(buf, b"48:1");
    }

    #[test]
    fn test_encode_cmy_foreground() {
        let mut buf = Vec::new();
        SgrAttr::Foreground(ExtendedColor::Cmy {
            c: 100,
            m: 150,
            y: 200,
        })
        .encode_ansi_into(&mut buf)
        .unwrap();
        assert_eq!(buf, b"38:3:100:150:200");
    }

    #[test]
    fn test_encode_cmyk_foreground() {
        let mut buf = Vec::new();
        SgrAttr::Foreground(ExtendedColor::Cmyk {
            c: 10,
            m: 20,
            y: 30,
            k: 40,
        })
        .encode_ansi_into(&mut buf)
        .unwrap();
        assert_eq!(buf, b"38:4:10:20:30:40");
    }

    #[test]
    fn test_extended_color_new_constructors() {
        assert_eq!(ExtendedColor::default_color(), ExtendedColor::Default);
        assert_eq!(
            ExtendedColor::cmy(10, 20, 30),
            ExtendedColor::Cmy {
                c: 10,
                m: 20,
                y: 30
            }
        );
        assert_eq!(
            ExtendedColor::cmyk(10, 20, 30, 40),
            ExtendedColor::Cmyk {
                c: 10,
                m: 20,
                y: 30,
                k: 40
            }
        );
    }

    // =========================================================================
    // Semicolon-separated extended color tests (xterm format via TryFromAnsiIter)
    // =========================================================================

    #[test]
    fn test_parse_semicolon_foreground_256() {
        // 38;5;196 format (xterm style)
        let attr = SgrAttr::try_from_ansi(b"38;5;196").unwrap();
        assert_eq!(attr, SgrAttr::Foreground(ExtendedColor::Palette(196)));
    }

    #[test]
    fn test_parse_semicolon_background_256() {
        // 48;5;21 format
        let attr = SgrAttr::try_from_ansi(b"48;5;21").unwrap();
        assert_eq!(attr, SgrAttr::Background(ExtendedColor::Palette(21)));
    }

    #[test]
    fn test_parse_semicolon_foreground_rgb() {
        // 38;2;255;128;0 format (xterm style)
        let attr = SgrAttr::try_from_ansi(b"38;2;255;128;0").unwrap();
        assert_eq!(
            attr,
            SgrAttr::Foreground(ExtendedColor::Rgb {
                r: 255,
                g: 128,
                b: 0
            })
        );
    }

    #[test]
    fn test_parse_semicolon_background_rgb() {
        // 48;2;0;0;128 format
        let attr = SgrAttr::try_from_ansi(b"48;2;0;0;128").unwrap();
        assert_eq!(
            attr,
            SgrAttr::Background(ExtendedColor::Rgb { r: 0, g: 0, b: 128 })
        );
    }

    #[test]
    fn test_parse_semicolon_underline_color_256() {
        // 58;5;208 format
        let attr = SgrAttr::try_from_ansi(b"58;5;208").unwrap();
        assert_eq!(attr, SgrAttr::UnderlineColor(ExtendedColor::Palette(208)));
    }

    #[test]
    fn test_parse_semicolon_underline_color_rgb() {
        // 58;2;255;0;0 format
        let attr = SgrAttr::try_from_ansi(b"58;2;255;0;0").unwrap();
        assert_eq!(
            attr,
            SgrAttr::UnderlineColor(ExtendedColor::Rgb { r: 255, g: 0, b: 0 })
        );
    }

    // =========================================================================
    // parse_sgr_params tests (multiple attributes in one sequence)
    // =========================================================================

    #[test]
    fn test_parse_sgr_params_simple() {
        // 1;31 = bold + red foreground
        let attrs = parse_sgr_params(b"1;31").unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0], SgrAttr::Bold);
        assert_eq!(attrs[1], SgrAttr::ForegroundRed);
    }

    #[test]
    fn test_parse_sgr_params_with_256_color() {
        // 1;38;5;196 = bold + 256-color foreground
        let attrs = parse_sgr_params(b"1;38;5;196").unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0], SgrAttr::Bold);
        assert_eq!(attrs[1], SgrAttr::Foreground(ExtendedColor::Palette(196)));
    }

    #[test]
    fn test_parse_sgr_params_with_rgb_color() {
        // 1;38;2;255;128;0 = bold + RGB foreground
        let attrs = parse_sgr_params(b"1;38;2;255;128;0").unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0], SgrAttr::Bold);
        assert_eq!(
            attrs[1],
            SgrAttr::Foreground(ExtendedColor::Rgb {
                r: 255,
                g: 128,
                b: 0
            })
        );
    }

    #[test]
    fn test_parse_sgr_params_complex() {
        // 1;3;38;5;196;48;2;0;0;128 = bold + italic + 256 fg + RGB bg
        let attrs = parse_sgr_params(b"1;3;38;5;196;48;2;0;0;128").unwrap();
        assert_eq!(attrs.len(), 4);
        assert_eq!(attrs[0], SgrAttr::Bold);
        assert_eq!(attrs[1], SgrAttr::Italic);
        assert_eq!(attrs[2], SgrAttr::Foreground(ExtendedColor::Palette(196)));
        assert_eq!(
            attrs[3],
            SgrAttr::Background(ExtendedColor::Rgb { r: 0, g: 0, b: 128 })
        );
    }

    #[test]
    fn test_parse_sgr_params_reset_only() {
        let attrs = parse_sgr_params(b"0").unwrap();
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0], SgrAttr::Reset);
    }

    #[test]
    fn test_parse_sgr_params_with_colon_format() {
        // Mix of semicolon and colon formats: 1;4:3;38:5:196
        let attrs = parse_sgr_params(b"1;4:3;38:5:196").unwrap();
        assert_eq!(attrs.len(), 3);
        assert_eq!(attrs[0], SgrAttr::Bold);
        assert_eq!(attrs[1], SgrAttr::UnderlineStyle(UnderlineStyle::Curly));
        assert_eq!(attrs[2], SgrAttr::Foreground(ExtendedColor::Palette(196)));
    }

    #[test]
    fn test_parse_sgr_params_fg_and_bg_extended() {
        // 38;5;196;48;5;21 = 256-color fg + 256-color bg
        let attrs = parse_sgr_params(b"38;5;196;48;5;21").unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0], SgrAttr::Foreground(ExtendedColor::Palette(196)));
        assert_eq!(attrs[1], SgrAttr::Background(ExtendedColor::Palette(21)));
    }

    #[test]
    fn test_try_from_ansi_iter_directly() {
        // Test using TryFromAnsiIter directly
        let params: Vec<&[u8]> = vec![b"1", b"38", b"5", b"196", b"4"];
        let mut iter = params.into_iter();

        let attr1 = SgrAttr::try_from_ansi_iter(&mut iter).unwrap();
        assert_eq!(attr1, SgrAttr::Bold);

        let attr2 = SgrAttr::try_from_ansi_iter(&mut iter).unwrap();
        assert_eq!(attr2, SgrAttr::Foreground(ExtendedColor::Palette(196)));

        let attr3 = SgrAttr::try_from_ansi_iter(&mut iter).unwrap();
        assert_eq!(attr3, SgrAttr::Underline);

        // Iterator should be exhausted
        assert!(SgrAttr::try_from_ansi_iter(&mut iter).is_err());
    }

    // =========================================================================
    // LegacySgr (semicolon encoding) tests
    // =========================================================================

    #[test]
    fn test_legacy_sgr_foreground_256() {
        let sgr = LegacySgr::new(SgrAttr::foreground_256(196));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38;5;196m");
    }

    #[test]
    fn test_legacy_sgr_background_256() {
        let sgr = LegacySgr::new(SgrAttr::background_256(21));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[48;5;21m");
    }

    #[test]
    fn test_legacy_sgr_foreground_rgb() {
        let sgr = LegacySgr::new(SgrAttr::foreground_rgb(255, 128, 0));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38;2;255;128;0m");
    }

    #[test]
    fn test_legacy_sgr_background_rgb() {
        let sgr = LegacySgr::new(SgrAttr::background_rgb(0, 0, 128));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[48;2;0;0;128m");
    }

    #[test]
    fn test_legacy_sgr_underline_color_256() {
        let sgr = LegacySgr::new(SgrAttr::UnderlineColor(
            ExtendedColor::Palette(208),
        ));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[58;5;208m");
    }

    #[test]
    fn test_legacy_sgr_underline_color_rgb() {
        let sgr = LegacySgr::new(SgrAttr::UnderlineColor(ExtendedColor::Rgb {
            r: 255,
            g: 128,
            b: 64,
        }));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[58;2;255;128;64m");
    }

    #[test]
    fn test_legacy_sgr_default_fallback() {
        // Default color is only defined in ISO standard, uses colon even with legacy wrapper
        let sgr = LegacySgr::new(SgrAttr::Foreground(ExtendedColor::Default));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38:1m");
    }

    #[test]
    fn test_legacy_sgr_cmy_fallback() {
        // CMY is only defined in ISO standard, uses colon even with legacy wrapper
        let sgr = LegacySgr::new(SgrAttr::Foreground(ExtendedColor::Cmy {
            c: 100,
            m: 150,
            y: 200,
        }));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38:3:100:150:200m");
    }

    #[test]
    fn test_legacy_sgr_cmyk_fallback() {
        // CMYK is only defined in ISO standard, uses colon even with legacy wrapper
        let sgr = LegacySgr::new(SgrAttr::Foreground(ExtendedColor::Cmyk {
            c: 10,
            m: 20,
            y: 30,
            k: 40,
        }));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[38:4:10:20:30:40m");
    }

    #[test]
    fn test_legacy_sgr_simple_code() {
        // Simple codes should work the same way
        let sgr = LegacySgr::new(SgrAttr::Bold);
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[1m");
    }

    #[test]
    fn test_legacy_sgr_underline_style() {
        // Underline style always uses colon format (4:Ps)
        let sgr =
            LegacySgr::new(SgrAttr::UnderlineStyle(UnderlineStyle::Curly));
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[4:3m");
    }

    #[test]
    fn test_legacy_sgr_from_sgr() {
        let sgr = Sgr::new(SgrAttr::foreground_256(42));
        let legacy: LegacySgr = sgr.clone().into();
        let back: Sgr = legacy.into();
        assert_eq!(sgr, back);
    }

    #[test]
    fn test_legacy_sgr_from_attr() {
        let attr = SgrAttr::foreground_256(42);
        let legacy: LegacySgr = attr.into();
        assert_eq!(legacy.0.attr, SgrAttr::foreground_256(42));
    }

    #[test]
    fn test_legacy_sgr_reset() {
        let sgr = LegacySgr::reset();
        let mut buf = Vec::new();
        sgr.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1b[0m");
    }

    // =========================================================================
    // SGR Video Attribute Stack (XTPUSHSGR/XTPOPSGR) tests
    // =========================================================================

    #[test]
    fn test_push_video_attributes_all() {
        let push = PushVideoAttributes::all();
        assert_eq!(push.attributes, None);
        let mut buf = Vec::new();
        push.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(&buf, b"\x1b[#{");
    }

    #[test]
    fn test_push_video_attributes_selective_single() {
        let push =
            PushVideoAttributes::selective(SgrStackAttribute::FOREGROUND);
        assert_eq!(push.attributes, Some(SgrStackAttribute::FOREGROUND));
        let mut buf = Vec::new();
        push.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(&buf, b"\x1b[32#{");
    }

    #[test]
    fn test_push_video_attributes_selective_multiple() {
        let push = PushVideoAttributes::selective(
            SgrStackAttribute::FOREGROUND | SgrStackAttribute::BACKGROUND,
        );
        let mut buf = Vec::new();
        push.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(&buf, b"\x1b[96#{");
    }

    #[test]
    fn test_push_video_attributes_selective_all_colors() {
        let push = PushVideoAttributes::selective(
            SgrStackAttribute::FOREGROUND
                | SgrStackAttribute::BACKGROUND
                | SgrStackAttribute::UNDERLINE_COLOR,
        );
        let mut buf = Vec::new();
        push.encode_ansi_into(&mut buf).unwrap();
        // 32 + 64 + 128 = 224
        assert_eq!(&buf, b"\x1b[224#{");
    }

    #[test]
    fn test_pop_video_attributes() {
        let pop = PopVideoAttributes;
        let mut buf = Vec::new();
        pop.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(&buf, b"\x1b[#}");
    }

    #[test]
    fn test_sgr_stack_attribute_default() {
        let attr = SgrStackAttribute::default();
        assert!(attr.is_empty());
        assert_eq!(attr.bits(), 0);
    }

    #[test]
    fn test_sgr_stack_attribute_contains() {
        let colors =
            SgrStackAttribute::FOREGROUND | SgrStackAttribute::BACKGROUND;
        assert!(colors.contains(SgrStackAttribute::FOREGROUND));
        assert!(colors.contains(SgrStackAttribute::BACKGROUND));
        assert!(!colors.contains(SgrStackAttribute::UNDERLINE));
    }

    #[test]
    fn test_sgr_stack_attribute_bitwise_ops() {
        let mut attr = SgrStackAttribute::BOLD_FAINT;
        attr |= SgrStackAttribute::UNDERLINE;
        assert!(attr.contains(SgrStackAttribute::BOLD_FAINT));
        assert!(attr.contains(SgrStackAttribute::UNDERLINE));
        assert_eq!(attr.bits(), 3);

        let masked = attr & SgrStackAttribute::BOLD_FAINT;
        assert_eq!(masked.bits(), 1);
    }

    #[test]
    fn test_push_video_attributes_default() {
        let push = PushVideoAttributes::default();
        assert_eq!(push.attributes, None);
    }
}
