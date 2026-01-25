//! X11 color specification parsing and representation.
//!
//! This crate provides the [`XColor`] enum for representing colors in various
//! color spaces as defined by X11/Xcms, along with parsing from X11 color
//! specification strings and efficient encoding back to byte streams.
//!
//! # Supported Color Spaces
//!
//! - **RGB Device** (`rgb:`) - 16-bit components (0-65535)
//! - **RGB Intensity** (`rgbi:`) - Floating-point values (0.0-1.0)
//! - **CIE XYZ** (`CIEXYZ:`) - CIE 1931 XYZ color space
//! - **CIE u'v'Y** (`CIEuvY:`) - CIE 1976 u'v'Y color space
//! - **CIE xyY** (`CIExyY:`) - CIE xyY chromaticity coordinates
//! - **CIE L\*a\*b\*** (`CIELab:`) - CIE 1976 L\*a\*b\* color space
//! - **CIE L\*u\*v\*** (`CIELuv:`) - CIE 1976 L\*u\*v\* color space
//! - **TekHVC** (`TekHVC:`) - Tektronix HVC color space
//!
//! # Legacy Sharp Syntax
//!
//! The `#` syntax is also supported and parsed as RGB Device:
//! - `#RGB` (4-bit per component)
//! - `#RRGGBB` (8-bit per component)
//! - `#RRRGGGBBB` (12-bit per component)
//! - `#RRRRGGGGBBBB` (16-bit per component)
//!
//! # Named Colors
//!
//! X11 named colors from `rgb.txt` are also supported (case-insensitive,
//! spaces optional):
//! - `red`, `green`, `blue`, `white`, `black`
//! - `DarkSlateGray`, `dark slate gray`, `darkslategray`
//! - And many more (~750 colors)
//!
//! See <https://tronche.com/gui/x/xlib/color/structures.html>

use std::fmt;
use std::io::{self, Write};
use std::str::FromStr;

use palette::white_point::D65;
use palette::{FromColor, IntoColor, Lab, Lchuv, LinSrgb, Luv, Srgb, Xyz, Yxy};

// Include the generated named colors lookup table
mod named_colors {
    include!(concat!(env!("OUT_DIR"), "/named_colors.rs"));
}

pub use named_colors::{NAMED_COLOR_COUNT, lookup_named_color};

/// Parse error with byte offset information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// Unknown color format prefix.
    UnknownFormat,
    /// Invalid hex digit at the given offset.
    InvalidHex { offset: usize },
    /// Invalid number of hex digits in sharp syntax.
    InvalidSharpLength { len: usize },
    /// Missing color component.
    MissingComponent,
    /// Too many color components.
    TooManyComponents,
    /// Invalid floating-point value at the given offset.
    InvalidFloat { offset: usize },
    /// Floating-point value out of range.
    OutOfRange,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty color specification"),
            ParseError::UnknownFormat => write!(f, "unknown color format"),
            ParseError::InvalidHex { offset } => {
                write!(f, "invalid hex value at offset {offset}")
            }
            ParseError::InvalidSharpLength { len } => {
                write!(
                    f,
                    "invalid # color length: {len} (expected 3, 6, 9, or 12)"
                )
            }
            ParseError::MissingComponent => {
                write!(f, "missing color component")
            }
            ParseError::TooManyComponents => {
                write!(f, "too many color components")
            }
            ParseError::InvalidFloat { offset } => {
                write!(f, "invalid floating-point value at offset {offset}")
            }
            ParseError::OutOfRange => write!(f, "value out of range"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Error type for encoding X11 color specifications.
#[derive(Debug)]
pub enum EncodeError {
    /// Buffer overflow - not enough space to write encoded data.
    BufferOverflow(usize),
    /// I/O error during encoding.
    IoError(io::Error),
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::BufferOverflow(n) => {
                write!(f, "buffer overflow: {n} bytes could not be written")
            }
            EncodeError::IoError(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for EncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EncodeError::IoError(e) => Some(e),
            EncodeError::BufferOverflow(_) => None,
        }
    }
}

impl From<EncodeError> for io::Error {
    fn from(err: EncodeError) -> Self {
        match err {
            EncodeError::BufferOverflow(n) => io::Error::new(
                io::ErrorKind::WriteZero,
                format!("buffer overflow: {n} bytes could not be written"),
            ),
            EncodeError::IoError(e) => e,
        }
    }
}

// Hex encoding lookup table for lowercase hex digits
const HEX_ENCODE: &[u8; 16] = b"0123456789abcdef";

/// A writer wrapper that tracks bytes written and overflow.
struct CountingWriter<W> {
    inner: W,
    written: usize,
    overflow: usize,
}

impl<W: Write> CountingWriter<W> {
    #[inline]
    fn new(inner: W) -> Self {
        Self {
            inner,
            written: 0,
            overflow: 0,
        }
    }

    #[inline]
    fn overflow(&self) -> usize {
        self.overflow
    }
}

