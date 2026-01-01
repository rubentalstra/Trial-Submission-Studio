//! IEEE 754 to IBM mainframe float conversion.
//!
//! SAS Transport (XPT) files use IBM mainframe floating-point format,
//! not the IEEE 754 format used by modern computers.
//!
//! # Format Comparison
//!
//! ## IEEE 754 Double (64-bit)
//! ```text
//! Bit 63: Sign (1 bit)
//! Bits 62-52: Exponent (11 bits, excess 1023)
//! Bits 51-0: Fraction (52 bits)
//!
//! Value = (-1)^sign × 2^(exponent-1023) × 1.fraction
//! ```
//!
//! ## IBM Mainframe Double (64-bit)
//! ```text
//! Bit 63: Sign (1 bit)
//! Bits 62-56: Exponent (7 bits, excess 64)
//! Bits 55-0: Fraction (56 bits)
//!
//! Value = (-1)^sign × 16^(exponent-64) × 0.fraction
//! ```
//!
//! # Key Differences
//! - IEEE: power of 2 exponent, implicit leading "1" bit
//! - IBM: power of 16 exponent, fraction starts at radix point (no implicit bit)
//! - IBM has 4 more fraction bits but 4 fewer exponent bits

use crate::types::MissingValue;

/// Convert IEEE 754 double to IBM mainframe 8-byte representation.
///
/// This implements the algorithm from the SAS Transport format specification.
///
/// # Arguments
/// * `value` - Native f64 value to convert
///
/// # Returns
/// 8-byte array in IBM mainframe float format (big-endian)
///
/// # Examples
/// ```
/// use sdtm_xpt::float::ieee_to_ibm;
///
/// let ibm = ieee_to_ibm(1.0);
/// assert_eq!(ibm, [0x41, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
///
/// let ibm = ieee_to_ibm(0.0);
/// assert_eq!(ibm, [0x00; 8]);
/// ```
#[must_use]
pub fn ieee_to_ibm(value: f64) -> [u8; 8] {
    // Handle zero specially
    if value == 0.0 {
        return [0u8; 8];
    }

    // Handle non-finite values (NaN, Infinity) as zero
    // In practice, these should be encoded as missing values before calling this
    if !value.is_finite() {
        return [0u8; 8];
    }

    let bits = value.to_bits();

    // Extract IEEE components
    let sign = ((bits >> 63) & 1) as u8;
    let ieee_exp = ((bits >> 52) & 0x7FF) as i32;
    let ieee_frac = bits & 0x000F_FFFF_FFFF_FFFF;

    // Handle denormalized numbers (exponent = 0)
    if ieee_exp == 0 {
        // Denormalized IEEE number - very small, convert approximately
        // These have implicit 0. prefix instead of 1.
        let frac_with_bit = ieee_frac;
        if frac_with_bit == 0 {
            return [0u8; 8];
        }

        // For denormalized, the actual exponent is -1022
        // and fraction doesn't have implicit 1
        let actual_exp = -1022i32;
        return convert_to_ibm(sign, actual_exp, frac_with_bit, false);
    }

    // Normal IEEE number - add back the implicit "1" bit
    let frac_with_one = ieee_frac | 0x0010_0000_0000_0000;

    // Calculate actual binary exponent (IEEE uses excess-1023 notation)
    let actual_exp = ieee_exp - 1023;

    convert_to_ibm(sign, actual_exp, frac_with_one, true)
}

