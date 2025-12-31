//! SAS missing value types.
//!
//! SAS Transport format supports 28 different missing value codes:
//! - Standard missing (`.`) - byte `0x2e`
//! - Underscore missing (`._`) - byte `0x5f`
//! - Special missing (`.A` through `.Z`) - bytes `0x41` through `0x5a`

use std::fmt;

/// SAS missing value types.
///
/// In SAS, missing values are represented differently than in most systems.
/// A missing numeric value has a special first byte followed by zeros.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MissingValue {
    /// Standard missing value (`.`)
    ///
    /// Encoded as `0x2e` followed by 7 zero bytes.
    Standard,

    /// Underscore missing value (`._`)
    ///
    /// Encoded as `0x5f` followed by 7 zero bytes.
    Underscore,

    /// Special missing value `.A` through `.Z`
    ///
    /// Encoded as the ASCII value of the letter (`0x41`-`0x5a`) followed by 7 zero bytes.
    Special(char),
}

impl MissingValue {
    /// Convert the missing value to its XPT first-byte representation.
    ///
    /// # Returns
    /// The byte value that identifies this missing type.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Standard => 0x2e,   // '.'
            Self::Underscore => 0x5f, // '_'
            Self::Special(c) => c as u8,
        }
    }

    /// Create a missing value from an XPT first byte.
    ///
    /// Only valid if the remaining 7 bytes are zeros.
    ///
    /// # Arguments
    /// * `byte` - The first byte of an 8-byte numeric value
    ///
    /// # Returns
    /// `Some(MissingValue)` if the byte represents a valid missing code,
    /// `None` otherwise.
    #[must_use]
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x2e => Some(Self::Standard),
            0x5f => Some(Self::Underscore),
            0x41..=0x5a => Some(Self::Special(byte as char)),
            _ => None,
        }
    }

    /// Check if a byte sequence represents a missing value.
    ///
    /// A missing value is recognized by:
    /// 1. First byte is a valid missing indicator
    /// 2. All subsequent bytes are zero
    ///
    /// # Arguments
    /// * `bytes` - The byte slice to check (typically 8 bytes for numeric)
    ///
    /// # Returns
    /// `Some(MissingValue)` if this represents a missing value, `None` otherwise.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        // Check if all bytes after the first are zero
        if !bytes.iter().skip(1).all(|&b| b == 0) {
            return None;
        }

        Self::from_byte(bytes[0])
    }

    /// Encode this missing value as an 8-byte array.
    ///
    /// # Returns
    /// An 8-byte array with the missing indicator in the first byte
    /// and zeros in the remaining 7 bytes.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[0] = self.to_byte();
        bytes
    }

    /// Get all 28 possible missing value codes.
    ///
    /// # Returns
    /// An array of all valid missing values: `.`, `._`, `.A` through `.Z`.
    #[must_use]
    pub fn all_codes() -> [Self; 28] {
        let mut codes = [Self::Standard; 28];
        codes[0] = Self::Standard;
        codes[1] = Self::Underscore;
        for (i, c) in ('A'..='Z').enumerate() {
            codes[i + 2] = Self::Special(c);
        }
        codes
    }

    /// Check if this is the standard missing value (`.`).
    #[must_use]
    pub const fn is_standard(self) -> bool {
        matches!(self, Self::Standard)
    }

    /// Check if this is a special missing value (`.A` through `.Z`).
    #[must_use]
    pub const fn is_special(self) -> bool {
        matches!(self, Self::Special(_))
    }

    /// Get the letter for a special missing value.
    ///
    /// # Returns
    /// `Some(char)` for special missing values `.A` through `.Z`,
    /// `None` for standard and underscore missing.
    #[must_use]
    pub const fn letter(self) -> Option<char> {
        match self {
            Self::Special(c) => Some(c),
            _ => None,
        }
    }
}

impl Default for MissingValue {
    fn default() -> Self {
        Self::Standard
    }
}

impl fmt::Display for MissingValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard => write!(f, "."),
            Self::Underscore => write!(f, "._"),
            Self::Special(c) => write!(f, ".{c}"),
        }
    }
}

impl TryFrom<char> for MissingValue {
    type Error = ();

