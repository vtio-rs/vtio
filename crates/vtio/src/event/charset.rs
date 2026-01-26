//! Terminal character set control and information messages.

/// Enable UTF-8 mode.
///
/// *Sequence*: `ESC % G`
///
/// Set character set to UTF-8.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_zpercent_cg/> for
/// terminal support specifics and details.
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
#[vtansi(esc, finalbyte = 'G', intermediate = "%")]
pub struct EnableUTF8Mode;

/// Disable UTF-8 mode.
///
/// *Sequence*: `ESC % @`
///
/// See <https://terminalguide.namepad.de/seq/a_esc_zpercent_x40_at/> for
/// terminal support specifics and details.
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
#[vtansi(esc, finalbyte = '@', intermediate = "%")]
pub struct DisableUTF8Mode;

/// Shift Out (SO).
///
/// *Sequence*: `0x0E` (C0 control code)
///
/// Invoke G1 character set into GL (left half of character table).
/// Maps G1 character set into the left (GL) character positions.
///
/// See <https://terminalguide.namepad.de/seq/a_c0-n/> for terminal support
/// specifics and details.
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
#[vtansi(c0, code = 0x0E)]
pub struct ShiftOut;

/// Shift In (SI).
///
/// *Sequence*: `0x0F` (C0 control code)
///
/// Invoke G0 character set into GL (left half of character table).
/// Maps G0 character set into the left (GL) character positions.
///
/// See <https://terminalguide.namepad.de/seq/a_c0-o/> for terminal support
/// specifics and details.
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
#[vtansi(c0, code = 0x0F)]
pub struct ShiftIn;

/// Locking Shift 2 (LS2).
///
/// *Sequence*: `ESC n`
///
/// Invoke G2 character set into GL (left half of character table).
///
/// See <https://terminalguide.namepad.de/seq/a_esc_sn/> for terminal support
/// specifics and details.
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
#[vtansi(esc, finalbyte = 'n')]
pub struct LockingShift2;

/// Locking Shift 3 (LS3).
///
/// *Sequence*: `ESC o`
///
/// Invoke G3 character set into GL (left half of character table).
///
/// See <https://terminalguide.namepad.de/seq/a_esc_so/> for terminal support
/// specifics and details.
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
#[vtansi(esc, finalbyte = 'o')]
pub struct LockingShift3;

/// Locking Shift 1 Right (LS1R).
///
/// *Sequence*: `ESC ~`
///
/// Invoke G1 character set into GR (right half of character table).
///
/// See <https://terminalguide.namepad.de/seq/a_esc_x7e_tilde/> for terminal
/// support specifics and details.
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
#[vtansi(esc, finalbyte = '~')]
pub struct LockingShift1Right;

/// Locking Shift 2 Right (LS2R).
///
/// *Sequence*: `ESC }`
///
/// Invoke G2 character set into GR (right half of character table).
///
/// See <https://terminalguide.namepad.de/seq/a_esc_x7d_right_brace/> for
/// terminal support specifics and details.
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
#[vtansi(esc, finalbyte = '}')]
pub struct LockingShift2Right;

/// Locking Shift 3 Right (LS3R).
///
/// *Sequence*: `ESC |`
///
/// Invoke G3 character set into GR (right half of character table).
///
/// See <https://terminalguide.namepad.de/seq/a_esc_x7c_pipe/> for terminal
/// support specifics and details.
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
#[vtansi(esc, finalbyte = '|')]
pub struct LockingShift3Right;

/// Single Shift 2 (SS2).
///
/// *Sequence*: `ESC N`
///
/// Temporarily invoke G2 character set for the next character only.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_cn/> for terminal
/// support specifics and details.
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
#[vtansi(esc, finalbyte = 'N')]
pub struct SingleShift2;

/// Single Shift 3 (SS3).
///
/// *Sequence*: `ESC O`
///
/// Temporarily invoke G3 character set for the next character only.
///
/// See <https://terminalguide.namepad.de/seq/a_esc_co/> for terminal
/// support specifics and details.
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
#[vtansi(esc, finalbyte = 'O')]
pub struct SingleShift3;

/// Character set codes used in charset designation sequences
/// (94 character variant).
///
/// These codes identify specific character sets that can be designated to
/// G0, G1, G2, or G3 charset registers.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    strum::EnumString,
    strum::IntoStaticStr,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[vtansi(encoded_len = 2)]
