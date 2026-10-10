//! Byte-to-bool lookup table for unpacking packed bit strings.
//!
//! A byte has only 256 possible values, so the table maps each value to its 8 bools
//! once, ahead of time:
//!
//! ```text
//! TABLE[0b0000_0000] = [F, F, F, F, F, F, F, F]
//! TABLE[0b0000_0101] = [T, F, T, F, F, F, F, F]   // bit 0 first (LSB-first, matching our bit order)
//! TABLE[0b1111_1111] = [T, T, T, T, T, T, T, T]
//! ... 256 entries, 2 KB total
//! ```
//!
//! To unpack, split each 64-bit word into its 8 bytes and look each one up.

pub const WORD_SIZE: usize = 64;

/// Maps every byte value to its 8 bits as bools, least-significant bit first.
///
/// Computed at compile time.
pub const TABLE: [[bool; 8]; 256] = {
    let mut table = [[false; 8]; 256];
    let mut byte = 0;
    while byte < 256 {
        let mut bit = 0;
        while bit < 8 {
            table[byte][bit] = (byte >> bit) & 1 != 0;
            bit += 1;
        }
        byte += 1;
    }
    table
};
