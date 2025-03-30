use bit_ops::bitops_u8;
use clap::Subcommand;

use crate::message::register::OpType;
use crate::message::register::ReadWriteOpCode;
use crate::message::register::WaitOnBitOpCode;

const END_OF_LIST: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

#[derive(Clone, Debug, PartialEq, Subcommand)]
pub enum RegOpRequest {
    Read { addr: u8, reg: u8 },
    Write { addr: u8, reg: u8, data: u16 },
    WaitOnBit1 { addr: u8, reg: u8, bit: u8 },
    WaitOnBit0 { addr: u8, reg: u8, bit: u8 },
    EndOfList,
}

#[derive(Debug, Clone)]
pub struct RegOpRequestList {
    inner: Vec<RegOpRequest>,
}

impl Default for RegOpRequestList {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl RegOpRequest {
    fn wire_size(&self) -> usize {
        4
    }

    fn marshal(&self, buf: &mut [u8]) -> anyhow::Result<usize> {
        match self {
            &RegOpRequest::Read { addr, reg } => {
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

                buf[2..4].copy_from_slice(&0x0000_u16.to_be_bytes());
            }
            &RegOpRequest::Write { addr, reg, data } => {
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
            &RegOpRequest::WaitOnBit0 { addr, reg, bit } => {
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

                buf[3] = 0x00;
            }
            &RegOpRequest::WaitOnBit1 { addr, reg, bit } => {
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

                buf[3] = 0x00;
            }
            &RegOpRequest::EndOfList => {
                buf.copy_from_slice(&END_OF_LIST);
            }
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
            // read/write
            Ok(OpType::ReadWrite) => {
                let data = u16::from_be_bytes(buf[2..4].try_into().unwrap());
                match opcode.try_into() {
                    Ok(ReadWriteOpCode::Write) => Ok(RegOpRequest::Write { addr, reg, data }),
                    Ok(ReadWriteOpCode::Read) => Ok(RegOpRequest::Read { addr, reg }),
                    Err(e) => Err(e),
                }
            }
            Ok(OpType::WaitOnBit) => {
                let bit = bitops_u8::get_bits(buf[2], 4, 0);
                assert_eq!(buf[3], 0x00);

                match opcode.try_into() {
                    Ok(WaitOnBitOpCode::Bit0) => Ok(RegOpRequest::WaitOnBit0 { addr, reg, bit }),
                    Ok(WaitOnBitOpCode::Bit1) => Ok(RegOpRequest::WaitOnBit1 { addr, reg, bit }),
                    Err(e) => Err(e),
                }
            }
            Ok(OpType::EndOfList) => Ok(RegOpRequest::EndOfList),
            Err(e) => Err(e),
        }
    }
}

impl RegOpRequestList {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_regop(&mut self, regop: RegOpRequest) {
        self.inner.push(regop);
    }

    pub fn wire_size(&self) -> usize {
        // +1 for end_of_list
        (self.inner.len() + 1) * 4
    }

    pub fn marshal(&self, buf: &mut [u8]) -> anyhow::Result<usize> {
        let mut offset = 0;
        for op in &self.inner {
            offset += op.marshal(&mut buf[offset..])?;
        }

        buf[offset..offset + 4].copy_from_slice(&END_OF_LIST);
        Ok(self.wire_size())
    }

    pub fn unmarshal(buf: &[u8]) -> anyhow::Result<RegOpRequestList> {
        let mut reqlist = RegOpRequestList::new();
        let len = buf.len();
        let mut offset = 0;

        loop {
            let regop = RegOpRequest::unmarshal(&buf[offset..])?;
            offset += regop.wire_size();

            if regop == RegOpRequest::EndOfList {
                break;
            }

            reqlist.inner.push(regop);
            if offset >= len {
                break;
            }
        }

        Ok(reqlist)
    }
}

impl AsRef<Vec<RegOpRequest>> for RegOpRequestList {
    fn as_ref(&self) -> &Vec<RegOpRequest> {
        &self.inner
    }
}

impl AsMut<Vec<RegOpRequest>> for RegOpRequestList {
    fn as_mut(&mut self) -> &mut Vec<RegOpRequest> {
        &mut self.inner
    }
}
