#![no_std]
#![feature(test)]

extern crate test;

pub mod field;

use core::ops::{Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo};

/// 提供类似于 SliceIndex 的使用方式。
pub trait BitIndex {
    fn low(&self) -> Bound<&u32>;
    fn upper(&self) -> Bound<&u32>;
}

impl BitIndex for RangeInclusive<u32> {
    #[inline]
    fn upper(&self) -> Bound<&u32> {
        self.end_bound()
    }
    #[inline]
    fn low(&self) -> Bound<&u32> {
        self.start_bound()
    }
}
impl BitIndex for RangeFull {
    #[inline]
    fn low(&self) -> Bound<&u32> {
        Bound::Included(&0)
    }

    #[inline]
    fn upper(&self) -> Bound<&u32> {
        Bound::Unbounded
    }
}
impl BitIndex for RangeFrom<u32> {
    #[inline]
    fn low(&self) -> Bound<&u32> {
        self.start_bound()
    }

    #[inline]
    fn upper(&self) -> Bound<&u32> {
        Bound::Unbounded
    }
}
impl BitIndex for RangeTo<u32> {
    #[inline]
    fn low(&self) -> Bound<&u32> {
        Bound::Included(&0)
    }

    #[inline]
    fn upper(&self) -> Bound<&u32> {
        self.end_bound()
    }
}
impl BitIndex for Range<u32> {
    #[inline]
    fn upper(&self) -> Bound<&u32> {
        self.end_bound()
    }

    #[inline]
    fn low(&self) -> Bound<&u32> {
        self.start_bound()
    }
}
impl BitIndex for u32 {
    #[inline]
    fn upper(&self) -> Bound<&u32> {
        Bound::Included(self)
    }

    #[inline]
    fn low(&self) -> Bound<&u32> {
        Bound::Included(self)
    }
}
/// 将整型转为 Bits，实际操作由 BitsOps 来实现。
pub trait IntoBits
where
    Self: Sized + Copy,
{
    type Output: BitsOps<Self>;
    fn bits<T: BitIndex>(self, range: T) -> Self::Output;
}
macro_rules! mask {
    ($Type:ty, $Range:expr) => {
        (((1 as $Type) << $Range.end()) - ((1 as $Type) << $Range.start()))
            | ((1 as $Type) << $Range.end())
    };
}

macro_rules! impl_intobits {
    ($($Type:ty) *) => {
        $(impl IntoBits for $Type {
            type Output = Bits<Self>;

            fn bits<T: BitIndex>(self, range:T) -> Bits<Self>{
                let upper = match  <T as BitIndex>::upper(&range) {
                    Bound::Unbounded => <$Type>::BITS - 1,
                    Bound::Included(v) => *v,
                    Bound::Excluded(v) => *v - 1,
                };
                let low = match  <T as BitIndex>::low(&range) {
                    Bound::Unbounded => 0,
                    Bound::Included(v) => *v,
                    Bound::Excluded(v) => *v,
                };

                Bits {
                    value:self,
                    range: low ..= upper
                }
            }
        })*
    };
}

impl_intobits!(u8 u16 u32 u64 u128 usize);

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
///
/// ## 关于溢出
///
/// 只尽可能的使输出的值符合预期：
/// `0u8.bits(0..=10).set() == 0xff` `0xffu8.bits(3..2).clr() == 0xff`
/// 当然这两个代码片段在非 release 编译下会导致溢出 panic（rust 自带的溢出检查）。
pub struct Bits<V: IntoBits> {
    range: RangeInclusive<u32>,
    value: V,
}

impl<V: IntoBits> IntoIterator for Bits<V> {
    type Item = Bit<V>;

    type IntoIter = BitsIter<V>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            low: *self.range.start(),
            upper: *self.range.end(),
            value: self.value,
        }
    }
}

pub struct Bit<V: IntoBits> {
    value: V,
}
impl<V: IntoBits> Bit<V> {
    #[inline]
    pub fn is_set(&self) -> bool {
        self.value.bits(0).is_set()
    }
    #[inline]
    pub fn is_clr(&self) -> bool {
        !self.is_set()
    }
}

