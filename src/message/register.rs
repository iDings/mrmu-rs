mod register_op_request;
mod register_op_response;
mod register_request;
mod register_response;

pub use register_op_request::RegOpRequest;
pub use register_op_request::RegOpRequestList;
pub use register_request::RegisterRequest;

#[allow(unused_imports)]
pub use register_op_response::RegOpResponse;

pub use register_op_response::RegOpResponseList;
pub use register_response::RegisterResponse;

// Functional_Specification.pdf 2.11.5.5 Read/Write Register
enum OpType {
    ReadWrite = 0x0,
    WaitOnBit = 0x1,
    EndOfList = 0xF,
}

enum ReadWriteOpCode {
    Read = 0x02,
    Write = 0x01,
}

enum WaitOnBitOpCode {
    Bit0 = 0x00,
    Bit1 = 0x03,
}

impl TryFrom<u8> for OpType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(OpType::ReadWrite),
            0x1 => Ok(OpType::WaitOnBit),
            0xF => Ok(OpType::EndOfList),
            _ => Err(anyhow::anyhow!(
                "{}:{} unkown optype:{}",
                file!(),
                line!(),
                value
            )),
        }
    }
}

impl TryFrom<u8> for ReadWriteOpCode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(ReadWriteOpCode::Write),
            0x02 => Ok(ReadWriteOpCode::Read),
            _ => Err(anyhow::anyhow!(
                "{}:{} unkown r/w opcode:{}",
                file!(),
                line!(),
                value
            )),
        }
    }
}

impl TryFrom<u8> for WaitOnBitOpCode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(WaitOnBitOpCode::Bit0),
            0x03 => Ok(WaitOnBitOpCode::Bit1),
            _ => Err(anyhow::anyhow!(
                "{}:{} unkown wait_on_bit opcode:{}",
                file!(),
                line!(),
                value
            )),
        }
    }
}
