use core::{marker::PhantomData, mem};
use std::io::BufRead;

use byteorder::ByteOrder;
#[cfg(feature = "std")]
use byteorder::ReadBytesExt;
use glam::{DVec2, DVec3, DVec4, Vec2, Vec3, Vec4};

use crate::{AiReal, AiVec2, AiVec3, AiVec4};

pub struct BinaryBufferReader<'source, T: ByteOrder> {
    start: *const u8,
    source: &'source [u8],
    _marker: PhantomData<T>,
}

impl<'source, T: ByteOrder> BinaryBufferReader<'source, T> {
    pub const fn new(source: &'source [u8]) -> Self {
        Self {
            start: source.as_ptr(),
            source,
            _marker: PhantomData,
        }
    }

    pub fn offset(&self) -> usize {
        // SAFETY: Both pointers are from the same allocation and within the bounds of the source,
        // and there is no public and internal operations to make source smaller than start.
        unsafe { self.source.as_ptr().offset_from_unsigned(self.start) }
    }

    pub const fn rest(&self) -> usize {
        self.source.len()
    }

    pub const fn len(&self) -> usize {
        self.source.len()
    }

    pub const fn first(&self) -> Option<u8> {
        self.source.first().copied()
    }

    pub fn peek<const N: usize>(&self) -> Option<&'source [u8; N]> {
        let s = self.source.get(..N)?;
        assert!(s.len() == N);
        Some(s.try_into().unwrap())
    }

    pub fn forward(&mut self, n: usize) -> Option<&'source [u8]> {
        let (data, rest) = self.source.split_at_checked(n)?;
        assert!(data.len() == n);
        self.source = rest;
        Some(data)
    }

    pub const fn clear(&mut self) {
        self.source = &[];
    }

    pub const unsafe fn forward_unchecked(&mut self, n: usize) -> &'source [u8] {
        let (data, rest) = unsafe { self.source.split_at_unchecked(n) };
        self.source = rest;
        data
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self.source.as_ptr()
    }

    pub const fn is_empty(&self) -> bool {
        self.source.is_empty()
    }
}

impl<'source, T: ByteOrder> BinaryRead<T> for BinaryBufferReader<'source, T> {
    fn read_exact(&mut self, buf: &mut [u8]) -> Option<()> {
        let (data, rest) = self.source.split_at_checked(buf.len())?;
        buf.copy_from_slice(data);
        self.source = rest;
        Some(())
    }
}

#[cfg(feature = "std")]
pub struct BinaryStreamReader<T: ByteOrder, R: ReadBytesExt> {
    source: R,
    _marker: PhantomData<T>,
}

impl<T: ByteOrder, R: ReadBytesExt> BinaryStreamReader<T, R> {
    pub fn new(source: R) -> Self {
        Self {
            source,
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "std")]
impl<T: ByteOrder, R: ReadBytesExt + BufRead> BinaryStreamReader<T, R> {
    pub fn is_eof(&mut self) -> bool {
        self.source.fill_buf().unwrap_or(&[]).is_empty()
    }
}

#[cfg(feature = "std")]
impl<T: ByteOrder, R: ReadBytesExt> BinaryRead<T> for BinaryStreamReader<T, R> {
    fn read_exact(&mut self, buf: &mut [u8]) -> Option<()> {
        self.source.read_exact(buf).ok()?;
        Some(())
    }
}

/// A `no_std` compatible interface for reading binary data for `ByteOrder` types.
pub(crate) trait BinaryRead<T: ByteOrder> {
    fn read_exact(&mut self, buf: &mut [u8]) -> Option<()>;

    fn read_into_buffer(&mut self, buf: &mut [u8]) -> Option<()> {
        self.read_exact(buf)?;
        Some(())
    }

    fn read_u8(&mut self) -> Option<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Some(buf[0])
    }

