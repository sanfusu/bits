pub trait Bitalized {
    type BaseType;
}
pub trait Field {
    type CacheType;
}

pub trait ReadField<F: Field>
where
    Self: Bitalized,
{
    fn read(&self, field: F) -> F::CacheType;
}

/// TryReadField 并不要求 CacheType 实现 TryFrom<Self::BaseType>
/// 这样可以减少部分暴露。
/// 但是如果在 Macro 中使用了 Try, 则要求 TryFrom trait 的实现。
pub trait TryReadField<F>
where
    Self: Bitalized,
    F: Field,
{
    type Error;
    fn try_read(&self, field: F) -> Result<F::CacheType, Self::Error>;
}

pub trait WriteField<F>
where
    Self: Bitalized,
    F: Field,
{
    fn write(&mut self, field: F, v: F::CacheType);
}
