/// 用于为含有 bit field 的结构体中的 bit 字段实现
/// [`Field`]、[`FieldReader`]、[`FieldWriter`] 接口。
/// 具体用法看示例：
///  ```
/// use bits::field::BufferWriter;
/// use bits::field::BufferReader;
/// use bits::IntoBits;
/// use bits::BitsOps;
///
/// pub struct FoolData {
///     data: u32,
///     data1: u32,
/// }
/// impl BufferWriter for FoolData{}
/// impl BufferReader for FoolData{}
///
/// // bit 字段 1，其余类推
/// pub struct Flag1;
/// pub struct Flag2;
/// pub struct Flag3;
/// pub struct Flag4;
/// pub struct Flag5;
/// pub struct Flag6;
///
/// bits::fields! {
///     FoolData [data] {
///         Flag1 [0..=3, rw, u32],
///         Flag2 [4..=5, rw, u32],
///         Flag3 [6, ro, bool],
///         Flag4 [7, rw, bool],
///         Flag5 [8..=9, rw, bool] {
///             input_converter: |x| match x {
///                 true => 0x1,
///                 _ => 0x0
///             };
///             output_converter: |x| match x {
///                 0x1 => true,
///                 _ => false
///             }
///         }
///     }
///     FoolData [data1] {
///         Flag6 [0..=3, rw, u32]
///     }
/// }
/// let mut fool = FoolData {data:0x0, data1: 0x0};
/// fool.write::<Flag1>(0xf);
/// assert_eq!(fool.data, 0xf);
///
/// fool.write::<Flag2>(0x3);
/// assert_eq!(fool.data, 0b0011_1111);
///
/// // error: the trait `FieldWriter<FoolData>` is not implemented for `Flag3`
/// // fool.write::<Flag3>(true); // Flag3 is not writeable
///
/// fool.write::<Flag4>(true);
/// assert_eq!(fool.data, 0b1011_1111);
///
/// let flag3 = fool.read::<Flag3>();
/// assert_eq!(flag3, false);
///
/// let flag4 = fool.read::<Flag4>();
/// assert_eq!(flag4, true);
///
/// fool.data = fool.data.bits(8..=9).write(0x2); // set fool.data bits 8..=9 to 0x2
/// assert_eq!(false, fool.read::<Flag5>()); // bits: 8..=9 equal to 0x2 which is false
///
/// fool.write::<Flag5>(true);
/// assert_eq!(0b01, fool.data.bits(8..=9).read());
/// ```
#[macro_export]
macro_rules! fields {
    (
        $(
            $buffer_type:ty [$raw_data:ident] {
                $(
                    $field:tt [$position_start:literal $(..= $position_end:literal)? , $rw:tt, $value_type:tt $(<$generic:tt>)?] $({
                        $(input_converter:$input_converter:expr;)?
                        $(output_converter:$output_converter:expr)?
                    })?
                ),+
            }
        )+
    ) => {
        $(
            $(
                impl $crate::field::Field<$buffer_type> for $field {
                    type ValueType = $value_type $(<$generic>)?;
                }
                $crate::fields!(
                    __ops: $rw,  $buffer_type, $value_type, $field, $raw_data, $position_start $(..= $position_end)? $(
                        $(input_converter:$input_converter;)?
                        $(output_converter:$output_converter;)?
                    )?
                );
            )+
        )+
    };
    (__ops: ro, $buffer_type:ty, $value_type:tt, $field:ident, $raw_data:ident, $position_start:literal $(..= $position_end:literal)?
        $(output_converter:$output_converter:expr)?
    ) => {
        impl $crate::field::FieldReader<$buffer_type> for $field {
            fn read(buffer: &$buffer_type) -> Self::ValueType {
                use $crate::IntoBits;
                use $crate::BitsOps;
                let x = buffer.$raw_data.bits($position_start $(..= $position_end)?).read();
                let y = $crate::fields!{
                    __output_converter: x, $value_type, $position_start $(..= $position_end)? $(,$output_converter)?
                };
                y
            }
        }
    };
    (__ops: rw, $buffer_type:ty, $value_type:tt, $field:ident, $raw_data:ident, $position_start:literal $(..= $position_end:literal)?
        $(input_converter:$input_converter:expr;)?
        $(output_converter:$output_converter:expr;)?
    ) => {
        $crate::fields!{
            __ops: ro, $buffer_type, $value_type, $field, $raw_data, $position_start $(..= $position_end)? $(output_converter:$output_converter)?
        }
        impl $crate::field::FieldWriter<$buffer_type> for $field {
            fn write(buffer: &mut $buffer_type, value: Self::ValueType){
                use $crate::IntoBits;
                use $crate::BitsOps;
                buffer.$raw_data = buffer.$raw_data.bits($position_start $(..= $position_end)?).write(
                    $crate::fields!{
                        __input_converter: value, $value_type, $position_start $(..= $position_end)? $(,$input_converter)?
                    }
                );
            }
        }
    };
    (__output_converter: $var:ident, bool, $position_start:literal) => {
        ($var == 1)
    };
    ($__converter:tt: $var:ident, $value_type:tt, $position_start:literal $(..= $position_end:literal)?) => {
        $var.into()
    };
    ($__converter:tt: $var:ident, $value_type:tt, $position_start:literal $(..= $position_end:literal)?, $converter:expr) => {
        $converter($var)
    };
}

/// # Buffer 写入器
///
/// 通常将 bit 字段归属的结构体看作缓存。通过实现 BufferWriter 来对 bit 字段进行写入操作。
/// bit 字段所归属的结构体需要实现该接口（如果含有任何可写 bit 字段）。
///
/// 👍 将 bit 字段归属的结构体看作缓存，在实现寄存器读写操作时会非常有益。
pub trait BufferWriter {
    fn write<T>(&mut self, value: T::ValueType) -> &mut Self
    where
        T: Field<Self> + FieldWriter<Self>,
    {
        T::write(self, value);
        self
    }
}
/// # Buffer 读出器
///
/// 通常将 bit 字段归属的结构体看作缓存。通过实现 BufferReader 来对 bit 字段进行读出操作。
/// bit 字段所归属的结构体需要实现该接口（如果含有任何可读 bit 字段）。
///
/// 👍 将 bit 字段归属的结构体看作缓存，在实现寄存器读写操作时会非常有益。
pub trait BufferReader {
    fn read<T: Field<Self> + FieldReader<Self>>(&self) -> T::ValueType {
        T::read(self)
    }
    fn output<T: Field<Self> + FieldReader<Self>>(&self, out: &mut T::ValueType) -> &Self {
        *out = T::read(self);
        self
    }
}

/// # 标识一个 bit 字段
///
/// 类型参数 BufferType 一般为 bit 字段所归属的结构体类型，这样可以保持一致性。
/// 也可以是其他结构体类型，这样多个结构体类型将拥有同一个字段，但彼此的位置和值类型可以互不相同。
pub trait Field<BufferType>
where
    BufferType: ?Sized,
{
    type ValueType;
}

/// # bit 字段写入器
///
/// bit 字段的实际写入函数，类型参数 BufferType 一般为 bit 字段归属的结构体类型。
pub trait FieldWriter<BufferType>: Field<BufferType>
where
    BufferType: ?Sized,
{
    fn write(buffer: &mut BufferType, value: Self::ValueType);
}
/// # bit 字段读出器
///
/// bit 字段的实际读出函数，类型参数 BufferType 一般为 bit 字段归属的结构体类型。
pub trait FieldReader<BufferType>: Field<BufferType>
where
    BufferType: ?Sized,
{
    fn read(buffer: &BufferType) -> Self::ValueType;
}
