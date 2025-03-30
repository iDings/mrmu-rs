use core::fmt;

use bit_ops::bitops_u8;

use crate::message::register::OpType;
use crate::message::register::ReadWriteOpCode;
use crate::message::register::WaitOnBitOpCode;

const END_OF_LIST: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

#[derive(Debug)]
pub enum RegOpResponse {
    Read {
        addr: u8,
        reg: u8,
        data: u16,
    },
    Write {
        addr: u8,
        reg: u8,
        data: u16,
    },
    WaitOnBit1 {
        addr: u8,
        reg: u8,
        bit: u8,
        result: u8,
    },
    WaitOnBit0 {
        addr: u8,
        reg: u8,
        bit: u8,
        result: u8,
    },
    EndOfList,
}

#[derive(Debug)]
pub struct RegOpResponseList {
    inner: Vec<RegOpResponse>,
}

impl Default for RegOpResponseList {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl RegOpResponse {
    fn wire_size(&self) -> usize {
        4
    }

    fn marshal(&self, buf: &mut [u8]) -> anyhow::Result<usize> {
        match self {
            &RegOpResponse::Read { addr, reg, data } => {
                // name: index 3 starting from 0, length 2
                let addr_3_2 = bitops_u8::get_bits(addr, 2, 3);
                let optype = OpType::ReadWrite as u8;
                let opcode = ReadWriteOpCode::Read as u8;
                buf[0] = bitops_u8::set_bits_n(
                    0x00,
                    &[(optype, 4, 4), (opcode, 2, 2), (addr_3_2, 2, 0)],
                );

                let addr_0_3 = bitops_u8::get_bits(addr, 3, 0);
                let reg_0_5 = bitops_u8::get_bits(reg, 5, 0);
                buf[1] = bitops_u8::set_bits_n(0, &[(addr_0_3, 3, 5), (reg_0_5, 5, 0)]);

                buf[2..4].copy_from_slice(&data.to_be_bytes());
            }
            &RegOpResponse::Write { addr, reg, data } => {
                let addr_3_2 = bitops_u8::get_bits(addr, 2, 3);
                let optype = OpType::ReadWrite as u8;
                let opcode = ReadWriteOpCode::Write as u8;
                buf[0] = bitops_u8::set_bits_n(
                    0x00,
                    &[(optype, 4, 4), (opcode, 2, 2), (addr_3_2, 2, 0)],
                );

                let addr_0_3 = bitops_u8::get_bits(addr, 3, 0);
                let reg_0_5 = bitops_u8::get_bits(reg, 5, 0);
                buf[1] = bitops_u8::set_bits_n(0, &[(addr_0_3, 3, 5), (reg_0_5, 5, 0)]);

                buf[2..4].copy_from_slice(&data.to_be_bytes());
            }
            &RegOpResponse::WaitOnBit1 {
                addr,
                reg,
                bit,
                result,
            } => {
                let addr_3_2 = bitops_u8::get_bits(addr, 2, 3);
                let optype = OpType::WaitOnBit as u8;
                let opcode = WaitOnBitOpCode::Bit1 as u8;
                buf[0] = bitops_u8::set_bits_n(
                    0x00,
                    &[(optype, 4, 4), (opcode, 2, 2), (addr_3_2, 2, 0)],
                );

                let addr_0_3 = bitops_u8::get_bits(addr, 3, 0);
                let reg_0_5 = bitops_u8::get_bits(reg, 5, 0);
                buf[1] = bitops_u8::set_bits_n(0, &[(addr_0_3, 3, 5), (reg_0_5, 5, 0)]);

                let bit_4_0 = bitops_u8::get_bits(bit, 4, 0);
                buf[2] = bitops_u8::set_bits(0, bit_4_0, 4, 0);

                buf[3] = result;
            }
            &RegOpResponse::WaitOnBit0 {
                addr,
                reg,
                bit,
                result,
            } => {
                let addr_3_2 = bitops_u8::get_bits(addr, 2, 3);
                let optype = OpType::WaitOnBit as u8;
                let opcode = WaitOnBitOpCode::Bit0 as u8;
                buf[0] = bitops_u8::set_bits_n(
                    0x00,
                    &[(optype, 4, 4), (opcode, 2, 2), (addr_3_2, 2, 0)],
                );

                let addr_0_3 = bitops_u8::get_bits(addr, 3, 0);
                let reg_0_5 = bitops_u8::get_bits(reg, 5, 0);
                buf[1] = bitops_u8::set_bits_n(0, &[(addr_0_3, 3, 5), (reg_0_5, 5, 0)]);

                let bit_4_0 = bitops_u8::get_bits(bit, 4, 0);
                buf[2] = bitops_u8::set_bits(0, bit_4_0, 4, 0);

                buf[3] = result;
            }
            &RegOpResponse::EndOfList => buf.copy_from_slice(&END_OF_LIST),
        }

        Ok(self.wire_size())
    }

