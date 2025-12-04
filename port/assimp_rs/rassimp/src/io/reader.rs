use byteorder::ByteOrder;
use glam::{DVec2, DVec3, DVec4, Vec2, Vec3, Vec4};

pub mod binary_reader;
pub mod error;
pub mod text_reader;

/// Parse 4 bytes read from bytes into 4 digits.
///
/// Copied from https://github.com/Alexhuszagh/rust-lexical/blob/988575dad6de2a9e86b34fff242c5f0a6e3dbf2c/lexical-parse-integer/src/algorithm.rs#L259
#[inline]
pub fn parse_4digits<const RADIX: u32>(mut v: u32) -> u32 {
    const {
        assert!(RADIX <= 10, "RADIX must be less than or equal to 10");
    }
    const SUB_MASK: u32 = 0x3030_3030;
    const DIGIT_MASK: u32 = 0x7f;
    v -= SUB_MASK;
    // Scale digits in `0 <= Nn <= 99`.
    v = (v * RADIX) + (v >> 8);
    // Scale digits in `0 <= Nnnn <= 9999`.
    v = ((v & DIGIT_MASK) * RADIX * RADIX) + ((v >> 16) & DIGIT_MASK);

    v
}

mod test {
    #[test]
    fn test_parse_4digits() {
        macro_rules! test_parse_4digits {
          ($($radix: literal, $upper: literal)*) => {
                $(
                  for a in b'0'..=$upper {
                    for b in b'0'..=$upper {
                        for c in b'0'..=$upper {
                            for d in b'0'..=$upper {
                                let v = u32::from_le_bytes([a, b, c, d]);
                                let a = (a - b'0') as u32;
                                let b = (b - b'0') as u32;
                                let c = (c - b'0') as u32;
                                let d = (d - b'0') as u32;
                                let expected = a * $radix * $radix * $radix + b * $radix * $radix + c * $radix + d;
                                assert_eq!(super::parse_4digits::<$radix>(v), expected);
                            }
                        }
                    }
                  }
                )*
            };
          }
        test_parse_4digits!(
          10, b'9'
           9, b'8'
           8, b'7'
           7, b'6'
           6, b'5'
           5, b'4'
           4, b'3'
           3, b'2'
           2, b'1'
           1, b'0'
        );
    }
}