impl<W: Write> Write for CountingWriter<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let requested = buf.len();
        let n = self.inner.write(buf)?;
        self.written += n;
        if n < requested {
            self.overflow += requested - n;
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Write bytes to a sink, returning the number of bytes written.
///
/// Uses CountingWriter to properly detect partial writes (overflow).
#[inline]
fn write_bytes<W: Write + ?Sized>(
    sink: &mut W,
    bytes: &[u8],
) -> Result<usize, EncodeError> {
    let mut w = CountingWriter::new(sink);
    match w.write(bytes) {
        Err(ref e) if e.kind() == io::ErrorKind::WriteZero => {
            Err(EncodeError::BufferOverflow(w.overflow()))
        }
        Err(e) => Err(EncodeError::IoError(e)),
        Ok(_n) if w.overflow() > 0 => {
            Err(EncodeError::BufferOverflow(w.overflow()))
        }
        Ok(n) => Ok(n),
    }
}

/// Write a u16 as 4 lowercase hex digits.
#[inline]
fn write_hex16<W: Write + ?Sized>(
    sink: &mut W,
    value: u16,
) -> Result<usize, EncodeError> {
    let bytes = [
        HEX_ENCODE[((value >> 12) & 0xF) as usize],
        HEX_ENCODE[((value >> 8) & 0xF) as usize],
        HEX_ENCODE[((value >> 4) & 0xF) as usize],
        HEX_ENCODE[(value & 0xF) as usize],
    ];
    write_bytes(sink, &bytes)
}

/// Write an f64 using ryu for efficient formatting.
#[inline]
fn write_f64<W: Write + ?Sized>(
    sink: &mut W,
    value: f64,
) -> Result<usize, EncodeError> {
    let mut buffer = zmij::Buffer::new();
    let s = buffer.format(value);
    write_bytes(sink, s.as_bytes())
}

/// X11 color specification.
///
/// Represents a color in one of several supported color spaces, matching
/// the `XcmsColor` union structure from X11.
///
/// # Examples
///
/// ```
/// use xparsecolor::XColor;
///
/// // Parse RGB device color
/// let color: XColor = "rgb:ffff/8080/0000".parse().unwrap();
/// assert!(matches!(color, XColor::Rgb { .. }));
///
/// // Parse RGB intensity color
/// let color: XColor = "rgbi:1.0/0.5/0.0".parse().unwrap();
/// assert!(matches!(color, XColor::RgbIntensity { .. }));
///
/// // Parse legacy sharp syntax
/// let color: XColor = "#ff8000".parse().unwrap();
/// assert!(matches!(color, XColor::Rgb { .. }));
///
/// // Parse named color
/// let color: XColor = "dark slate gray".parse().unwrap();
/// assert!(matches!(color, XColor::Rgb { .. }));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum XColor {
    /// RGB Device color with 16-bit components (0x0000 to 0xffff).
    ///
    /// These values are appropriate for the specified output device and
    /// are interchangeable with the red, green, and blue values in an
    /// X11 `XColor` structure.
    Rgb {
        /// Red component (0-65535).
        red: u16,
        /// Green component (0-65535).
        green: u16,
        /// Blue component (0-65535).
        blue: u16,
    },

    /// RGB Intensity with floating-point components (0.0 to 1.0).
    ///
    /// Linear intensity values where 1.0 indicates full intensity,
    /// 0.5 half intensity, and so on. Note: these are NOT gamma corrected.
    RgbIntensity {
        /// Red intensity (0.0-1.0).
        red: f64,
        /// Green intensity (0.0-1.0).
        green: f64,
        /// Blue intensity (0.0-1.0).
        blue: f64,
    },

    /// CIE XYZ color space.
    CieXyz {
        /// X tristimulus value.
        x: f64,
        /// Y tristimulus value (luminance, 0.0-1.0).
        y: f64,
        /// Z tristimulus value.
        z: f64,
    },

    /// CIE 1976 u'v'Y color space.
    CieUvY {
        /// u' chromaticity coordinate (0.0 to ~0.6).
        u_prime: f64,
        /// v' chromaticity coordinate (0.0 to ~0.6).
        v_prime: f64,
        /// Y luminance (0.0-1.0).
        y: f64,
    },

    /// CIE xyY color space (chromaticity coordinates).
    CieXyY {
        /// x chromaticity coordinate (0.0 to ~0.75).
        x: f64,
        /// y chromaticity coordinate (0.0 to ~0.85).
        y: f64,
        /// Y luminance (0.0-1.0).
        y_luminance: f64,
    },

    /// CIE 1976 L*a*b* color space.
    CieLab {
        /// L* lightness (0.0-100.0).
        l_star: f64,
        /// a* green-red axis.
        a_star: f64,
        /// b* blue-yellow axis.
        b_star: f64,
    },

    /// CIE 1976 L*u*v* color space.
    CieLuv {
        /// L* lightness (0.0-100.0).
        l_star: f64,
        /// u* chromaticity coordinate.
        u_star: f64,
        /// v* chromaticity coordinate.
        v_star: f64,
    },

    /// Tektronix HVC color space.
    TekHvc {
        /// Hue angle (0.0-360.0).
        h: f64,
        /// Value/lightness (0.0-100.0).
        v: f64,
        /// Chroma (0.0-100.0).
        c: f64,
    },
}

impl Default for XColor {
    fn default() -> Self {
        XColor::Rgb {
            red: 0,
            green: 0,
            blue: 0,
        }
    }
}

impl XColor {
    /// Create a new RGB Device color from 16-bit components.
    #[must_use]
    pub const fn rgb(red: u16, green: u16, blue: u16) -> Self {
        XColor::Rgb { red, green, blue }
    }

    /// Create a new RGB Device color from 8-bit components.
    ///
    /// The 8-bit values are expanded to 16-bit by repeating the byte
    /// (matching `XParseColor`'s 2-digit hex behavior).
    #[must_use]
    pub const fn from_rgb8(red: u8, green: u8, blue: u8) -> Self {
        XColor::Rgb {
            red: (red as u16) << 8 | red as u16,
            green: (green as u16) << 8 | green as u16,
            blue: (blue as u16) << 8 | blue as u16,
        }
    }

    /// Create a new RGB Intensity color from floating-point values.
    #[must_use]
    pub const fn rgb_intensity(red: f64, green: f64, blue: f64) -> Self {
        XColor::RgbIntensity { red, green, blue }
    }

    /// Create a new CIE XYZ color.
    #[must_use]
    pub const fn cie_xyz(x: f64, y: f64, z: f64) -> Self {
        XColor::CieXyz { x, y, z }
    }

