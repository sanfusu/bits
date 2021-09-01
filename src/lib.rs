#![no_std]
pub mod field;

use core::ops::{Range, RangeInclusive};

/// 提供类似于 SliceIndex 的使用方式。
pub trait BitIndex {
    fn offset(&self) -> u32;
    fn len(&self) -> u32;
}

impl BitIndex for RangeInclusive<u32> {
    #[inline]
    fn offset(&self) -> u32 {
        *self.start()
    }
    #[inline]
    fn len(&self) -> u32 {
        self.end() - self.start() + 1
    }
}

impl BitIndex for Range<u32> {
    #[inline]
    fn offset(&self) -> u32 {
        self.start
    }

    #[inline]
    fn len(&self) -> u32 {
        self.end - self.start
    }
}
impl BitIndex for u32 {
    #[inline]
    fn offset(&self) -> u32 {
        *self
    }
    #[inline]
    fn len(&self) -> u32 {
        1
    }
}

/// 将整型转为 Bits，实际操作由 BitsOps 来实现。
pub trait IntoBits<T: BitIndex>
where
    Self: Sized + Copy,
{
    type Output: BitsOps<Self>;
    fn bits(self, range: T) -> Self::Output;
}
macro_rules! mask {
    ($Type:ty, $Range:expr) => {
        match (1 as $Type).checked_shl($Range.len()) {
            Some(value) => value - 1,
            None => <$Type>::MAX,
        } << $Range.offset()
    };
}
macro_rules! impl_intobits {
    ($($Type:ty) *) => {
        $(impl<T:BitIndex> IntoBits<T> for $Type {
            type Output = Bits<T, Self>;
            fn bits(self, range:T) -> Self::Output{
                Bits {
                    value:self,
                    range
                }
            }
        })*
    };
}

impl_intobits!(u8 u16 u32 u64 u128);

/// 该结构体可以通过 `0x10u32.bits(0x01)` 来构造
/// ```
/// use bits::BitsOps;
/// use bits::IntoBits;
/// assert_eq!(0u8.bits(0).set(), 0x01);
/// assert_eq!(0u8.bits(1).set(), 0x02);
/// assert_eq!(0u8.bits(4..=7).set(), 0xf0);
/// assert_eq!(0xffu8.bits(4..=7).clr(), 0x0f);
/// assert_eq!(0xffu8.bits(3).revert(), 0xf7);
/// assert_eq!(0xffu8.bits(4..=7).revert(), 0x0f);
/// assert_eq!(0u8.bits(4..=7).write(0x10), 0x0);
/// // 只会写入 value 的相应的 bit 位。低 4 bit 并不会被修改。
/// assert_eq!(0u8.bits(4..=7).write(0x12), 0x20);
/// assert_eq!(0x12u8.bits(4..=7).read(), 0x1);
/// assert_eq!(0xf0u8.bits(4..=7).is_set(), true);
/// assert_eq!(0x70u8.bits(4..=7).is_set(), false);
/// assert_eq!(0x70u8.bits(4..=7).is_clr(), false);
/// assert_eq!(0x70u8.bits(0..=3).is_clr(), true);
/// ```
/// 单独构造该结构体主要是为了将 bit range 和要写入的值分开，这两者的类型可能会一样，在无 IDE 类型提示的情况下导致调用顺序颠倒：
/// `0u8.bits_write(5, 1)` 无法区分哪一个是 range，哪一个是要写入的值。
///
/// 当然也可以通过 `0u8.bits_set(5); ` 来避免，但 bits_write 的存在依旧会暴露风险。
///
/// 综上选择单独构造 Bits 结构体。
pub struct Bits<R: BitIndex, V: IntoBits<R>> {
    range: R,
    value: V,
}

impl<R: BitIndex, V: IntoBits<R> + IntoBits<u32>> IntoIterator for Bits<R, V> {
    type Item = Bit;

    type IntoIter = BitsIter<V>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            low: self.range.offset(),
            upper: self.range.offset() + self.range.len() - 1,
            value: self.value,
        }
    }
}

pub struct Bit {
    value: bool,
}
impl Bit {
    #[inline]
    pub fn is_set(&self) -> bool {
        self.value
    }
    #[inline]
    pub fn is_clr(&self) -> bool {
        !self.value
    }
}
pub struct BitsIter<V: IntoBits<u32>> {
    value: V,
    upper: u32,
    low: u32,
}

impl<V: IntoBits<u32>> Iterator for BitsIter<V> {
    type Item = Bit;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = if self.low <= self.upper {
            Some(Bit {
                value: self.value.bits(self.low).is_set(),
            })
        } else {
            None
        };
        self.low += 1;
        ret
    }
}
/// bits 的实际操作
pub trait BitsOps<T> {
    fn set(&self) -> T;
    fn clr(&self) -> T;
    fn revert(&self) -> T;
    fn write(&self, value: T) -> T;
    fn read(&self) -> T;
    fn is_clr(&self) -> bool;
    fn is_set(&self) -> bool;
}

macro_rules! impl_bitsops {
    ($($Type:ty) *) => {
        $(impl<R:BitIndex> BitsOps<$Type> for  Bits<R, $Type> {
            #[must_use="set function dosen't modify the self in place, you should assign to it explicitly"]
            fn set(&self) -> $Type {
                let mask = mask!($Type, self.range);
                self.value | mask
            }
            #[must_use="clr function dosen't modify the self in place, you should assign to it explicitly"]
            fn clr(&self) ->  $Type {
                let mask = mask!($Type, self.range);
                self.value & (!mask)
            }
            #[must_use="revert function dosen't modify the self in place, you should assign to it explicitly"]
            fn revert(&self) -> $Type {
                let mask = mask!($Type, self.range);
                self.value ^ mask
            }
            #[must_use="write function dosen't modify the self in place, you should assign to it explicitly"]
            fn write(&self, value: $Type) -> $Type {
                let mask = mask!($Type, self.range);
                (self.value & (!mask)) | ((value << self.range.offset()) & mask)
            }
            fn read(&self) -> $Type {
                let mask = mask!($Type, self.range);
                (self.value & mask) >> self.range.offset()
            }
            fn is_clr(&self)-> bool {
                self.read() == 0
            }
            fn is_set(&self)->bool {
                self.read() == mask!($Type, self.range) >> self.range.offset()
            }
        })*
    };
}
impl_bitsops!(u8 u16 u32 u64 u128);
