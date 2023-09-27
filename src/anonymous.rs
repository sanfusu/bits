use std::ops::Range;

pub struct Bits<U>(pub U);
pub struct BitsMut<'a, U>(pub &'a mut U);

macro_rules! impl_uint {
    ($($uint:ty),+) => {
        $(
            impl Bits<$uint> {
                pub fn read(&self, range: Range<u32>) -> $uint {
                    let mask = (1 << (range.end)) - (1 << range.start);
                    (mask & self.0) >> range.start
                }
                pub fn write(&mut self, range: Range<u32>, v: $uint) {
                    let mask = (1 << range.end) - (1 << range.start);
                    self.0 = ((!mask) & self.0) | v
                }
                pub fn set(&mut self, range: Range<u32>) {
                    let mask = (1 << range.end) - (1 << range.start);
                    self.0 = mask | self.0
                }
                pub fn clear(&mut self, range: Range<u32>) {
                    let mask = (1 << range.end) - (1 << range.start);
                    self.0 = (!mask) & self.0
                }
                pub fn has_all(&self, range: Range<u32>) -> bool {
                    let mask = (1 << range.end) - (1 << range.start);
                    (self.0 & mask) == mask
                }
                pub fn has_any(&self, range: Range<u32>) -> bool {
                    let mask = (1 << range.end) - (1 << range.start);
                    (self.0 & mask) == 0
                }
            }
            impl<'a> BitsMut<'a, $uint> {
                pub fn write(&mut self, range: Range<u32>, v: $uint) {
                    let mask = (1 << range.end) - (1 << range.start);
                    debug_assert!(((!mask) & v) == 0, "value exceed range");
                    *self.0 = ((!mask) & *self.0) | v
                }
            }
        )+
    };
}

impl_uint!(u8, u16, u32, u64, u128);
