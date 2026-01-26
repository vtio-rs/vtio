//! Common types shared across event modules.
//!
//! # Coordinate Systems
//!
//! There are two coordinate conventions used in terminal sequences:
//!
//! - **Screen position**: uses `(row, col)` order, matching how terminals
//!   address cells. Used in cursor positioning, rectangular operations, etc.
//!
//! - **Mouse coordinates**: uses `(col, row)` order, matching the X/Y
//!   convention common in mouse protocols.
//!
//! Both use 1-based indexing as required by terminal protocols.

/// A screen position defined by row and column.
///
/// All coordinates are 1-based.
///
/// This type uses `(row, col)` order, matching the convention used in most
/// terminal control sequences for cursor positioning and rectangular
/// operations.
///
/// # Fields
///
/// - `row`: Row number (1-based)
/// - `col`: Column number (1-based)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
)]
#[vtansi(format = "vector")]
pub struct Coords {
    /// Row number (1-based).
    pub row: u16,
    /// Column number (1-based).
    pub col: u16,
}

impl Coords {
    /// Create a new position.
    ///
    /// # Arguments
    ///
    /// * `row` - Row number (1-based)
    /// * `col` - Column number (1-based)
    #[must_use]
    pub const fn new(row: u16, col: u16) -> Self {
        Self { row, col }
    }
}

/// A rectangular area defined by top, left, bottom, and right coordinates.
///
/// All coordinates are 1-based line/column numbers.
///
/// This type can be used with `#[vtansi(flatten)]` to embed rectangular
/// coordinates directly in CSI sequences, or with `#[vtansi(format = "vector")]`
/// for automatic derive-based encoding/decoding.
///
/// # Fields
///
/// - `top`: Top line of the rectangle (1-based)
/// - `left`: Left column of the rectangle (1-based)
/// - `bottom`: Bottom line of the rectangle (1-based)
/// - `right`: Right column of the rectangle (1-based)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    vtansi::derive::ToAnsi,
    vtansi::derive::FromAnsi,
)]
#[vtansi(format = "vector")]
pub struct Rect {
    /// Top line of the rectangle (1-based).
    pub top: u16,
    /// Left column of the rectangle (1-based).
    pub left: u16,
    /// Bottom line of the rectangle (1-based).
    pub bottom: u16,
    /// Right column of the rectangle (1-based).
    pub right: u16,
}

impl Rect {
    /// Create a new rectangular area.
    ///
    /// # Arguments
    ///
    /// * `top` - Top line (1-based)
    /// * `left` - Left column (1-based)
    /// * `bottom` - Bottom line (1-based)
    /// * `right` - Right column (1-based)
    #[must_use]
    pub const fn new(top: u16, left: u16, bottom: u16, right: u16) -> Self {
        Self {
            top,
            left,
            bottom,
            right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::AnsiEncode;

    #[test]
    fn test_position_new() {
        let pos = Coords::new(10, 20);
        assert_eq!(pos.row, 10);
        assert_eq!(pos.col, 20);
    }

    #[test]
    fn test_position_encoding() {
        let pos = Coords::new(5, 10);
        assert_eq!(pos.encode_ansi().unwrap(), b"5;10");
    }

    #[test]
    fn test_rectangular_area_new() {
        let area = Rect::new(1, 2, 10, 20);
        assert_eq!(area.top, 1);
        assert_eq!(area.left, 2);
        assert_eq!(area.bottom, 10);
        assert_eq!(area.right, 20);
    }

    #[test]
    fn test_rectangular_area_encoding() {
        let area = Rect::new(1, 1, 24, 80);
        assert_eq!(area.encode_ansi().unwrap(), b"1;1;24;80");
    }
}