    /// Create a new CIE u'v'Y color.
    #[must_use]
    pub const fn cie_uvy(u_prime: f64, v_prime: f64, y: f64) -> Self {
        XColor::CieUvY {
            u_prime,
            v_prime,
            y,
        }
    }

    /// Create a new CIE xyY color.
    #[must_use]
    pub const fn cie_xyy(x: f64, y: f64, y_luminance: f64) -> Self {
        XColor::CieXyY { x, y, y_luminance }
    }

    /// Create a new CIE L*a*b* color.
    #[must_use]
    pub const fn cie_lab(l_star: f64, a_star: f64, b_star: f64) -> Self {
        XColor::CieLab {
            l_star,
            a_star,
            b_star,
        }
    }

    /// Create a new CIE L*u*v* color.
    #[must_use]
    pub const fn cie_luv(l_star: f64, u_star: f64, v_star: f64) -> Self {
        XColor::CieLuv {
            l_star,
            u_star,
            v_star,
        }
    }

    /// Create a new TekHVC color.
    #[must_use]
    pub const fn tek_hvc(h: f64, v: f64, c: f64) -> Self {
        XColor::TekHvc { h, v, c }
    }

    /// Encode this color into a writer.
    ///
    /// This is the most efficient encoding method, writing directly to the
    /// provided sink without intermediate allocations.
    ///
    /// Returns the number of bytes written on success.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the sink fails.
    pub fn encode_into<W: Write + ?Sized>(
        &self,
        w: &mut W,
    ) -> Result<usize, EncodeError> {
        match *self {
            XColor::Rgb { red, green, blue } => {
                let mut n = write_bytes(w, b"rgb:")?;
                n += write_hex16(w, red)?;
                n += write_bytes(w, b"/")?;
                n += write_hex16(w, green)?;
                n += write_bytes(w, b"/")?;
                n += write_hex16(w, blue)?;
                Ok(n)
            }
            XColor::RgbIntensity { red, green, blue } => {
                let mut n = write_bytes(w, b"rgbi:")?;
                n += write_f64(w, red)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, green)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, blue)?;
                Ok(n)
            }
            XColor::CieXyz { x, y, z } => {
                let mut n = write_bytes(w, b"CIEXYZ:")?;
                n += write_f64(w, x)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, y)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, z)?;
                Ok(n)
            }
            XColor::CieUvY {
                u_prime,
                v_prime,
                y,
            } => {
                let mut n = write_bytes(w, b"CIEuvY:")?;
                n += write_f64(w, u_prime)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, v_prime)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, y)?;
                Ok(n)
            }
            XColor::CieXyY { x, y, y_luminance } => {
                let mut n = write_bytes(w, b"CIExyY:")?;
                n += write_f64(w, x)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, y)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, y_luminance)?;
                Ok(n)
            }
            XColor::CieLab {
                l_star,
                a_star,
                b_star,
            } => {
                let mut n = write_bytes(w, b"CIELab:")?;
                n += write_f64(w, l_star)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, a_star)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, b_star)?;
                Ok(n)
            }
            XColor::CieLuv {
                l_star,
                u_star,
                v_star,
            } => {
                let mut n = write_bytes(w, b"CIELuv:")?;
                n += write_f64(w, l_star)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, u_star)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, v_star)?;
                Ok(n)
            }
            XColor::TekHvc { h, v, c } => {
                let mut n = write_bytes(w, b"TekHVC:")?;
                n += write_f64(w, h)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, v)?;
                n += write_bytes(w, b"/")?;
                n += write_f64(w, c)?;
                Ok(n)
            }
        }
    }

    /// Encode this color into a byte slice.
    ///
    /// Returns the number of bytes written on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too small.
    #[inline]
    pub fn encode_into_slice(
        &self,
        buf: &mut [u8],
    ) -> Result<usize, EncodeError> {
        self.encode_into(&mut &mut buf[..])
    }

    /// Encode this color into a new `Vec<u8>`.
    ///
    /// # Errors
    ///
    /// Returns an error if encoding fails (should not happen with Vec).
    #[inline]
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut v = Vec::with_capacity(32);
        self.encode_into(&mut v)?;
        Ok(v)
    }

    /// Parse a color from a byte slice.
    pub fn try_from_bytes(input: &[u8]) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::Empty);
        }

        // Match prefix bytes directly
        if input.starts_with(b"rgb:") {
            parse_rgb_device_bytes(&input[4..])
        } else if input.starts_with(b"#") {
            parse_sharp_bytes(&input[1..])
        } else if input.starts_with(b"rgbi:") {
            parse_rgbi_bytes(&input[5..])
        } else if input.starts_with(b"CIEXYZ:") {
            parse_ciexyz_bytes(&input[7..])
        } else if input.starts_with(b"CIEuvY:") {
            parse_cieuvy_bytes(&input[7..])
        } else if input.starts_with(b"CIExyY:") {
            parse_ciexyy_bytes(&input[7..])
        } else if input.starts_with(b"CIELab:") {
            parse_cielab_bytes(&input[7..])
        } else if input.starts_with(b"CIELuv:") {
            parse_cieluv_bytes(&input[7..])
        } else if input.starts_with(b"TekHVC:") {
            parse_tekhvc_bytes(&input[7..])
        } else {
            parse_named_bytes(input)
        }
    }

    /// Convert this color to RGB Device format.
    ///
    /// For device-independent color spaces, this performs the appropriate
    /// color space conversion using the D65 white point and sRGB primaries.
    #[must_use]
    pub fn to_rgb(&self) -> XColor {
        match *self {
            XColor::Rgb { .. } => *self,
            XColor::RgbIntensity { red, green, blue } => {
                // RGBi is linear RGB, convert through LinSrgb -> Srgb
                Srgb::from_linear(LinSrgb::new(red, green, blue)).into()
            }
            XColor::CieXyz { x, y, z } => {
                Srgb::from_color(Xyz::<D65, f64>::new(x, y, z)).into()
            }
            XColor::CieUvY {
                u_prime,
                v_prime,
                y,
            } => {
                // Convert CIE u'v'Y to XYZ
                // u' = 4X / (X + 15Y + 3Z)
                // v' = 9Y / (X + 15Y + 3Z)
                if v_prime.abs() < 1e-10 {
                    return XColor::rgb(0, 0, 0);
                }
                let x_val = 9.0 * y * u_prime / (4.0 * v_prime);
                let z_val = (12.0 - 3.0 * u_prime - 20.0 * v_prime) * y
                    / (4.0 * v_prime);
                Srgb::from_color(Xyz::<D65, f64>::new(x_val, y, z_val)).into()
            }
            XColor::CieXyY { x, y, y_luminance } => {
                Srgb::from_color(Yxy::<D65, f64>::new(x, y, y_luminance)).into()
            }
            XColor::CieLab {
                l_star,
                a_star,
                b_star,
            } => Srgb::from_color(Lab::<D65, f64>::new(l_star, a_star, b_star))
                .into(),
            XColor::CieLuv {
                l_star,
                u_star,
                v_star,
            } => Srgb::from_color(Luv::<D65, f64>::new(l_star, u_star, v_star))
                .into(),
            XColor::TekHvc { h, v, c } => {
                // TekHVC is a cylindrical form of CIE L*u*v*
                // H = hue angle, V = L* (lightness), C = chroma
                Srgb::from_color(Lchuv::<D65, f64>::new(v, c, h)).into()
            }
        }
    }

    /// Convert this color to RGB Intensity format.
    #[must_use]
    pub fn to_rgb_intensity(&self) -> XColor {
        match *self {
            XColor::RgbIntensity { .. } => *self,
            XColor::Rgb { red, green, blue } => {
                // Convert sRGB to linear RGB
                let srgb: Srgb<f64> = Srgb::new(
                    red as f64 / 65535.0,
                    green as f64 / 65535.0,
                    blue as f64 / 65535.0,
                );
                let lin: LinSrgb<f64> = srgb.into_linear();
                XColor::RgbIntensity {
                    red: lin.red,
                    green: lin.green,
                    blue: lin.blue,
                }
            }
            _ => {
                // Convert to RGB first, then to intensity
                let rgb = self.to_rgb();
                rgb.to_rgb_intensity()
            }
        }
    }

    /// Convert to 8-bit RGB values.
    ///
    /// First converts to RGB Device format if necessary, then takes the
    /// high byte of each component.
    #[must_use]
    pub fn to_rgb8(&self) -> (u8, u8, u8) {
        match self.to_rgb() {
            XColor::Rgb { red, green, blue } => {
                ((red >> 8) as u8, (green >> 8) as u8, (blue >> 8) as u8)
            }
            _ => unreachable!(),
        }
    }

    /// Get RGB Device components if this is an RGB color.
    ///
    /// Returns `None` if this is not an RGB Device color.
    #[must_use]
    pub const fn as_rgb(&self) -> Option<(u16, u16, u16)> {
        match *self {
            XColor::Rgb { red, green, blue } => Some((red, green, blue)),
            _ => None,
        }
    }

    /// Get RGB Intensity components if this is an RGBi color.
    ///
    /// Returns `None` if this is not an RGB Intensity color.
    #[must_use]
    pub const fn as_rgb_intensity(&self) -> Option<(f64, f64, f64)> {
        match *self {
            XColor::RgbIntensity { red, green, blue } => {
                Some((red, green, blue))
            }
            _ => None,
        }
    }

    /// Convert to a palette `Srgb<f64>` color.
    #[must_use]
    pub fn to_srgb(&self) -> Srgb<f64> {
        (*self).into()
    }

    /// Convert to a palette `LinSrgb<f64>` (linear sRGB) color.
    #[must_use]
    pub fn to_lin_srgb(&self) -> LinSrgb<f64> {
        self.to_srgb().into_linear()
    }

    /// Convert to a palette `Xyz<D65, f64>` color.
    #[must_use]
    pub fn to_xyz(&self) -> Xyz<D65, f64> {
        self.to_lin_srgb().into_color()
    }

    /// Convert to a palette `Lab<D65, f64>` color.
    #[must_use]
    pub fn to_lab(&self) -> Lab<D65, f64> {
        self.to_xyz().into_color()
    }

    /// Convert to a palette `Luv<D65, f64>` color.
    #[must_use]
    pub fn to_luv(&self) -> Luv<D65, f64> {
        self.to_xyz().into_color()
    }

    /// Create from a palette `Srgb` color.
    #[must_use]
    pub fn from_srgb(srgb: Srgb<f64>) -> Self {
        srgb.into()
    }

    /// Create from a palette `LinSrgb` (linear sRGB) color.
    #[must_use]
    pub fn from_lin_srgb(lin: LinSrgb<f64>) -> Self {
        Self::from_srgb(Srgb::from_linear(lin))
    }

    /// Create from a palette `Xyz` color.
    #[must_use]
    pub fn from_xyz(xyz: Xyz<D65, f64>) -> Self {
        Self::from_srgb(Srgb::from_color(xyz))
    }

    /// Create from a palette `Lab` color.
    #[must_use]
    pub fn from_lab(lab: Lab<D65, f64>) -> Self {
        Self::from_srgb(Srgb::from_color(lab))
    }

    /// Create from a palette `Luv` color.
    #[must_use]
    pub fn from_luv(luv: Luv<D65, f64>) -> Self {
        Self::from_srgb(Srgb::from_color(luv))
    }
}