/// # Bits 迭代器
///
/// ~~⚠️ 性能较差，比手动掩码移位循环慢三分之一左右。~~
/// 目前使用迭代器的速度要比手动编码快 99%，很戏剧化。
pub struct BitsIter<V: IntoBits> {
    value: V,
    low: u32,
    upper: u32,
}

impl<V: IntoBits> Iterator for BitsIter<V> {
    type Item = Bit<V>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = if self.low <= self.upper {
            Some(Bit {
                value: self.value.bits(self.low).read(),
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
    fn count_ones(&self) -> u32;
}

macro_rules! impl_bitsops {
    ($($Type:ty) *) => {
        $(impl BitsOps<$Type> for Bits<$Type> {
            #[must_use="set function dosen't modify the self in place, you should assign to it explicitly"]
            #[inline]
            fn set(&self) -> $Type {
                let mask = mask!($Type, self.range);
                self.value | mask
            }
            #[must_use="clr function dosen't modify the self in place, you should assign to it explicitly"]
            #[inline]
            fn clr(&self) -> $Type {
                let mask = mask!($Type, self.range);
                self.value & (!mask)
            }
            #[must_use="revert function dosen't modify the self in place, you should assign to it explicitly"]
            #[inline]
            fn revert(&self) -> $Type {
                let mask = mask!($Type, self.range);
                self.value ^ mask
            }
            #[must_use="write function dosen't modify the self in place, you should assign to it explicitly"]
            #[inline]
            fn write(&self, value: $Type) -> $Type {
                let mask = mask!($Type, self.range);
                (self.value & (!mask)) | ((value << self.range.start()) & mask)
            }
            #[inline]
            fn read(&self) -> $Type {
                let mask = mask!($Type, self.range);
                (self.value & mask) >> self.range.start()
            }
            #[inline]
            fn is_clr(&self) -> bool {
                self.read() == 0
            }
            #[inline]
            fn is_set(&self) -> bool {
                let mask = mask!($Type, self.range);
                (self.value & mask) == mask
            }
            /// 运行效率和标准库（编译器内部提供的）不相上下。
            #[inline]
            fn count_ones(&self) -> u32 {
                use core::convert::TryInto;
                let mut ret = self.read();
                let mut i = 1;
                while i <= core::mem::size_of::<$Type>() * 4 {
                    let max = !(0 as $Type);
                    let div = (1 << i) + 1;
                    let a:$Type = max / div;
                    let b:$Type = a << i;
                    ret = (a & ret) + ((b & ret) >> i);
                    i = i << 1;
                }
                ret.try_into().unwrap()
            }
        })*
    };
}
impl_bitsops!(u8 u16 u32 u64 u128 usize);

/// 这是一个示例，旨在演示思路
/// 1. 先每两个 bit 为一组计数，并且每一组之间可以并行计算。（利用了加法器的 bit 间的并行性）
/// 2. 合并，得出每 4 个 bit 为一组的计数
/// 3. 再次合并，得出每 8 个 bit 为一组的计数。
/// 4. 如果是单字节，则到此结束，否则以此类推下去。
#[inline]
fn __count_ones_u8(data: u8) -> u32 {
    let x1 = data & 0b0101_0101;
    let x2 = (data & 0b1010_1010) >> 1;

    let y = x1 + x2;
    let y1 = (y & 0b1100_1100) >> 2;
    let y2 = y & 0b0011_0011;

    let z = y1 + y2;
    let z1 = z & 0b0000_1111;
    let z2 = (z & 0b1111_0000) >> 4;

    return (z2 + z1) as u32;
}

/// 这是一个示例，旨在演示思路，实际上在计算 x1 和 x2 时没有并行。
/// 所以利用 u8 来计算 u16 不是一个好的做法，
/// 沿着 __count_ones_u8 的思路才是正道。
#[no_mangle]
fn __count_ones_u16(data: u16) -> u32 {
    let x1 = __count_ones_u8(data.to_ne_bytes()[1]);
    let x2 = __count_ones_u8(data.to_ne_bytes()[0]);
    x1 + x2
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    use crate::{BitsOps, IntoBits};

    #[test]
    fn bits_ops_test() {
        assert_eq!(0xffu8.bits(..).clr(), 0);
        assert_eq!(0xffu8.bits(..1).clr(), 0xff - 0b1);
        assert_eq!(0xffu8.bits(1..).clr(), 1);
        assert_eq!(0xffu8.bits(1..2).clr(), 0xff - 0b10);
        assert_eq!(0xffu8.bits(1..=2).clr(), 0xff - 0b110);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn bits_ops_test_end_overflow() {
        0xffu8.bits(0..=8).clr();
    }
    #[test]
    #[should_panic(expected = "overflow")]
    fn bits_ops_test_start_overflow() {
        0xffu8.bits(2..=1).clr();
    }
    #[no_mangle]
    fn bits_iterator(data: u64, out: &mut [u8; 64]) {
        for (idx, bit) in data.bits(0..=63).into_iter().enumerate() {
            out[idx] = bit.is_set() as u8;
        }
    }
    #[no_mangle]
    fn plain_loop(data: u64, out: &mut [u8; 64]) {
        let mut mask = 0x1u64;
        let mut idx = 0usize;
        while idx < 64 {
            if data & mask != 0 {
                out[idx] = 1;
            } else {
                out[idx] = 0;
            }
            mask <<= 1;
            idx += 1;
        }
    }

    #[test]
    fn bits_iterator_test() {
        let mut out_iterator = [0u8; 64];
        let mut out_loop = [0u8; 64];
        (0..=0xffff).for_each(|x| {
            bits_iterator(x, &mut out_iterator);
            plain_loop(x, &mut out_loop);
            assert_eq!(out_iterator, out_loop);
        })
    }
    #[test]
    // TODO 需要随机测试
    fn count_ones_test() {
        (0..=0x7f).for_each(|x: u8| assert_eq!(x.bits(..).count_ones(), x.count_ones()));
        (0x5a5a..=0xffff).for_each(|x: u16| assert_eq!(x.bits(..).count_ones(), x.count_ones()));
        (0x5a5a5a5a..=0x5a5aff5a)
            .for_each(|x: u32| assert_eq!(x.bits(..).count_ones(), x.count_ones()));
        (0x5a5a_5a5a_5a5a_5a5a..=0x5a5a_55aa_ffff_5a5a)
            .for_each(|x: u64| assert_eq!(x.bits(..).count_ones(), x.count_ones()));
    }

    #[bench]
    fn bench_plain_loop_code(b: &mut Bencher) {
        let n = test::black_box(0xffff);
        let mut out = test::black_box([0u8; 64]);
        b.iter(|| (0..=n).for_each(|x| plain_loop(x, &mut out)))
    }

    #[bench]
    fn bench_bits_iterator_code(b: &mut Bencher) {
        let n = test::black_box(0xffff);
        let mut out = test::black_box([0u8; 64]);
        b.iter(|| (0..=n).for_each(|x| bits_iterator(x, &mut out)))
    }
    #[no_mangle]
    fn count_ones_bits(data: u64) -> u32 {
        data.bits(..).count_ones()
    }
    #[no_mangle]
    fn count_ones_interal(data: u64) -> u32 {
        data.count_ones()
    }
    #[bench]
    fn bench_count_ones_bits(b: &mut Bencher) {
        let n = test::black_box(0xffff);
        let mut result = test::black_box(0);
        b.iter(|| {
            (0..=n).for_each(|x: u16| {
                result += x.bits(0..=15).count_ones();
            })
        });
    }
    #[bench]
    fn bench_count_ones_internal(b: &mut Bencher) {
        let n = test::black_box(0xffff);
        let mut result = test::black_box(0);
        b.iter(|| {
            (0..=n).for_each(|x: u16| {
                result += x.count_ones();
            })
        })
    }
}

// 👌 请注意对比和修改测试跑分结果
//
// test tests::bench_bits_iterator_code  ... bench:      15,323 ns/iter (+/- 172)
// test tests::bench_count_ones_bits     ... bench:      26,211 ns/iter (+/- 326)
// test tests::bench_count_ones_internal ... bench:      28,036 ns/iter (+/- 494)
// test tests::bench_plain_loop_code     ... bench:   1,514,810 ns/iter (+/- 13,509)