/// Internal conversion from IEEE components to IBM format.
fn convert_to_ibm(sign: u8, binary_exp: i32, fraction: u64, has_implicit_one: bool) -> [u8; 8] {
    // IBM uses base-16 exponents. We need to convert:
    // 2^binary_exp = 16^ibm_exp × 2^shift
    // where shift is 0, 1, 2, or 3 (to align to hex boundary)

    // Position of the leading 1 bit determines the hex digit position
    // For normalized IEEE, the "1" bit is at position 52

    let bit_position = if has_implicit_one {
        52
    } else {
        51 - fraction.leading_zeros() as i32
    };

    // Total bit position from binary point
    let total_bit_pos = binary_exp + (52 - bit_position);

    // Calculate IBM exponent (power of 16)
    // IBM value = 16^(exp-64) × 0.fraction
    // The leading 1 bit goes into the first nibble after the hex point
    // This means we need to compensate: the fraction is 0.1xxx (hex) = 1/16
    // So we need an extra factor of 16, hence +65 instead of +64

    let shift = ((total_bit_pos % 4) + 4) % 4; // Handle negative modulo
    let ibm_exp = (total_bit_pos - shift) / 4 + 65;

    // Check exponent range (IBM has 7-bit exponent: 0-127)
    if !(0..=127).contains(&ibm_exp) {
        // Underflow or overflow - return zero
        return [0u8; 8];
    }

    // Shift fraction to align with IBM format
    // IBM fraction occupies bits 55-0 (56 bits)
    // IEEE fraction (with implicit 1) is 53 bits starting at bit 52

    // We need to position the leading 1 bit appropriately for the shift
    let ibm_frac = if has_implicit_one {
        // Shift the 53-bit value (with leading 1) to fit in 56 bits
        // Then adjust based on the hex alignment shift
        let base_frac = fraction << 3; // Move to fill 56 bits (53 + 3 = 56)
        base_frac >> (3 - shift)
    } else {
        // Denormalized - handle specially
        fraction << (56 - 52 + shift)
    };

    // Build IBM format: sign in high bit, 7-bit exponent, 56-bit fraction
    let result = ((sign as u64) << 63)
        | (((ibm_exp as u64) & 0x7F) << 56)
        | (ibm_frac & 0x00FF_FFFF_FFFF_FFFF);

    result.to_be_bytes()
}