pub enum Charset94Code {
    /// ASCII character set.
    #[strum(serialize = "B")]
    Ascii,
    /// British character set.
    #[strum(serialize = "A")]
    British,
    /// DEC Special Character and Line Drawing Set.
    #[strum(serialize = "0")]
    DecSpecialGraphic,
    /// DEC Alternate Character Set.
    #[strum(serialize = "1")]
    DecAltChars,
    /// DEC Alternate Graphics.
    #[strum(serialize = "2")]
    DecAltGraphics,
    /// DEC Supplemental character set.
    #[strum(serialize = "<")]
    DecSupp,
    /// Dutch character set.
    #[strum(serialize = "4")]
    Dutch,
    /// Finnish character set.
    #[strum(serialize = "5")]
    Finnish,
    /// Finnish character set (variant 2).
    #[strum(serialize = "C")]
    Finnish2,
    /// French character set.
    #[strum(serialize = "R")]
    French,
    /// French character set (variant 2).
    #[strum(serialize = "f")]
    French2,
    /// French Canadian character set.
    #[strum(serialize = "Q")]
    FrenchCanadian,
    /// French Canadian character set (variant 2).
    #[strum(serialize = "9")]
    FrenchCanadian2,
    /// German character set.
    #[strum(serialize = "K")]
    German,
    /// Italian character set.
    #[strum(serialize = "Y")]
    Italian,
    /// Norwegian/Danish character set.
    #[strum(serialize = "`")]
    NorwegianDanish,
    /// Norwegian/Danish character set (variant 2).
    #[strum(serialize = "E")]
    NorwegianDanish2,
    /// Norwegian/Danish character set (variant 3).
    #[strum(serialize = "6")]
    NorwegianDanish3,
    /// Spanish character set.
    #[strum(serialize = "Z")]
    Spanish,
    /// Swedish character set.
    #[strum(serialize = "7")]
    Swedish,
    /// Swedish character set (variant 2).
    #[strum(serialize = "H")]
    Swedish2,
    /// Swiss character set.
    #[strum(serialize = "=")]
    Swiss,
    /// DEC Technical character set.
    #[strum(serialize = ">")]
    DecTechnical,
    /// DEC Supplemental Graphic character set.
    #[strum(serialize = "%5")]
    DecSuppGraphic,
    /// Portuguese character set.
    #[strum(serialize = "%6")]
    Portuguese,
    /// Turkish character set.
    #[strum(serialize = "%0")]
    Turkish,
    /// Turkish Supplement character set.
    #[strum(serialize = "%2")]
    TurkishSupplement,
    /// Hebrew character set.
    #[strum(serialize = "%=")]
    Hebrew,
    /// DEC Hebrew Supplement character set.
    #[strum(serialize = "\"4")]
    DecHebrewSupplement,
    /// Greek character set.
    #[strum(serialize = "\">")]
    Greek,
    /// DEC Greek Supplement character set.
    #[strum(serialize = "\"?")]
    DecGreekSupplement,
    /// IBM code page 437 (Linux console only).
    #[strum(serialize = "U")]
    Cp437,
}

/// Character set codes used in charset designation sequences
/// (96 character variant).
///
/// These codes identify specific character sets that can be designated to
/// G0, G1, G2, or G3 charset registers.
#[derive(
    Debug,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    strum::EnumString,
    strum::IntoStaticStr,
    vtansi::derive::FromAnsi,
    vtansi::derive::ToAnsi,
)]
#[vtansi(encoded_len = 2)]
pub enum Charset96Code {
    /// Latin-1 Supplemental (96-character set).
    #[strum(serialize = "A")]
    Latin1Supplemental,
    /// Greek (bottom part of ISO-8859-7, 96-character set).
    #[strum(serialize = "F")]
    GreekSupplemental,
    /// Hebrew (bottom part of ISO-8859-8, 96-character set).
    #[strum(serialize = "H")]
    HebrewSupplemental,
    /// Latin-Cyrillic (bottom part of ISO-8859-5, 96-character set).
    #[strum(serialize = "L")]
    LatinCyrillic,
    /// Latin-5 (bottom part of ISO-8859-9, 96-character set).
    #[strum(serialize = "M")]
    Latin5,
}

/// Designate G0 Character Set (94 characters).
///
/// *Sequence*: `ESC ( Pc`
///
/// Designate a 94-character set to the G0 character set register.
/// This is part of the ISO-2022 character set designation mechanism.
///
/// See <https://terminalguide.namepad.de/seq/> charset designation section
/// for terminal support specifics and details.
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
#[vtansi(esc, intermediate = "(")]
pub struct DesignateG0 {
    #[vtansi(locate = "final")]
    pub charset: Charset94Code,
}

