#![no_std]
use core::ops::RangeInclusive;

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

/// 对外接口，实际操作由 Bits 结构体完成。
pub trait BitOps
where
    Self: Sized,
{
    fn bits<T: BitIndex>(self, range: T) -> Bits<T, Self> {
        Bits { value: self, range }
    }
}
macro_rules! mask {
    ($Type:ty, $Range:expr) => {
        match (1 as $Type).checked_shl($Range.len()) {
            Some(value) => value - 1,
            None => <$Type>::MAX,
        } << $Range.offset()
    };
}
macro_rules! impl_bitops {
    ($($Type:ty) *) => {
        $(impl BitOps for $Type {})*
    };
}

impl_bitops!(u8 u16 u32 u64 u128);

/// 该结构体可以通过 `0x10u32.bits(0x01)` 来构造
/// ```
/// extern crate bits;
/// use bits::BitOps;
/// assert_eq!(0u8.bits(0).set(), 0x01);
/// assert_eq!(0u8.bits(1).set(), 0x02);
/// assert_eq!(0u8.bits(4..=7).set(), 0xf0);
/// assert_eq!(0xffu8.bits(4..=7).clr(), 0x0f);
/// assert_eq!(0xffu8.bits(3).revert(), 0xf7);
/// assert_eq!(0xffu8.bits(4..=7).revert(), 0x0f);
/// assert_eq!(0u8.bits(4..=7).write(0x10), 0x10);
/// // 只会写入 value 的相应的 bit 位。低 4 bit 并不会被修改。
/// assert_eq!(0u8.bits(4..=7).write(0x12), 0x10);
/// assert_eq!(0x12u8.bits(4..=7).read(), 0x1);
/// ```
/// 单独构造该结构体主要是为了将 bit range 和要写入的值分开，这两者的类型可能会一样，在无 IDE 类型提示的情况下导致调用顺序颠倒：
/// `0u8.bits_write(5, 1);` 无法区分哪一个是 range，哪一个是要写入的值。
/// 当然也可以通过 `0u8.bits_set(5); ` 来避免，但 bits_write 依旧暴露存在风险。
///
/// 综上选择单独构造 Bits 结构体。
pub struct Bits<R: BitIndex, V: BitOps> {
    range: R,
    value: V,
}

macro_rules! impl_bits {
    ($($Type:ty) *) => {
        $(impl<T:BitIndex> Bits<T, $Type> {
            #[must_use="set function dosen't modify the self in place, you should assign to it explicitly"]
            pub fn set(&self) -> $Type {
                let mask = mask!($Type, self.range);
                self.value | mask
            }
            #[must_use="clr function dosen't modify the self in place, you should assign to it explicitly"]
            pub fn clr(&self) ->  $Type {
                let mask = mask!($Type, self.range);
                self.value & (!mask)
            }
            #[must_use="revert function dosen't modify the self in place, you should assign to it explicitly"]
            pub fn revert(&self) -> $Type {
                let mask = mask!($Type, self.range);
                self.value ^ mask
            }
            #[must_use="write function dosen't modify the self in place, you should assign to it explicitly"]
            pub fn write(&self, value: $Type) -> $Type {
                let mask = mask!($Type, self.range);
                (self.value & (!mask)) | ((value << self.range.offset()) & mask)
            }
            pub fn read(&self) -> $Type {
                let mask = mask!($Type, self.range);
                (self.value & mask) >> self.range.offset()
            }
        })*
    };
}
impl_bits!(u8 u16 u32 u64 u128);