    fn unmarshal(buf: &[u8]) -> anyhow::Result<Self> {
        let optype = bitops_u8::get_bits(buf[0], 4, 4);
        let opcode = bitops_u8::get_bits(buf[0], 2, 2);
        let addr_3_2 = bitops_u8::get_bits(buf[0], 2, 0);
        let addr_0_3 = bitops_u8::get_bits(buf[1], 3, 5);
        let addr = (addr_3_2 << 3) | addr_0_3;
        let reg = bitops_u8::get_bits(buf[1], 5, 0);

        match optype.try_into() {
            Ok(OpType::ReadWrite) => {
                let data = u16::from_be_bytes(buf[2..4].try_into().unwrap());
                match opcode.try_into() {
                    Ok(ReadWriteOpCode::Write) => Ok(RegOpResponse::Write { addr, reg, data }),
                    Ok(ReadWriteOpCode::Read) => Ok(RegOpResponse::Read { addr, reg, data }),
                    Err(e) => Err(e),
                }
            }
            Ok(OpType::WaitOnBit) => {
                let bit = bitops_u8::get_bits(buf[2], 4, 0);
                let result = buf[3];
                match opcode.try_into() {
                    Ok(WaitOnBitOpCode::Bit0) => Ok(RegOpResponse::WaitOnBit0 {
                        addr,
                        reg,
                        bit,
                        result,
                    }),
                    Ok(WaitOnBitOpCode::Bit1) => Ok(RegOpResponse::WaitOnBit1 {
                        addr,
                        reg,
                        bit,
                        result,
                    }),
                    Err(e) => Err(e),
                }
            }
            Ok(OpType::EndOfList) => Ok(RegOpResponse::EndOfList),
            Err(e) => Err(e),
        }
    }
}

impl RegOpResponseList {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn wire_size(&self) -> usize {
        // +1 for end_of_list
        (self.inner.len() + 1) * 4
    }

    pub fn marshal(&self, buf: &mut [u8]) -> anyhow::Result<usize> {
        let mut offset = 0;
        for regop in &self.inner {
            offset += regop.marshal(&mut buf[offset..])?;
        }

        buf.copy_from_slice(&END_OF_LIST);
        Ok(self.wire_size())
    }

    pub fn unmarshal(buf: &[u8]) -> anyhow::Result<Self> {
        let len = buf.len();
        let mut offset = 0;
        let mut resplist = RegOpResponseList::new();

        loop {
            let regop = RegOpResponse::unmarshal(&buf[offset..])?;
            offset += regop.wire_size();

            match &regop {
                &RegOpResponse::WaitOnBit1 { result, .. }
                | &RegOpResponse::WaitOnBit0 { result, .. } => {
                    resplist.inner.push(regop);
                    if result == 0xFF {
                        break;
                    }
                }
                &RegOpResponse::Read { .. } | &RegOpResponse::Write { .. } => {
                    resplist.inner.push(regop)
                }
                &RegOpResponse::EndOfList => break,
            }

            if offset >= len {
                break;
            }
        }

        Ok(resplist)
    }
}

impl AsRef<Vec<RegOpResponse>> for RegOpResponseList {
    fn as_ref(&self) -> &Vec<RegOpResponse> {
        &self.inner
    }
}

impl AsMut<Vec<RegOpResponse>> for RegOpResponseList {
    fn as_mut(&mut self) -> &mut Vec<RegOpResponse> {
        &mut self.inner
    }
}

impl fmt::Display for RegOpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegOpResponse::Read { addr, reg, data } => {
                write!(
                    f,
                    "Read:addr=0x{:02x}, reg=0x{:02x}, data=0x{:04x}",
                    addr, reg, data
                )
            }
            RegOpResponse::Write { addr, reg, data } => {
                write!(
                    f,
                    "Write: addr=0x{:02x}, reg=0x{:02x}, data=0x{:04x}",
                    addr, reg, data
                )
            }
            RegOpResponse::WaitOnBit0 {
                addr,
                reg,
                bit,
                result,
            } => {
                write!(
                    f,
                    "WaitOnBit0: addr=0x{:02x}, reg=0x{:02x}, bit=0x{:02x}, result=0x{:02x}",
                    addr, reg, bit, result
                )
            }
            RegOpResponse::WaitOnBit1 {
                addr,
                reg,
                bit,
                result,
            } => {
                write!(
                    f,
                    "WaitOnBit1: addr=0x{:02x}, reg=0x{:02x}, bit=0x{:02x}, result=0x{:02x}",
                    addr, reg, bit, result
                )
            }
            RegOpResponse::EndOfList => {
                write!(f, "EndOfList")
            }
        }
    }
}

// @todo: pretty-print
impl fmt::Display for RegOpResponseList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RegOpRequestList {{[")?;
        for op in &self.inner {
            write!(f, " {} ", op)?;
        }
        write!(f, "]}}")
    }
}
