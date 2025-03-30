pub mod customer_info_read;
pub mod fw_version;
pub mod getid;
pub mod header;
pub mod register;
pub mod version_read;

use std::usize;

use crate::message_code::MessageCode;

pub struct Message<P> {
    pub payload: P,
}

pub trait MessageHeaderOperation {
    type Output: MessageHeaderOperation;

    fn wire_size(&self) -> usize;
    fn marshal(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize>;
    fn unmarshal(buffer: &[u8]) -> anyhow::Result<Self::Output>;

    fn message_code(&self) -> MessageCode;
}

pub trait MessageOperation {
    type Output: MessageOperation;
    type Header: MessageHeaderOperation;

    fn message_code() -> MessageCode;

    fn wire_size(&self) -> usize;
    fn marshal(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize>;
    fn unmarshal(buffer: &[u8]) -> anyhow::Result<Self::Output>;

    fn header(&self) -> &Self::Header;
    fn header_mut(&mut self) -> &mut Self::Header;
}

pub fn marshal<T: MessageOperation>(message: &mut T, buf: &mut [u8]) -> anyhow::Result<usize> {
    message.marshal(buf)
}

pub fn unmarshal<T: MessageOperation<Output = T>>(buffer: &[u8]) -> anyhow::Result<T> {
    let header = T::Header::unmarshal(buffer)?;
    let code = header.message_code();
    if code != T::message_code() {
        match code {
            MessageCode::ErrorResponse => {
                todo!()
            }
            MessageCode::ErrorResponseEx => {
                todo!()
            }
            _ => return Err(anyhow::anyhow!("code:{} mismatch", code as u16)),
        }
    }

    Ok(T::unmarshal(buffer)?)
}

pub fn prealloc_buffer(msg: &impl MessageOperation) -> Vec<u8> {
    let mut buf = Vec::new();

    let len = if msg.wire_size() > 60 {
        msg.wire_size()
    } else {
        60
    };

    buf.resize(len, 0);
    buf
}
