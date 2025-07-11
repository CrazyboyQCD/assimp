#[cfg(feature = "x_file")]
pub mod x;

const INDENT: &str = "  ";

/// Level of indentation
#[derive(Clone, Copy)]
pub struct Level(pub usize);

impl Level {
    pub const fn next(self) -> Self {
        Level(self.0 + 1)
    }

    pub const fn back(self) -> Self {
        Level(self.0.wrapping_sub(1))
    }
}

impl core::fmt::Display for Level {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        (0..self.0).try_for_each(|_| formatter.write_str(INDENT))
    }
}