impl fmt::Display for XColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            XColor::Rgb { red, green, blue } => {
                write!(f, "rgb:{red:04x}/{green:04x}/{blue:04x}")
            }
            XColor::RgbIntensity { red, green, blue } => {
                write!(f, "rgbi:{red}/{green}/{blue}")
            }
            XColor::CieXyz { x, y, z } => {
                write!(f, "CIEXYZ:{x}/{y}/{z}")
            }
            XColor::CieUvY {
                u_prime,
                v_prime,
                y,
            } => {
                write!(f, "CIEuvY:{u_prime}/{v_prime}/{y}")
            }
            XColor::CieXyY { x, y, y_luminance } => {
                write!(f, "CIExyY:{x}/{y}/{y_luminance}")
            }
            XColor::CieLab {
                l_star,
                a_star,
                b_star,
            } => {
                write!(f, "CIELab:{l_star}/{a_star}/{b_star}")
            }
            XColor::CieLuv {
                l_star,
                u_star,
                v_star,
            } => {
                write!(f, "CIELuv:{l_star}/{u_star}/{v_star}")
            }
            XColor::TekHvc { h, v, c } => {
                write!(f, "TekHVC:{h}/{v}/{c}")
            }
        }
    }
}

impl TryFrom<&[u8]> for XColor {
    type Error = ParseError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_bytes(value)
    }
}

