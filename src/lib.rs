use std::ops::RangeInclusive;

pub trait BitIndex {
    fn offset(&self) -> u32;
    fn len(&self) -> u32;
}

impl BitIndex for RangeInclusive<u32> {
    fn offset(&self) -> u32 {
        *self.start()
    }

    fn len(&self) -> u32 {
        self.end() - self.start()
    }
}
impl BitIndex for u32 {
    fn offset(&self) -> u32 {
        *self
    }

    fn len(&self) -> u32 {
        1
    }
}

pub trait BitOps
where
    Self: Sized,
{
    fn bits_set<T>(self, range: T) -> Self
    where
        T: BitIndex;
    fn bits_clr<T: BitIndex>(self, range: T) -> Self;
    fn bits_revert<T: BitIndex>(self, range: T) -> Self;
    fn bits_write<T: BitIndex>(self, range: T, value: Self) -> Self;
    fn bits_read<T: BitIndex>(self, range: T) -> Self;
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
        $(impl BitOps for $Type {
            fn bits_set<T:BitIndex>(self, range:T) -> Self {
                let mask = mask!($Type, range);
                self | mask
            }
            fn bits_clr<T:BitIndex>(self, range: T) -> Self {
                let mask = mask!($Type, range);
                self & (!mask)
            }
            fn bits_revert<T:BitIndex>(self, range: T) -> Self {
                let mask = mask!($Type, range);
                self ^ mask
            }
            fn bits_write<T:BitIndex>(self, range:T, value: Self) -> Self {
                let mask = mask!($Type, range);
                self & (!mask) | value
            }
            fn bits_read<T:BitIndex>(self, range: T) -> Self {
                let mask = mask!($Type, range);
                (self & mask) >> range.offset()
            }
        })*
    };
}

impl_bitops!(u8 u16 u32 u64 u128);
