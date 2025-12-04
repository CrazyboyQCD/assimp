/*
---------------------------------------------------------------------------
Open Asset Import Library (assimp)
---------------------------------------------------------------------------

Copyright (c) 2006-2025, assimp team

All rights reserved.

Redistribution and use of this software in source and binary forms,
with or without modification, are permitted provided that the following
conditions are met:

* Redistributions of source code must retain the above
  copyright notice, this list of conditions and the
  following disclaimer.

* Redistributions in binary form must reproduce the above
  copyright notice, this list of conditions and the
  following disclaimer in the documentation and/or other
  materials provided with the distribution.

* Neither the name of the assimp team, nor the names of its
  contributors may be used to endorse or promote products
  derived from this software without specific prior
  written permission of the assimp team.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
---------------------------------------------------------------------------
*/

//! Implements file-format-specific importer(s) and exporter(s) for the library

#[cfg(feature = "mmd_format")]
pub mod mmd;
#[cfg(feature = "x_format")]
pub mod x;

#[cfg(feature = "std")] // Gltf crate is not no-std compatible yet
#[cfg(feature = "gltf_format")]
pub mod gltf2;

pub mod assbin;
pub mod assxml;

use formatter_utils::*;

mod formatter_utils {
    use core::{
        fmt::{Display, Formatter, Result as FmtResult},
        time::Duration,
    };

    pub(super) trait RepeatedFormatter: Sized {
        fn next(self) -> Self;
        fn back(self) -> Self;
    }

    const DEFAULT_INDENT: &str = "  ";

    /// Level of indentation
    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub(super) struct DefaultRepeatedIndent(usize);

    impl DefaultRepeatedIndent {
        pub(super) const fn new(count: usize) -> Self {
            Self(count)
        }
    }

    impl RepeatedFormatter for DefaultRepeatedIndent {
        /// Get the next level.
        fn next(self) -> Self {
            DefaultRepeatedIndent::new(self.0.wrapping_add(1))
        }

        /// Get the previous level.
        fn back(self) -> Self {
            DefaultRepeatedIndent::new(self.0.wrapping_sub(1))
        }
    }

    impl Display for DefaultRepeatedIndent {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
            (0..self.0).try_for_each(|_| formatter.write_str(DEFAULT_INDENT))
        }
    }

    #[derive(Clone, Copy)]
    pub(super) struct CustomRepeatedString(usize, &'static str);

    impl CustomRepeatedString {
        pub const fn new(count: usize, s: &'static str) -> Self {
            Self(count, s)
        }
    }

    impl RepeatedFormatter for CustomRepeatedString {
        /// Get the next level.
        fn next(self) -> Self {
            CustomRepeatedString(self.0.wrapping_add(1), self.1)
        }

        /// Get the previous level.
        fn back(self) -> Self {
            CustomRepeatedString(self.0.wrapping_sub(1), self.1)
        }
    }

    impl Display for CustomRepeatedString {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
            (0..self.0).try_for_each(|_| formatter.write_str(self.1))
        }
    }

    pub(super) struct AscTimeFormatter(core::time::Duration);

    impl AscTimeFormatter {
        #[allow(unused)]
        pub fn new(d: Duration) -> Self {
            Self(d)
        }

        pub(super) fn now() -> Self {
            Self(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default(),
            )
        }
    }

    impl Display for AscTimeFormatter {
        // Ported from musl
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            /// 2000-03-01 (mod 400 year, immediately after feb29
            const LEAPOCH: i64 = 946684800 + 86400 * (31 + 29);
            const DAYS_PER_400Y: i32 = 365 * 400 + 97;
            const DAYS_PER_100Y: i32 = 365 * 100 + 24;
            const DAYS_PER_4Y: i32 = 365 * 4 + 1;
            const MONTH_COUNT: usize = 12;
            const NAME_BYTE_CNT: usize = 3;
            const MONTH_NAMES_LEN: usize = MONTH_COUNT * NAME_BYTE_CNT;
            // merge names to save some binary space
            const MONTH_AND_WEEK_DAY_NAMES: &[u8] =
                b"JanFebMarAprMayJunJulAugSepOctNovDecSunMonTueWedThuFriSat";
            const DAYS_IN_MONTH: [i32; MONTH_COUNT] =
                [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];

            let t = self.0.as_secs() as i64;
            let secs_since_leapoch = t - LEAPOCH;

            let mut days = (secs_since_leapoch / 86400) as i32;
            let mut rem_secs = (secs_since_leapoch % 86400) as i32;

            if rem_secs < 0 {
                rem_secs += 86400;
                days -= 1;
            }

            let w_day = (3 + days) % 7;
            let w_day = if w_day < 0 { w_day + 7 } else { w_day } as usize;
            assert!(w_day < 7);

            let mut qc_cycles = days / DAYS_PER_400Y;
            let mut rem_days = days % DAYS_PER_400Y;

            if rem_days < 0 {
                rem_days += DAYS_PER_400Y;
                qc_cycles -= 1;
            }

            let mut c_cycles = rem_days / DAYS_PER_100Y;
            if c_cycles == 4 {
                c_cycles -= 1;
            }
            rem_days -= c_cycles * DAYS_PER_100Y;

            let mut q_cycles = rem_days / DAYS_PER_4Y;
            if q_cycles == 25 {
                q_cycles -= 1;
            }
            rem_days -= q_cycles * DAYS_PER_4Y;

            let mut rem_years = rem_days / 365;
            if rem_years == 4 {
                rem_years -= 1;
            }
            rem_days -= rem_years * 365;

            let mut years = rem_years + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;

            let mut months = 0;
            for (i, &days) in DAYS_IN_MONTH.iter().enumerate() {
                months = i as i32;
                if rem_days >= days {
                    rem_days -= days;
                } else {
                    break;
                }
            }
            if months >= 10 {
                months -= 12;
                years += 1;
            }

            assert!((-2..10).contains(&months));

            let year = (years + 2000) as usize;
            let month = (months + 2) as usize;
            let mday = (rem_days + 1) as usize;
            let wday = w_day;

            let hour = (rem_secs / 3600) as usize;
            let min = (rem_secs / 60 % 60) as usize;
            let sec = (rem_secs % 60) as usize;

            let (week_day_name, month_name) = {
                let week_start = MONTH_NAMES_LEN + wday * NAME_BYTE_CNT;
                let week_day_name_bytes =
                    &MONTH_AND_WEEK_DAY_NAMES[week_start..week_start + NAME_BYTE_CNT];

                let month_start = month * NAME_BYTE_CNT;
                let month_name_bytes =
                    &MONTH_AND_WEEK_DAY_NAMES[month_start..month_start + NAME_BYTE_CNT];

                // SAFETY: The names is ascii so the bytes are valid UTF-8.
                unsafe {
                    (
                        str::from_utf8_unchecked(week_day_name_bytes),
                        str::from_utf8_unchecked(month_name_bytes),
                    )
                }
            };

            write!(
                f,
                "{week_day_name} {month_name}{mday:3} {hour:02}:{min:02}:{sec:02} {year}\n",
            )
        }
    }
}