impl From<Srgb<f64>> for XColor {
    fn from(srgb: Srgb<f64>) -> Self {
        XColor::Rgb {
            red: (srgb.red.clamp(0.0, 1.0) * 65535.0).round() as u16,
            green: (srgb.green.clamp(0.0, 1.0) * 65535.0).round() as u16,
            blue: (srgb.blue.clamp(0.0, 1.0) * 65535.0).round() as u16,
        }
    }
}

impl From<XColor> for Srgb<f64> {
    fn from(color: XColor) -> Self {
        match color.to_rgb() {
            XColor::Rgb { red, green, blue } => Srgb::new(
                red as f64 / 65535.0,
                green as f64 / 65535.0,
                blue as f64 / 65535.0,
            ),
            _ => unreachable!(),
        }
    }
}

/// Error type for parsing X11 color specifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XColorParseError {
    /// The input string is empty.
    Empty,
    /// Unknown color format prefix.
    UnknownFormat(String),
    /// Invalid hex digit in color specification.
    InvalidHex(String),
    /// Invalid number of hex digits in sharp syntax.
    InvalidSharpLength(usize),
    /// Missing color component in specification.
    MissingComponent,
    /// Too many color components in specification.
    TooManyComponents,
    /// Invalid floating-point value.
    InvalidFloat(String),
    /// Floating-point value out of range (for rgbi:).
    OutOfRange(String),
}

impl fmt::Display for XColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XColorParseError::Empty => write!(f, "empty color specification"),
            XColorParseError::UnknownFormat(s) => {
                write!(f, "unknown color format: {s}")
            }
            XColorParseError::InvalidHex(s) => {
                write!(f, "invalid hex value: {s}")
            }
            XColorParseError::InvalidSharpLength(len) => {
                write!(
                    f,
                    "invalid # color length: {len} (expected 3, 6, 9, or 12)"
                )
            }
            XColorParseError::MissingComponent => {
                write!(f, "missing color component")
            }
            XColorParseError::TooManyComponents => {
                write!(f, "too many color components")
            }
            XColorParseError::InvalidFloat(s) => {
                write!(f, "invalid floating-point value: {s}")
            }
            XColorParseError::OutOfRange(s) => {
                write!(f, "value out of range: {s}")
            }
        }
    }
}

impl std::error::Error for XColorParseError {}

impl FromStr for XColor {
    type Err = XColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_bytes(s.as_bytes()).map_err(|e| e.into())
    }
}

impl From<ParseError> for XColorParseError {
    fn from(e: ParseError) -> Self {
        match e {
            ParseError::Empty => XColorParseError::Empty,
            ParseError::UnknownFormat => {
                XColorParseError::UnknownFormat(String::new())
            }
            ParseError::InvalidHex { .. } => {
                XColorParseError::InvalidHex(String::new())
            }
            ParseError::InvalidSharpLength { len } => {
                XColorParseError::InvalidSharpLength(len)
            }
            ParseError::MissingComponent => XColorParseError::MissingComponent,
            ParseError::TooManyComponents => {
                XColorParseError::TooManyComponents
            }
            ParseError::InvalidFloat { .. } => {
                XColorParseError::InvalidFloat(String::new())
            }
            ParseError::OutOfRange => {
                XColorParseError::OutOfRange(String::new())
            }
        }
    }
}

// ============================
// Byte-based parsing functions
// ============================

/// Hex digit lookup table: maps ASCII byte to its hex value (0-15), or 0xFF if invalid.
const HEX_DECODE: [u8; 256] = {
    let mut table = [0xFFu8; 256];
    let mut i = 0u8;
    while i < 10 {
        table[(b'0' + i) as usize] = i;
        i += 1;
    }
    let mut i = 0u8;
    while i < 6 {
        table[(b'a' + i) as usize] = 10 + i;
        table[(b'A' + i) as usize] = 10 + i;
        i += 1;
    }
    table
};

/// Parse a single hex digit from a byte.
#[inline(always)]
fn hex_digit(b: u8) -> Option<u16> {
    let v = HEX_DECODE[b as usize];
    if v == 0xFF { None } else { Some(v as u16) }
}

