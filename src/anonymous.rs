use core::ops::{Range, RangeInclusive};

use crate::Bitalized;

pub struct Bits<U>(pub U);
impl<U> Bitalized for Bits<U> {
    type BaseType = U;
}
pub struct BitsMut<'a, U>(pub &'a mut U);

pub trait BitIndex<T>
where
    T: Bitalized,
{
    fn mask(&self) -> T::BaseType;
    fn offset(&self) -> T::BaseType;
}

macro_rules! impl_uint {
    ($($uint:ty),+) => {
        $(
            impl BitIndex<Bits<$uint>> for RangeInclusive<u32> {
                fn mask(&self) -> <Bits<$uint> as Bitalized>::BaseType {
                    (1 << (self.end())) - (1 << self.start())
                }
                fn offset(&self) -> <Bits<$uint> as Bitalized>::BaseType {
                    *self.start() as  <Bits<$uint> as Bitalized>::BaseType
                }
            }
            impl BitIndex<Bits<$uint>> for Range<u32> {
                fn mask(&self) -> <Bits<$uint> as Bitalized>::BaseType {
                    (1 << (self.end)) - (1 << self.start)
                }
                fn offset(&self) -> <Bits<$uint> as Bitalized>::BaseType {
                    self.start as  <Bits<$uint> as Bitalized>::BaseType
                }
            }
            impl BitIndex<Bits<$uint>> for u32 {
                fn mask(&self) -> <Bits<$uint> as Bitalized>::BaseType {
                    1 << self
                }
                fn offset(&self) -> <Bits<$uint> as Bitalized>::BaseType {
                    *self as  <Bits<$uint> as Bitalized>::BaseType
                }
            }
            impl Bits<$uint> {
                pub fn read<P>(&self, range: P) -> $uint where P: BitIndex<Bits<$uint>> {
                    let mask = range.mask();
                    (mask & self.0) >> range.offset()
                }
                pub fn write<P>(&mut self, range: P, v: $uint) where P: BitIndex<Bits<$uint>> {
                    let mask = range.mask();
                    self.0 = ((!mask) & self.0) | v
                }
                 pub fn set<P>(&mut self, range: P) where P: BitIndex<Bits<$uint>> {
                    let mask = range.mask();
                    self.0 = mask | self.0
                }
                 pub fn clear<P>(&mut self, range: P) where P: BitIndex<Bits<$uint>> {
                    let mask = range.mask();
                    self.0 = (!mask) & self.0
                }
                 pub fn has_all<P>(&self, range: P) -> bool where P: BitIndex<Bits<$uint>> {
                    let mask = range.mask();
                    (self.0 & mask) == mask
                }
                 pub fn has_any<P>(&self, range: P) -> bool where P: BitIndex<Bits<$uint>> {
                    let mask = range.mask();
                    (self.0 & mask) == 0
                }
            }

            impl<'a> BitsMut<'a, $uint> {
                pub fn write<P>(&mut self, range: P, v: $uint) where P: BitIndex<Bits<$uint>> {
                    let mask = range.mask();
                    debug_assert!(((!mask) & v) == 0, "value exceed range");
                    *self.0 = ((!mask) & *self.0) | v
                }
            }
        )+
    };
}

impl_uint!(u8, u16, u32, u64, u128);
