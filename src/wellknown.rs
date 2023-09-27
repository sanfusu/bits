use std::convert::{From, Into, TryFrom};

pub trait Bitalized {
    type BaseType;
}

pub trait ReadableField<C: Bitalized + ?Sized> {
    type TargetType: From<C::BaseType>;
}
pub trait WriteableField<C: Bitalized + ?Sized> {
    type TargetType: Into<C::BaseType>;
}

pub trait TryReadableField<C: Bitalized + ?Sized> {
    type TargetType: TryFrom<C::BaseType>;
}

pub trait ReadField<F: ReadableField<Self>>
where
    Self: Bitalized,
{
    fn read(&self, field: F) -> F::TargetType;
}
pub trait TryReadField<F: TryReadableField<Self>>
where
    Self: Bitalized,
{
    fn try_read(
        &self,
        field: F,
    ) -> Result<F::TargetType, <F::TargetType as TryFrom<Self::BaseType>>::Error>;
}
pub trait WriteField<F: WriteableField<Self>>
where
    Self: Bitalized,
{
    fn write(&mut self, field: F, v: F::TargetType);
}