    /// Create a special missing value from a letter.
    ///
    /// # Arguments
    /// * `c` - A character 'A' through 'Z' (case insensitive)
    ///
    /// # Returns
    /// `Ok(MissingValue::Special(c))` for valid letters,
    /// `Err(())` otherwise.
    fn try_from(c: char) -> Result<Self, Self::Error> {
        let upper = c.to_ascii_uppercase();
        if upper.is_ascii_uppercase() {
            Ok(Self::Special(upper))
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_missing() {
        let mv = MissingValue::Standard;
        assert_eq!(mv.to_byte(), 0x2e);
        assert!(mv.is_standard());
        assert!(!mv.is_special());
        assert_eq!(format!("{mv}"), ".");
    }

    #[test]
    fn test_underscore_missing() {
        let mv = MissingValue::Underscore;
        assert_eq!(mv.to_byte(), 0x5f);
        assert!(!mv.is_standard());
        assert!(!mv.is_special());
        assert_eq!(format!("{mv}"), "._");
    }

    #[test]
    fn test_special_missing() {
        let mv = MissingValue::Special('A');
        assert_eq!(mv.to_byte(), 0x41);
        assert!(!mv.is_standard());
        assert!(mv.is_special());
        assert_eq!(mv.letter(), Some('A'));
        assert_eq!(format!("{mv}"), ".A");

        let mv = MissingValue::Special('Z');
        assert_eq!(mv.to_byte(), 0x5a);
        assert_eq!(format!("{mv}"), ".Z");
    }

    #[test]
    fn test_from_byte() {
        assert_eq!(MissingValue::from_byte(0x2e), Some(MissingValue::Standard));
        assert_eq!(
            MissingValue::from_byte(0x5f),
            Some(MissingValue::Underscore)
        );
        assert_eq!(
            MissingValue::from_byte(0x41),
            Some(MissingValue::Special('A'))
        );
        assert_eq!(
            MissingValue::from_byte(0x5a),
            Some(MissingValue::Special('Z'))
        );
        assert_eq!(MissingValue::from_byte(0x00), None);
        assert_eq!(MissingValue::from_byte(0x40), None);
        assert_eq!(MissingValue::from_byte(0x5b), None);
    }

    #[test]
    fn test_from_bytes() {
        // Standard missing
        let bytes = [0x2e, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(
            MissingValue::from_bytes(&bytes),
            Some(MissingValue::Standard)
        );

        // Special missing .A
        let bytes = [0x41, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(
            MissingValue::from_bytes(&bytes),
            Some(MissingValue::Special('A'))
        );

        // Not missing - non-zero trailing bytes
        let bytes = [0x2e, 0, 0, 0, 0, 0, 0, 1];
        assert_eq!(MissingValue::from_bytes(&bytes), None);

        // Not missing - invalid first byte
        let bytes = [0x00, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(MissingValue::from_bytes(&bytes), None);

        // Empty bytes
        assert_eq!(MissingValue::from_bytes(&[]), None);
    }

    #[test]
    fn test_to_bytes() {
        let bytes = MissingValue::Standard.to_bytes();
        assert_eq!(bytes, [0x2e, 0, 0, 0, 0, 0, 0, 0]);

        let bytes = MissingValue::Underscore.to_bytes();
        assert_eq!(bytes, [0x5f, 0, 0, 0, 0, 0, 0, 0]);

        let bytes = MissingValue::Special('M').to_bytes();
        assert_eq!(bytes, [0x4d, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_all_codes() {
        let codes = MissingValue::all_codes();
        assert_eq!(codes.len(), 28);
        assert_eq!(codes[0], MissingValue::Standard);
        assert_eq!(codes[1], MissingValue::Underscore);
        assert_eq!(codes[2], MissingValue::Special('A'));
        assert_eq!(codes[27], MissingValue::Special('Z'));
    }

    #[test]
    fn test_try_from_char() {
        assert_eq!(MissingValue::try_from('A'), Ok(MissingValue::Special('A')));
        assert_eq!(MissingValue::try_from('z'), Ok(MissingValue::Special('Z')));
        assert!(MissingValue::try_from('1').is_err());
        assert!(MissingValue::try_from('.').is_err());
    }

    #[test]
    fn test_default() {
        assert_eq!(MissingValue::default(), MissingValue::Standard);
    }

    #[test]
    fn test_roundtrip() {
        for code in MissingValue::all_codes() {
            let bytes = code.to_bytes();
            let recovered = MissingValue::from_bytes(&bytes).unwrap();
            assert_eq!(code, recovered);
        }
    }
}
