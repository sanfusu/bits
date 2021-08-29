整型的相关 bit 操作，如：

```rust
use bits::BitsOps;
use bits::IntoBits;
assert_eq!(0u8.bits(0).set(), 0x01);
assert_eq!(0u8.bits(1).set(), 0x02);
assert_eq!(0u8.bits(4..=7).set(), 0xf0);
assert_eq!(0xffu8.bits(4..=7).clr(), 0x0f);
assert_eq!(0xffu8.bits(3).revert(), 0xf7);
assert_eq!(0xffu8.bits(4..=7).revert(), 0x0f);
assert_eq!(0u8.bits(4..=7).write(0x10), 0x0);
// 只会写入 value 的相应的 bit 位。低 4 bit 并不会被修改。
assert_eq!(0u8.bits(4..=7).write(0x12), 0x20);
assert_eq!(0x12u8.bits(4..=7).read(), 0x1);
```

在 BitsOps 接口的基础上添加了结构体 bit 字段相关的接口，以及辅助实现的宏：

```rust
use bits::field::BufferWriter;
use bits::field::BufferReader;
use bits::IntoBits;
use bits::BitsOps;

pub struct FoolData {
    data: u32,
    data1: u32,
}
impl BufferWriter for FoolData{}
impl BufferReader for FoolData{}

// bit 字段 1，其余类推
pub struct Flag1;
pub struct Flag2;
pub struct Flag3;
pub struct Flag4;
pub struct Flag5;
pub struct Flag6;

bits::fields! {
    FoolData [data] {
        Flag1 [0..=3, rw, u32],
        Flag2 [4..=5, rw, u32],
        Flag3 [6, ro, bool],
        Flag4 [7, rw, bool],
        Flag5 [8..=9, rw, bool] {
            input_converter: |x| match x {
                true => 0x1,
                _ => 0x0
            };
            output_converter: |x| match x {
                0x1 => true,
                _ => false
            }
        }
    }
    FoolData [data1] {
        Flag6 [0..=3, rw, u32]
    }
}
let mut fool = FoolData {data:0x0, data1: 0x0};
fool.write::<Flag1>(0xf);
assert_eq!(fool.data, 0xf);

fool.write::<Flag2>(0x3);
assert_eq!(fool.data, 0b0011_1111);

// error: the trait `FieldWriter<FoolData>` is not implemented for `Flag3`
// fool.write::<Flag3>(true); // Flag3 is not writeable

fool.write::<Flag4>(true);
assert_eq!(fool.data, 0b1011_1111);

let flag3 = fool.read::<Flag3>();
assert_eq!(flag3, false);

let flag4 = fool.read::<Flag4>();
assert_eq!(flag4, true);

fool.data = fool.data.bits(8..=9).write(0x2); // set fool.data bits 8..=9 to 0x2
assert_eq!(false, fool.read::<Flag5>()); // bits: 8..=9 equal to 0x2 which is false

fool.write::<Flag5>(true);
assert_eq!(0b01, fool.data.bits(8..=9).read());
```