/// Parse 1-4 hex digits from a byte slice, returning (value, bytes_consumed).
#[inline]
fn parse_hex_bytes(
    input: &[u8],
    offset: usize,
) -> Result<(u16, usize), ParseError> {
    if input.is_empty() {
        return Err(ParseError::MissingComponent);
    }

    let mut value: u16 = 0;
    let mut count = 0usize;

    for (i, &b) in input.iter().enumerate() {
        if b == b'/' || i >= 4 {
            break;
        }
        let digit = hex_digit(b)
            .ok_or(ParseError::InvalidHex { offset: offset + i })?;
        value = (value << 4) | digit;
        count += 1;
    }

    if count == 0 {
        return Err(ParseError::MissingComponent);
    }

    // Scale based on digit count
    let scaled = match count {
        1 => value << 12 | value << 8 | value << 4 | value,
        2 => value << 8 | value,
        3 => value << 4 | (value >> 8),
        4 => value,
        _ => unreachable!(),
    };

    Ok((scaled, count))
}

/// Parse `rgb:r/g/b` format from bytes.
fn parse_rgb_device_bytes(input: &[u8]) -> Result<XColor, ParseError> {
    let (red, r_len) = parse_hex_bytes(input, 4)?;

    let rest = &input[r_len..];
    if rest.is_empty() || rest[0] != b'/' {
        return Err(ParseError::MissingComponent);
    }

    let (green, g_len) = parse_hex_bytes(&rest[1..], 4 + r_len + 1)?;

    let rest = &rest[1 + g_len..];
    if rest.is_empty() || rest[0] != b'/' {
        return Err(ParseError::MissingComponent);
    }

    let (blue, b_len) = parse_hex_bytes(&rest[1..], 4 + r_len + 1 + g_len + 1)?;

    let rest = &rest[1 + b_len..];
    if !rest.is_empty() {
        return Err(ParseError::TooManyComponents);
    }

    Ok(XColor::Rgb { red, green, blue })
}

/// Parse `#RGB`, `#RRGGBB`, `#RRRGGGBBB`, or `#RRRRGGGGBBBB` from bytes.
fn parse_sharp_bytes(input: &[u8]) -> Result<XColor, ParseError> {
    let len = input.len();

    let component_len = match len {
        3 | 6 | 9 | 12 => len / 3,
        _ => return Err(ParseError::InvalidSharpLength { len }),
    };

    let parse_component = |start: usize| -> Result<u16, ParseError> {
        let mut value: u16 = 0;
        for i in 0..component_len {
            let digit =
                hex_digit(input[start + i]).ok_or(ParseError::InvalidHex {
                    offset: 1 + start + i,
                })?;
            value = (value << 4) | digit;
        }
        Ok(match component_len {
            1 => value << 12,
            2 => value << 8,
            3 => value << 4,
            4 => value,
            _ => unreachable!(),
        })
    };

    let red = parse_component(0)?;
    let green = parse_component(component_len)?;
    let blue = parse_component(component_len * 2)?;

    Ok(XColor::Rgb { red, green, blue })
}

/// Parse a floating-point number from bytes, returning (value, bytes_consumed).
/// Stops at '/' or end of input.
fn parse_float_bytes(
    input: &[u8],
    offset: usize,
) -> Result<(f64, usize), ParseError> {
    if input.is_empty() {
        return Err(ParseError::MissingComponent);
    }

    // Find end of number (at '/' or end of input)
    let end = input.iter().position(|&b| b == b'/').unwrap_or(input.len());

    if end == 0 {
        return Err(ParseError::MissingComponent);
    }

    // SAFETY: We're parsing ASCII float characters, which are valid UTF-8
    let s = std::str::from_utf8(&input[..end])
        .map_err(|_| ParseError::InvalidFloat { offset })?;

    let value: f64 =
        s.parse().map_err(|_| ParseError::InvalidFloat { offset })?;

    Ok((value, end))
}

/// Parse a floating-point intensity (0.0-1.0) from bytes.
fn parse_float_intensity_bytes(
    input: &[u8],
    offset: usize,
) -> Result<(f64, usize), ParseError> {
    let (value, len) = parse_float_bytes(input, offset)?;
    if !(0.0..=1.0).contains(&value) {
        return Err(ParseError::OutOfRange);
    }
    Ok((value, len))
}

/// Parse `rgbi:r/g/b` format from bytes.
fn parse_rgbi_bytes(input: &[u8]) -> Result<XColor, ParseError> {
    let (red, r_len) = parse_float_intensity_bytes(input, 5)?;

    let rest = &input[r_len..];
    if rest.is_empty() || rest[0] != b'/' {
        return Err(ParseError::MissingComponent);
    }

    let (green, g_len) =
        parse_float_intensity_bytes(&rest[1..], 5 + r_len + 1)?;

    let rest = &rest[1 + g_len..];
    if rest.is_empty() || rest[0] != b'/' {
        return Err(ParseError::MissingComponent);
    }

    let (blue, b_len) =
        parse_float_intensity_bytes(&rest[1..], 5 + r_len + 1 + g_len + 1)?;

    let rest = &rest[1 + b_len..];
    if !rest.is_empty() {
        return Err(ParseError::TooManyComponents);
    }

    Ok(XColor::RgbIntensity { red, green, blue })
}

/// Parse three floats separated by '/' from bytes.
fn parse_three_floats_bytes(
    input: &[u8],
    base_offset: usize,
) -> Result<(f64, f64, f64), ParseError> {
    let (a, a_len) = parse_float_bytes(input, base_offset)?;

    let rest = &input[a_len..];
    if rest.is_empty() || rest[0] != b'/' {
        return Err(ParseError::MissingComponent);
    }

    let (b, b_len) = parse_float_bytes(&rest[1..], base_offset + a_len + 1)?;

    let rest = &rest[1 + b_len..];
    if rest.is_empty() || rest[0] != b'/' {
        return Err(ParseError::MissingComponent);
    }

    let (c, c_len) =
        parse_float_bytes(&rest[1..], base_offset + a_len + 1 + b_len + 1)?;

    let rest = &rest[1 + c_len..];
    if !rest.is_empty() {
        return Err(ParseError::TooManyComponents);
    }

    Ok((a, b, c))
}

