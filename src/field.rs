/// ```
/// BufferType[data] {
///     field1 [position, r/w, type] {
///         input_converter: |x| {};
///         output_converter: |x| {}
///     },
///     field2 [position, r/w, type] {
///         input_converter: |x| {};
///         output_converter: |x| {},
///     },
///     field3 (position, r/w, type),
///     field4 (position, r/w, type)
/// }
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

/// Impl for RegBuffer::Regbuff if you want to config field.
pub trait BufferWriter {
    #[must_use = "The modified value works after flushed into register"]
    fn write<T>(&mut self, value: T::ValueType) -> &mut Self
    where
        T: Field<Self> + FieldWriter<Self>,
    {
        T::write(self, value);
        self
    }
}
/// impl for RegBuffer::Regbuff if you want to get field;
pub trait BufferReader {
    fn read<T: Field<Self> + FieldReader<Self>>(&self) -> T::ValueType {
        T::read(self)
    }
    fn output<T: Field<Self> + FieldReader<Self>>(&self, out: &mut T::ValueType) -> &Self {
        *out = T::read(self);
        self
    }
}

/// impl for Reg's fields;
/// RegFieldWrite and RegFieldRead use the same ValueType and Regbuff to keep consistent.
pub trait Field<BufferType>
where
    BufferType: ?Sized,
{
    type ValueType;
}

/// impl for RegField's instance
pub trait FieldWriter<BufferType>: Field<BufferType>
where
    BufferType: ?Sized,
{
    fn write(buffer: &mut BufferType, value: Self::ValueType);
}
/// impl for RegField's instance
pub trait FieldReader<BufferType>: Field<BufferType>
where
    BufferType: ?Sized,
{
    fn read(buffer: &BufferType) -> Self::ValueType;
}
