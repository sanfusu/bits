use std::ops::RangeInclusive;

pub trait BitOps
where
    Self: Sized,
{
    fn bits_set(self, range: RangeInclusive<u32>) -> Self;
    fn bits_clr(self, range: RangeInclusive<u32>) -> Self;
    fn bits_revert(self, range: RangeInclusive<u32>) -> Self;
    fn bits_write(self, range: RangeInclusive<u32>, value: Self) -> Self;
    fn bits_read(self, range: RangeInclusive<u32>) -> Self;
}

macro_rules! impl_bitops {
    ($($Type:ty) *) => {
        $(impl BitOps for $Type {
            fn bits_set(self, range: RangeInclusive<u32>) -> Self {
                let mask = match (1 as $Type).checked_shl(range.end() - range.start()){
                    Some(value) => value - 1,
                    None => <$Type>::MAX,
                };
                self | mask
            }
            fn bits_clr(self, range: RangeInclusive<u32>) -> Self {
                let mask = 1 << (range.end() - range.start()) - 1;
                self & (!mask)
            }
            fn bits_revert(self, range: RangeInclusive<u32>) -> Self {
                let mask = 1 << (range.end() - range.start()) - 1;
                self ^ mask
            }
            fn bits_write(self, range: RangeInclusive<u32>, value: Self) -> Self {
                let mask = 1 << (range.end() - range.start()) - 1;
                self & (!mask) | value
            }
            fn bits_read(self, range: RangeInclusive<u32>) -> Self {
                let mask = 1 << (range.end() - range.start()) - 1;
                (self & mask) >> range.start()
            }
        })*
    };
}

impl_bitops!(u8 u16 u32 u64 u128);
