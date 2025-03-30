use bit_ops::bitops_u16;

mod global1_register;
mod port_register;

pub use port_register::PhysicalControl;
pub use port_register::PortRegister;
pub use port_register::PortSTatus;

#[derive(PartialEq)]
pub struct BitInfo {
    pub len: u8,
    pub shift: u8,
}

#[macro_export]
macro_rules! bitinfo_comb_flat {
    ($len: expr, $shift: expr) => {
        $len << 8 | $shift
    };
}

#[macro_export]
macro_rules! bitinfo_comb_len {
    ($comb: expr) => {
        ($comb >> 8) as u8
    };
}

#[macro_export]
macro_rules! bitinfo_comb_shift {
    ($comb: expr) => {
        ($comb & 0x00FF) as u8
    };
}

#[macro_export]
macro_rules! bitinfo_comb_deflat {
    ($comb: expr) => {
        BitInfo {
            len: ($comb >> 8) as u8,
            shift: ($comb & 0x00FF) as u8,
        }
    };
}

pub fn u16_get_bits<T: Into<BitInfo>>(base: u16, opaque: T) -> u16 {
    let info = opaque.into();
    bitops_u16::get_bits(base, info.len.into(), info.shift.into())
}

pub fn u16_set_bits<T: Into<BitInfo>>(base: u16, val: u16, opaque: T) -> u16 {
    let info = opaque.into();
    bitops_u16::set_bits(base, val, info.len.into(), info.shift.into())
}