    fn read_i8(&mut self) -> Option<i8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Some(buf[0] as i8)
    }

    fn read_u16(&mut self) -> Option<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Some(T::read_u16(&buf))
    }

    fn read_i16(&mut self) -> Option<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Some(T::read_i16(&buf))
    }

    fn read_u24(&mut self) -> Option<u32> {
        let mut buf = [0; 3];
        self.read_exact(&mut buf)?;
        Some(T::read_u24(&buf))
    }

    fn read_i24(&mut self) -> Option<i32> {
        let mut buf = [0; 3];
        self.read_exact(&mut buf)?;
        Some(T::read_i24(&buf))
    }

    fn read_u32(&mut self) -> Option<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Some(T::read_u32(&buf))
    }

    fn read_i32(&mut self) -> Option<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Some(T::read_i32(&buf))
    }

    fn read_u48(&mut self) -> Option<u64> {
        let mut buf = [0; 6];
        self.read_exact(&mut buf)?;
        Some(T::read_u48(&buf))
    }

    fn read_i48(&mut self) -> Option<i64> {
        let mut buf = [0; 6];
        self.read_exact(&mut buf)?;
        Some(T::read_i48(&buf))
    }

    fn read_u64(&mut self) -> Option<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Some(T::read_u64(&buf))
    }

    fn read_i64(&mut self) -> Option<i64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Some(T::read_i64(&buf))
    }

    fn read_u128(&mut self) -> Option<u128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf)?;
        Some(T::read_u128(&buf))
    }

    fn read_i128(&mut self) -> Option<i128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf)?;
        Some(T::read_i128(&buf))
    }

    fn read_uint128(&mut self, nbytes: usize) -> Option<u128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf[..nbytes])?;
        Some(T::read_uint128(&buf[..nbytes], nbytes))
    }

    fn read_int128(&mut self, nbytes: usize) -> Option<i128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf[..nbytes])?;
        Some(T::read_int128(&buf[..nbytes], nbytes))
    }

    fn read_f32(&mut self) -> Option<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Some(T::read_f32(&buf))
    }

    fn read_f64(&mut self) -> Option<f64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Some(T::read_f64(&buf))
    }

    fn read_ai_real(&mut self) -> Option<AiReal> {
        let mut buf = [0; mem::size_of::<AiReal>()];
        self.read_exact(&mut buf)?;
        #[cfg(not(feature = "double_precision"))]
        {
            Some(T::read_f32(&buf))
        }
        #[cfg(feature = "double_precision")]
        {
            Some(T::read_f64(&buf))
        }
    }

    fn read_vec2(&mut self) -> Option<Vec2> {
        Some(Vec2::new(self.read_f32()?, self.read_f32()?))
    }

    fn read_vec3(&mut self) -> Option<Vec3> {
        Some(Vec3::new(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    fn read_vec4(&mut self) -> Option<Vec4> {
        Some(Vec4::new(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    fn read_dvec2(&mut self) -> Option<DVec2> {
        Some(DVec2::new(self.read_f64()?, self.read_f64()?))
    }

    fn read_dvec3(&mut self) -> Option<DVec3> {
        Some(DVec3::new(
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
        ))
    }

    fn read_dvec4(&mut self) -> Option<DVec4> {
        Some(DVec4::new(
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
        ))
    }

    fn read_ai_vec2(&mut self) -> Option<AiVec2> {
        #[cfg(not(feature = "double_precision"))]
        {
            self.read_vec2()
        }
        #[cfg(feature = "double_precision")]
        {
            self.read_dvec2()
        }
    }

    fn read_ai_vec3(&mut self) -> Option<AiVec3> {
        #[cfg(not(feature = "double_precision"))]
        {
            self.read_vec3()
        }
        #[cfg(feature = "double_precision")]
        {
            self.read_dvec3()
        }
    }

    fn read_ai_vec4(&mut self) -> Option<AiVec4> {
        #[cfg(not(feature = "double_precision"))]
        {
            self.read_vec4()
        }
        #[cfg(feature = "double_precision")]
        {
            self.read_dvec4()
        }
    }
}