/// Designate G1 Character Set (94 characters).
///
/// *Sequence*: `ESC ) Pc`
///
/// Designate a 94-character set to the G1 character set register.
/// This is part of the ISO-2022 character set designation mechanism.
///
/// See <https://terminalguide.namepad.de/seq/> charset designation section
/// for terminal support specifics and details.
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
#[vtansi(esc, intermediate = ")")]
pub struct DesignateG1 {
    #[vtansi(locate = "final")]
    pub charset: Charset94Code,
}

/// Designate G2 Character Set (94 characters).
///
/// *Sequence*: `ESC * Pc`
///
/// Designate a 94-character set to the G2 character set register.
/// This is part of the ISO-2022 character set designation mechanism.
///
/// See <https://terminalguide.namepad.de/seq/> charset designation section
/// for terminal support specifics and details.
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
#[vtansi(esc, intermediate = "*")]
pub struct DesignateG2 {
    #[vtansi(locate = "final")]
    pub charset: Charset94Code,
}

/// Designate G3 Character Set (94 characters).
///
/// *Sequence*: `ESC + Pc`
///
/// Designate a 94-character set to the G3 character set register.
/// This is part of the ISO-2022 character set designation mechanism.
///
/// See <https://terminalguide.namepad.de/seq/> charset designation section
/// for terminal support specifics and details.
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
#[vtansi(esc, intermediate = "+")]
pub struct DesignateG3 {
    #[vtansi(locate = "final")]
    pub charset: Charset94Code,
}

/// Designate G1 Character Set (96 characters).
///
/// *Sequence*: `ESC - Pc`
///
/// Designate a 96-character set to the G1 character set register.
/// This is part of the ISO-2022 character set designation mechanism.
///
/// See <https://terminalguide.namepad.de/seq/> charset designation section
/// for terminal support specifics and details.
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
#[vtansi(esc, intermediate = "-")]
pub struct DesignateG1_96 {
    #[vtansi(locate = "final")]
    pub charset: Charset96Code,
}

/// Designate G2 Character Set (96 characters).
///
/// *Sequence*: `ESC . Pc`
///
/// Designate a 96-character set to the G2 character set register.
/// This is part of the ISO-2022 character set designation mechanism.
///
/// See <https://terminalguide.namepad.de/seq/> charset designation section
/// for terminal support specifics and details.
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
#[vtansi(esc, intermediate = ".")]
pub struct DesignateG2_96 {
    #[vtansi(locate = "final")]
    pub charset: Charset96Code,
}

/// Designate G3 Character Set (96 characters).
///
/// *Sequence*: `ESC / Pc`
///
/// Designate a 96-character set to the G3 character set register.
/// This is part of the ISO-2022 character set designation mechanism.
///
/// See <https://terminalguide.namepad.de/seq/> charset designation section
/// for terminal support specifics and details.
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
#[vtansi(esc, intermediate = "/")]
pub struct DesignateG3_96 {
    #[vtansi(locate = "final")]
    pub charset: Charset96Code,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtansi::{AnsiEncode, StaticAnsiEncode};

    #[test]
    fn test_c0_control_codes() {
        assert_eq!(ShiftOut::BYTES, b"\x0E");
        assert_eq!(ShiftIn::BYTES, b"\x0F");
    }

    #[test]
    fn test_const_esc_sequences() {
        assert_eq!(EnableUTF8Mode::BYTES, b"\x1B%G");
        assert_eq!(DisableUTF8Mode::BYTES, b"\x1B%@");
        assert_eq!(LockingShift2::BYTES, b"\x1Bn");
        assert_eq!(LockingShift3::BYTES, b"\x1Bo");
        assert_eq!(SingleShift2::BYTES, b"\x1BN");
        assert_eq!(SingleShift3::BYTES, b"\x1BO");
        assert_eq!(LockingShift1Right::BYTES, b"\x1B~");
        assert_eq!(LockingShift2Right::BYTES, b"\x1B}");
        assert_eq!(LockingShift3Right::BYTES, b"\x1B|");
    }

    #[test]
    fn test_variable_esc_sequences() {
        let mut buf = Vec::new();
        let msg = DesignateG0 {
            charset: Charset94Code::Ascii,
        };
        msg.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1B(B");

        let mut buf = Vec::new();
        let msg = DesignateG1 {
            charset: Charset94Code::DecSpecialGraphic,
        };
        msg.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1B)0");

        let mut buf = Vec::new();
        let msg = DesignateG0 {
            charset: Charset94Code::DecSuppGraphic,
        };
        msg.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1B(%5");

        let mut buf = Vec::new();
        let msg = DesignateG2_96 {
            charset: Charset96Code::Latin1Supplemental,
        };
        msg.encode_ansi_into(&mut buf).unwrap();
        assert_eq!(buf, b"\x1B.A");
    }
}