fn parse_ciexyz_bytes(input: &[u8]) -> Result<XColor, ParseError> {
    let (x, y, z) = parse_three_floats_bytes(input, 7)?;
    Ok(XColor::CieXyz { x, y, z })
}

fn parse_cieuvy_bytes(input: &[u8]) -> Result<XColor, ParseError> {
    let (u_prime, v_prime, y) = parse_three_floats_bytes(input, 7)?;
    Ok(XColor::CieUvY {
        u_prime,
        v_prime,
        y,
    })
}

fn parse_ciexyy_bytes(input: &[u8]) -> Result<XColor, ParseError> {
    let (x, y, y_luminance) = parse_three_floats_bytes(input, 7)?;
    Ok(XColor::CieXyY { x, y, y_luminance })
}

fn parse_cielab_bytes(input: &[u8]) -> Result<XColor, ParseError> {
    let (l_star, a_star, b_star) = parse_three_floats_bytes(input, 7)?;
    Ok(XColor::CieLab {
        l_star,
        a_star,
        b_star,
    })
}

fn parse_cieluv_bytes(input: &[u8]) -> Result<XColor, ParseError> {
    let (l_star, u_star, v_star) = parse_three_floats_bytes(input, 7)?;
    Ok(XColor::CieLuv {
        l_star,
        u_star,
        v_star,
    })
}

fn parse_tekhvc_bytes(input: &[u8]) -> Result<XColor, ParseError> {
    let (h, v, c) = parse_three_floats_bytes(input, 7)?;
    Ok(XColor::TekHvc { h, v, c })
}

