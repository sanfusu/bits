use crate::Bitalized;

pub trait Register: Bitalized {
    type CacheType;
    unsafe fn from_raw(raw: Self::BaseType) -> Self::CacheType;
    fn to_raw(self) -> Self::BaseType;
}

/// bus 可以是内存数据总线，也可以是外围通信总线。
/// 将寄存器作为 trait 参数，这样在实现的时候可以为具体的寄存器定制读写方式。
/// 如果寄存器有通用的 cache 方式，则可以另行实现（定义）具体 Bus 相关的寄存器 trait，如 I2cReg.
/// 我们需要一个 macro，去辅助定义这样的 bus
/// ```
/// #[bus]
/// pub struct I2cBus {
///     pub Reg1: Reg1Cache;
///     pub Reg2: Reg2Cache;
/// }
/// ```
/// 这里我们我们需要生成 Reg1 和 Reg2 寄存器。
/// 这种方式，我们也很好的规定了 I2cBus 和其所拥有的寄存器之间的所属关系。
pub trait Bus<T: Register> {
    fn cache(reg: T) -> T::CacheType;
    fn flush(cache: T::CacheType);
}

pub struct A;
impl Bitalized for A {
    type BaseType = u32;
}
impl Register for A {
    type CacheType = u32;

    unsafe fn from_raw(_raw: Self::BaseType) -> Self::CacheType {
        todo!()
    }

    fn to_raw(self) -> Self::BaseType {
        todo!()
    }
}
pub struct I2C;
impl Bus<A> for I2C {
    // 我们明确知道在 I2C 总线下缓存寄存器 A
    // 具体实现中可能是通过 I2C 通信协议去读取从设备的寄存器 A。通信协议的具体实现不在讨论范围内。
    fn cache(_reg: A) -> <A as Register>::CacheType {
        todo!()
    }

    fn flush(_cache: <A as Register>::CacheType) {
        todo!()
    }
}

// pub trait I2CReg {
//     const OFFSET: u32;
// }
// pub struct B;
// impl Register for B {
//     type CacheType = u32;
// }
// impl I2CReg for B {
//     const OFFSET: u32 = 0;
// }
// impl Bus<B> for I2C {
//     fn cache(_reg: B) -> <B as Register>::CacheType {
//         B::OFFSET
//     }

//     fn flush(_cache: <B as Register>::CacheType) {
//         todo!()
//     }
// }
