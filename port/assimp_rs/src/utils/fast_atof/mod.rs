use error::FastAtofError;
use lexical_parse_float::{Options, format::STANDARD, parse::ParseFloat};

use crate::AiReal;
pub mod error;

#[allow(unused)]
pub static NUM_ITEMS: usize = 16;

#[allow(unused)]
pub static FAST_ATOF_TABLE: [f64; NUM_ITEMS] = [
    // we write [16] here instead of [] to work around a swig bug
    0.0, 1e-1, 1e-2, 1e-3, 1e-4, 1e-5, 1e-6, 1e-7, 1e-8, 1e-9, 1e-10, 1e-11, 1e-12, 1e-13, 1e-14,
    1e-15,
];

#[allow(unused)]
const AI_FAST_ATOF_RELAVANT_DECIMALS: usize = 15;

#[allow(unused)]
pub fn strtoul10_64(
    mut src: &[u8],
    max_count: Option<usize>,
) -> Result<(&[u8], u64, usize), FastAtofError> {
    let mut value = 0u64;
    assert!(src.len() > 0);
    let b = src[0];
    if b < b'0' || b > b'9' {
        return Err(FastAtofError::InvalidNumericString(
            String::from_utf8_lossy(src).into_owned(),
        ));
    }
    let mut cnt = 0;
    while let &[b, ref rest @ ..] = src {
        if !b.is_ascii_digit() {
            break;
        }
        let new_value = value.wrapping_mul(10).wrapping_add((b - b'0') as u64);
        if new_value < value {
            return Ok((src, 0, 0));
        }
        value = new_value;
        src = rest;
        cnt += 1;
        if Some(cnt) == max_count {
            while let &[b, ref rest @ ..] = src {
                if b.is_ascii_digit() {
                    src = rest;
                } else {
                    break;
                }
            }
            return Ok((src, value, cnt));
        }
    }
    return Ok((src, value, cnt));
}

#[allow(unused)]
pub fn fast_atoreal_move(src: &[u8], check_comma: bool) -> Result<(&[u8], AiReal), FastAtofError> {
    // let mut f = 0.0;
    // let (&maybe_sign_byte, rest) = src
    //     .split_first()
    //     .ok_or(FastAtofError::UnexpectedEndOfFile)?;
    // let inv = maybe_sign_byte == b'-';
    // if inv || maybe_sign_byte == b'+' {
    //     src = rest;
    // }
    // let (bytes, rest) = src
    //     .split_at_checked(3)
    //     .ok_or(FastAtofError::UnexpectedEndOfFile)?;
    // let bytes: &[u8; 3] = bytes.try_into().unwrap();
    // if bytes.eq_ignore_ascii_case(b"nan") {
    //     return Ok((rest, f64::NAN));
    // } else if bytes.eq_ignore_ascii_case(b"inf") {
    //     if let Some((_, _rest)) = rest.split_at_checked(5) {
    //         let rest = if rest.eq_ignore_ascii_case(b"inity") {
    //             _rest
    //         } else {
    //             rest
    //         };
    //         return Ok((
    //             rest,
    //             if inv {
    //                 f64::NEG_INFINITY
    //             } else {
    //                 f64::INFINITY
    //             },
    //         ));
    //     }
    // }
    // let (&byte, _) = src
    //     .split_first()
    //     .ok_or(FastAtofError::UnexpectedEndOfFile)?;
    // if !byte.is_ascii_digit() {
    //     return Err(FastAtofError::InvalidRealNumber(
    //         String::from_utf8_lossy(src).into_owned(),
    //     ));
    // }
    // if byte != b'.' && (!check_comma || byte != b',') {
    //     let (rest, value, _) = strtoul10_64(src, None)?;
    //     src = rest;
    //     f = value as f64;
    // }
    // let (a, rest) = src
    //     .split_at_checked(1)
    //     .ok_or(FastAtofError::UnexpectedEndOfFile)?;
    // let a = a[0];
    // let (&b, _) = rest
    //     .split_first()
    //     .ok_or(FastAtofError::UnexpectedEndOfFile)?;
    // if a == b'.' || (check_comma && a == b',') && b.is_ascii_digit() {
    //     src = rest;
    //     let (rest, value, diff) = strtoul10_64(src, Some(AI_FAST_ATOF_RELAVANT_DECIMALS))?;
    //     src = rest;
    //     f += (value as f64) * FAST_ATOF_TABLE[diff];
    // } else if a == b'.' {
    //     src = rest;
    // }
    // let (&b, rest) = src
    //     .split_first()
    //     .ok_or(FastAtofError::UnexpectedEndOfFile)?;
    // if b.eq_ignore_ascii_case(&b'e') {
    //     src = rest;
    //     let (&b, rest) = src
    //         .split_first()
    //         .ok_or(FastAtofError::UnexpectedEndOfFile)?;
    //     let e_inv = b == b'-';
    //     if e_inv || b == b'+' {
    //         src = rest;
    //     }
    //     let (rest, exp, _) = strtoul10_64(src, None)?;
    //     src = rest;
    //     f *= 10.0f64.powf(exp as f64);
    // }
    // if inv {
    //     f = -f;
    // }
    // Ok((src, f))
    match AiReal::fast_path_partial::<STANDARD>(src, const { &Options::new() }) {
        Ok((f, offs)) => {
            Ok((
                // SAFETY: offs is checked to be within src by fast_path_partial
                unsafe { src.get_unchecked(offs..) },
                f,
            ))
        }
        Err(e) => Err(FastAtofError::from(e)),
    }
}