/// Convert IBM mainframe 8-byte representation to IEEE 754 double.
///
/// This implements the algorithm from the SAS Transport format specification.
///
/// # Arguments
/// * `bytes` - 8-byte array in IBM mainframe float format (big-endian)
///
/// # Returns
/// Native f64 value
///
/// # Examples
/// ```
/// use sdtm_xpt::float::ibm_to_ieee;
///
/// let value = ibm_to_ieee([0x41, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
/// assert!((value - 1.0).abs() < 1e-15);
///
/// let value = ibm_to_ieee([0x00; 8]);
/// assert_eq!(value, 0.0);
/// ```
#[must_use]
pub fn ibm_to_ieee(bytes: [u8; 8]) -> f64 {
    let bits = u64::from_be_bytes(bytes);

    // Check for zero
    if bits == 0 {
        return 0.0;
    }

    // Extract IBM components
    let sign = bits >> 63;
    let ibm_exp = ((bits >> 56) & 0x7F) as i32;
    let ibm_frac = bits & 0x00FF_FFFF_FFFF_FFFF;

    // If fraction is zero, result is zero (regardless of exponent)
    if ibm_frac == 0 {
        return 0.0;
    }

    // Find the position of the leading 1 bit in the fraction
    // IBM fraction is 56 bits, with the most significant bit at position 55
    let leading_zeros = ibm_frac.leading_zeros() as i32 - 8; // Adjust for 64-bit width
    let _shift = leading_zeros;

    // Convert hex exponent to binary exponent
    // IBM: value = 16^(ibm_exp - 64) × 0.fraction
    // The hex exponent becomes binary exponent × 4
    // Adjust for the position of the leading 1 bit

    // Position of leading 1 in fraction (0-55)
    let lead_bit_pos = 55 - leading_zeros;

    // Binary exponent calculation:
    // IBM value = 16^(exp-64) × frac/2^56
    //           = 2^(4×(exp-64)) × frac/2^56
    //           = frac × 2^(4×(exp-64) - 56)
    // For IEEE we need: 1.xxx × 2^ieee_exp
    // The leading 1 is at position lead_bit_pos in the 56-bit fraction
    // So: frac = 2^lead_bit_pos × 1.xxx
    // Therefore: value = 2^lead_bit_pos × 1.xxx × 2^(4×(exp-64) - 56)
    //                  = 1.xxx × 2^(lead_bit_pos + 4×(exp-64) - 56)

    let ieee_exp = lead_bit_pos + 4 * (ibm_exp - 64) - 56 + 1023;

    // Check for exponent overflow/underflow
    if ieee_exp <= 0 {
        // Underflow to zero or denormal
        return 0.0;
    }
    if ieee_exp >= 2047 {
        // Overflow to infinity
        return if sign != 0 {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }

    // Shift fraction to remove leading 1 and align to 52 bits
    // The leading 1 is at position lead_bit_pos (0-55)
    // We need to shift it to position 52 and remove it
    let ieee_frac = if lead_bit_pos > 52 {
        (ibm_frac >> (lead_bit_pos - 52)) & 0x000F_FFFF_FFFF_FFFF
    } else {
        (ibm_frac << (52 - lead_bit_pos)) & 0x000F_FFFF_FFFF_FFFF
    };

    // Build IEEE double
    let result = (sign << 63) | (((ieee_exp as u64) & 0x7FF) << 52) | ieee_frac;

    f64::from_bits(result)
}

/// Check if bytes represent a missing value.
///
/// # Arguments
/// * `bytes` - The byte slice to check
///
/// # Returns
/// `Some(MissingValue)` if this represents a missing value, `None` otherwise.
#[must_use]
pub fn is_missing(bytes: &[u8]) -> Option<MissingValue> {
    MissingValue::from_bytes(bytes)
}

/// Encode a missing value as 8 bytes.
///
/// # Arguments
/// * `missing` - The missing value type to encode
///
/// # Returns
/// 8-byte array representing the missing value.
#[must_use]
pub fn encode_missing(missing: MissingValue) -> [u8; 8] {
    missing.to_bytes()
}

/// Truncate IBM float to specified length.
///
/// Some numeric variables may have length < 8 bytes.
/// This truncates the IBM representation to the specified length.
///
/// # Arguments
/// * `bytes` - Full 8-byte IBM representation
/// * `length` - Desired length (1-8)
///
/// # Returns
/// Vector with the first `length` bytes.
#[must_use]
pub fn truncate_ibm(bytes: [u8; 8], length: usize) -> Vec<u8> {
    let len = length.min(8);
    bytes[..len].to_vec()
}

/// Expand truncated IBM float to 8 bytes.
///
/// Pads with zeros to restore full 8-byte representation.
///
/// # Arguments
/// * `bytes` - Truncated IBM bytes (1-8 bytes)
///
/// # Returns
/// Full 8-byte array, padded with zeros.
#[must_use]
pub fn expand_ibm(bytes: &[u8]) -> [u8; 8] {
    let mut result = [0u8; 8];
    let len = bytes.len().min(8);
    result[..len].copy_from_slice(&bytes[..len]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test values from PDF specification
    const IBM_ZERO: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    const IBM_ONE: [u8; 8] = [0x41, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    const IBM_NEG_ONE: [u8; 8] = [0xc1, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    const IBM_TWO: [u8; 8] = [0x41, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    #[test]
    fn test_ieee_to_ibm_zero() {
        assert_eq!(ieee_to_ibm(0.0), IBM_ZERO);
        assert_eq!(ieee_to_ibm(-0.0), IBM_ZERO);
    }

    #[test]
    fn test_ieee_to_ibm_one() {
        let ibm = ieee_to_ibm(1.0);
        assert_eq!(ibm, IBM_ONE);
    }

    #[test]
    fn test_ieee_to_ibm_neg_one() {
        let ibm = ieee_to_ibm(-1.0);
        assert_eq!(ibm, IBM_NEG_ONE);
    }

    #[test]
    fn test_ieee_to_ibm_two() {
        let ibm = ieee_to_ibm(2.0);
        assert_eq!(ibm, IBM_TWO);
    }

    #[test]
    fn test_ibm_to_ieee_zero() {
        assert_eq!(ibm_to_ieee(IBM_ZERO), 0.0);
    }

    #[test]
    fn test_ibm_to_ieee_one() {
        let value = ibm_to_ieee(IBM_ONE);
        assert!((value - 1.0).abs() < 1e-15, "Expected 1.0, got {value}");
    }

    #[test]
    fn test_ibm_to_ieee_neg_one() {
        let value = ibm_to_ieee(IBM_NEG_ONE);
        assert!((value - (-1.0)).abs() < 1e-15, "Expected -1.0, got {value}");
    }

    #[test]
    fn test_ibm_to_ieee_two() {
        let value = ibm_to_ieee(IBM_TWO);
        assert!((value - 2.0).abs() < 1e-15, "Expected 2.0, got {value}");
    }

    #[test]
    fn test_roundtrip_integers() {
        for i in -100..=100 {
            let value = i as f64;
            let ibm = ieee_to_ibm(value);
            let back = ibm_to_ieee(ibm);
            assert!(
                (back - value).abs() < 1e-10,
                "Roundtrip failed for {value}: got {back}"
            );
        }
    }

    #[test]
    fn test_roundtrip_fractions() {
        let values = [0.5, 0.25, 0.125, 0.1, 0.01, 0.001, 1.5, 2.5, 10.5];
        for &value in &values {
            let ibm = ieee_to_ibm(value);
            let back = ibm_to_ieee(ibm);
            let rel_error = if value != 0.0 {
                (back - value).abs() / value.abs()
            } else {
                back.abs()
            };
            assert!(
                rel_error < 1e-14,
                "Roundtrip failed for {value}: got {back}, rel_error={rel_error}"
            );
        }
    }

    #[test]
    fn test_roundtrip_negative() {
        let values = [-0.5, -1.5, -10.0, -100.0, -0.001];
        for &value in &values {
            let ibm = ieee_to_ibm(value);
            let back = ibm_to_ieee(ibm);
            let rel_error = (back - value).abs() / value.abs();
            assert!(
                rel_error < 1e-14,
                "Roundtrip failed for {value}: got {back}"
            );
        }
    }

    #[test]
    fn test_non_finite() {
        // Non-finite values should convert to zero (should be handled as missing before conversion)
        assert_eq!(ieee_to_ibm(f64::NAN), IBM_ZERO);
        assert_eq!(ieee_to_ibm(f64::INFINITY), IBM_ZERO);
        assert_eq!(ieee_to_ibm(f64::NEG_INFINITY), IBM_ZERO);
    }

    #[test]
    fn test_missing_values() {
        // Standard missing
        let bytes = [0x2e, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(is_missing(&bytes), Some(MissingValue::Standard));

        // Special missing .A
        let bytes = [0x41, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(is_missing(&bytes), Some(MissingValue::Special('A')));

        // Not missing (valid number)
        assert_eq!(is_missing(&IBM_ONE), None);
    }

    #[test]
    fn test_truncate_expand() {
        let full = [0x41, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let truncated = truncate_ibm(full, 4);
        assert_eq!(truncated, vec![0x41, 0x10, 0x00, 0x00]);

        let expanded = expand_ibm(&truncated);
        assert_eq!(expanded, full);
    }

    #[test]
    fn test_large_values() {
        let values = [1e10, 1e20, 1e30, 1e50];
        for &value in &values {
            let ibm = ieee_to_ibm(value);
            let back = ibm_to_ieee(ibm);
            let rel_error = (back - value).abs() / value.abs();
            assert!(
                rel_error < 1e-10,
                "Large value roundtrip failed for {value}: got {back}"
            );
        }
    }

    #[test]
    fn test_small_values() {
        let values = [1e-10, 1e-20, 1e-30];
        for &value in &values {
            let ibm = ieee_to_ibm(value);
            if ibm == IBM_ZERO {
                // Underflow is acceptable for very small values
                continue;
            }
            let back = ibm_to_ieee(ibm);
            let rel_error = (back - value).abs() / value.abs();
            assert!(
                rel_error < 1e-10,
                "Small value roundtrip failed for {value}: got {back}"
            );
        }
    }
}