/// Parse a named color from bytes.
fn parse_named_bytes(input: &[u8]) -> Result<XColor, ParseError> {
    // Try direct lookup first (no allocation)
    if let Some((r, g, b)) = named_colors::lookup_normalized(input) {
        return Ok(XColor::from_rgb8(r, g, b));
    }

    // Normalize (lowercase, remove whitespace) and try again
    let normalized: Vec<u8> = input
        .iter()
        .filter(|&&b| !b.is_ascii_whitespace())
        .map(|&b| b.to_ascii_lowercase())
        .collect();

    named_colors::lookup_normalized(&normalized)
        .map(|(r, g, b)| XColor::from_rgb8(r, g, b))
        .ok_or(ParseError::UnknownFormat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bytes() {
        // Test byte-based parsing directly
        let color = XColor::try_from_bytes(b"rgb:ffff/8080/0000").unwrap();
        assert!(matches!(
            color,
            XColor::Rgb {
                red: 0xffff,
                green: 0x8080,
                blue: 0
            }
        ));

        let color = XColor::try_from_bytes(b"#ff8000").unwrap();
        assert!(matches!(
            color,
            XColor::Rgb {
                red: 0xff00,
                green: 0x8000,
                blue: 0
            }
        ));

        let color = XColor::try_from_bytes(b"rgbi:1.0/0.5/0.0").unwrap();
        assert!(matches!(color, XColor::RgbIntensity { .. }));

        // Test opportunistic named color lookup (already lowercase, no spaces)
        let color = XColor::try_from_bytes(b"red").unwrap();
        assert!(matches!(
            color,
            XColor::Rgb {
                red: 0xffff,
                green: 0,
                blue: 0
            }
        ));

        // Test named color with normalization needed
        let color = XColor::try_from_bytes(b"Dark Slate Gray").unwrap();
        assert!(matches!(color, XColor::Rgb { .. }));
    }

    #[test]
    fn test_encode_rgb() {
        let color = XColor::rgb(0xffff, 0x8080, 0x0000);
        let encoded = color.encode().unwrap();
        assert_eq!(encoded, b"rgb:ffff/8080/0000");

        // Test round-trip
        let parsed = XColor::try_from_bytes(&encoded).unwrap();
        assert_eq!(color, parsed);
    }

    #[test]
    fn test_encode_into_slice() {
        let color = XColor::rgb(0xffff, 0x8080, 0x0000);
        let mut buf = [0u8; 32];
        let n = color.encode_into_slice(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"rgb:ffff/8080/0000");
    }

    #[test]
    fn test_encode_rgbi() {
        let color = XColor::rgb_intensity(1.0, 0.5, 0.0);
        let encoded = color.encode().unwrap();
        // ryu may format differently, just check prefix and round-trip
        assert!(encoded.starts_with(b"rgbi:"));
        let parsed = XColor::try_from_bytes(&encoded).unwrap();
        if let XColor::RgbIntensity { red, green, blue } = parsed {
            assert!((red - 1.0).abs() < 0.001);
            assert!((green - 0.5).abs() < 0.001);
            assert!((blue - 0.0).abs() < 0.001);
        } else {
            panic!("expected RgbIntensity");
        }
    }

    #[test]
    fn test_encode_ciexyz() {
        let color = XColor::cie_xyz(0.5, 0.5, 0.5);
        let encoded = color.encode().unwrap();
        assert!(encoded.starts_with(b"CIEXYZ:"));
    }

    #[test]
    fn test_encode_buffer_overflow() {
        let color = XColor::rgb(0xffff, 0x8080, 0x0000);
        // "rgb:ffff/8080/0000" is 18 bytes
        let mut buf = [0u8; 10]; // Too small
        let result = color.encode_into_slice(&mut buf);
        assert!(matches!(result, Err(EncodeError::BufferOverflow(_))));

        // Exact size should work
        let mut buf = [0u8; 18];
        let n = color.encode_into_slice(&mut buf).unwrap();
        assert_eq!(n, 18);
        assert_eq!(&buf[..n], b"rgb:ffff/8080/0000");
    }

    #[test]
    fn test_named_color_simple() {
        let color: XColor = "red".parse().unwrap();
        assert!(matches!(
            color,
            XColor::Rgb {
                red: 0xffff,
                green: 0,
                blue: 0
            }
        ));

        let color: XColor = "green".parse().unwrap();
        assert!(matches!(
            color,
            XColor::Rgb {
                red: 0,
                green: 0xffff,
                blue: 0
            }
        ));

        let color: XColor = "blue".parse().unwrap();
        assert!(matches!(
            color,
            XColor::Rgb {
                red: 0,
                green: 0,
                blue: 0xffff
            }
        ));

        let color: XColor = "white".parse().unwrap();
        assert!(matches!(
            color,
            XColor::Rgb {
                red: 0xffff,
                green: 0xffff,
                blue: 0xffff
            }
        ));

        let color: XColor = "black".parse().unwrap();
        assert!(matches!(
            color,
            XColor::Rgb {
                red: 0,
                green: 0,
                blue: 0
            }
        ));
    }

    #[test]
    fn test_named_color_case_insensitive() {
        let lower: XColor = "red".parse().unwrap();
        let upper: XColor = "RED".parse().unwrap();
        let mixed: XColor = "ReD".parse().unwrap();
        assert_eq!(lower, upper);
        assert_eq!(lower, mixed);
    }

    #[test]
    fn test_named_color_with_spaces() {
        // With spaces
        let with_spaces: XColor = "dark slate gray".parse().unwrap();
        // Without spaces
        let no_spaces: XColor = "darkslategray".parse().unwrap();
        // CamelCase
        let camel: XColor = "DarkSlateGray".parse().unwrap();

        assert_eq!(with_spaces, no_spaces);
        assert_eq!(with_spaces, camel);
    }

    #[test]
    fn test_named_color_lookup_function() {
        let result = lookup_named_color("snow");
        assert_eq!(result, Some((255, 250, 250)));

        let result = lookup_named_color("SNOW");
        assert_eq!(result, Some((255, 250, 250)));

        let result = lookup_named_color("notacolor");
        assert_eq!(result, None);
    }

    #[test]
    fn test_named_color_count() {
        // Should have a reasonable number of colors from rgb.txt
        const { assert!(NAMED_COLOR_COUNT > 100) };
        const { assert!(NAMED_COLOR_COUNT < 1000) };
    }

    #[test]
    fn test_rgb_device_format() {
        let color: XColor = "rgb:ffff/8080/0000".parse().unwrap();
        assert!(matches!(
            color,
            XColor::Rgb {
                red: 0xffff,
                green: 0x8080,
                blue: 0
            }
        ));
    }

    #[test]
    fn test_sharp_syntax() {
        // 6-digit hex
        let color: XColor = "#ff8000".parse().unwrap();
        assert!(matches!(
            color,
            XColor::Rgb {
                red: 0xff00,
                green: 0x8000,
                blue: 0
            }
        ));

        // 3-digit hex
        let color: XColor = "#f80".parse().unwrap();
        assert!(matches!(
            color,
            XColor::Rgb {
                red: 0xf000,
                green: 0x8000,
                blue: 0
            }
        ));
    }

    #[test]
    fn test_rgbi_format() {
        let color: XColor = "rgbi:1.0/0.5/0.0".parse().unwrap();
        assert!(matches!(color, XColor::RgbIntensity { red, green, blue }
            if (red - 1.0).abs() < 0.001
            && (green - 0.5).abs() < 0.001
            && (blue - 0.0).abs() < 0.001
        ));
    }

    #[test]
    fn test_unknown_color_error() {
        let result: Result<XColor, _> = "not_a_valid_color_name".parse();
        assert!(matches!(result, Err(XColorParseError::UnknownFormat(_))));
    }

    #[test]
    fn test_display_rgb() {
        let color = XColor::rgb(0xffff, 0x8080, 0x0000);
        assert_eq!(color.to_string(), "rgb:ffff/8080/0000");
    }

    #[test]
    fn test_to_rgb8() {
        let color = XColor::rgb(0xffff, 0x8080, 0x0000);
        assert_eq!(color.to_rgb8(), (0xff, 0x80, 0x00));
    }

    #[test]
    fn test_from_rgb8() {
        let color = XColor::from_rgb8(255, 128, 0);
        assert!(matches!(
            color,
            XColor::Rgb {
                red: 0xffff,
                green: 0x8080,
                blue: 0
            }
        ));
    }

    #[test]
    fn test_palette_conversions() {
        let color = XColor::rgb(0xffff, 0x8080, 0x0000);

        // Test to_srgb
        let srgb = color.to_srgb();
        assert!((srgb.red - 1.0).abs() < 0.001);
        assert!((srgb.green - 0.502).abs() < 0.01);
        assert!((srgb.blue - 0.0).abs() < 0.001);

        // Test round-trip through palette
        let back = XColor::from_srgb(srgb);
        assert_eq!(color, back);
    }

    #[test]
    fn test_cielab_to_rgb() {
        // Test CIE L*a*b* conversion using palette
        let lab_color = XColor::CieLab {
            l_star: 50.0,
            a_star: 0.0,
            b_star: 0.0,
        };
        let rgb = lab_color.to_rgb();
        assert!(matches!(rgb, XColor::Rgb { .. }));
    }

    #[test]
    fn test_cieluv_to_rgb() {
        // Test CIE L*u*v* conversion using palette
        let luv_color = XColor::CieLuv {
            l_star: 50.0,
            u_star: 0.0,
            v_star: 0.0,
        };
        let rgb = luv_color.to_rgb();
        assert!(matches!(rgb, XColor::Rgb { .. }));
    }
}
