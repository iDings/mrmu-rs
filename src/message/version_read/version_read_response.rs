use std::u16;

use crate::message::MessageHeaderOperation;
use crate::message_builder::{MessageBuilder, MessageBuilderOperation};
use crate::{
    message::{header::ResponseHeader, MessageOperation},
    message_code::MessageCode,
};

const CODE: MessageCode = MessageCode::VersionRead;

#[derive(Debug)]
pub struct VersionReadResponse {
    header: ResponseHeader,
    pub crc32: u32,
}

impl VersionReadResponse {
    pub fn payload_wire_size(&self) -> usize {
        std::mem::size_of::<u32>()
    }
}

impl Default for VersionReadResponse {
    fn default() -> Self {
        Self {
            header: ResponseHeader {
                code: 0xF270,
                ..Default::default()
            },
            crc32: Default::default(),
        }
    }
}

impl MessageOperation for VersionReadResponse {
    type Output = Self;
    type Header = ResponseHeader;

    fn message_code() -> MessageCode {
        MessageCode::VersionRead
    }

    fn wire_size(&self) -> usize {
        self.header.wire_size() + self.payload_wire_size()
    }

    fn marshal(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        let (hbuf, pbuf) = buffer.split_at_mut(self.header.wire_size());
        self.header.marshal(hbuf)?;
        pbuf[0..4].copy_from_slice(&self.crc32.to_be_bytes());
        Ok(self.wire_size())
    }

    fn unmarshal(buffer: &[u8]) -> anyhow::Result<Self::Output> {
        let header = ResponseHeader::unmarshal(buffer)?;
        let (_, pbuf) = buffer.split_at(header.wire_size());
        // @todo: 1. check code 2. check with type_length
        let crc32 = u32::from_be_bytes(pbuf[..4].try_into().unwrap());

        Ok(Self { header, crc32 })
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn header_mut(&mut self) -> &mut Self::Header {
        &mut self.header
    }
}

impl TryFrom<ResponseHeader> for VersionReadResponse {
    type Error = anyhow::Error;

    fn try_from(value: ResponseHeader) -> Result<Self, Self::Error> {
        let code: MessageCode = value.code.try_into()?;
        if code != MessageCode::VersionRead {
            return Err(anyhow::anyhow!(
                "{}:{} message type mismatch {}",
                file!(),
                line!(),
                value.code
            ));
        }

        Ok(Self {
            header: value,
            crc32: Default::default(),
        })
    }
}

impl From<MessageBuilder<ResponseHeader>> for MessageBuilder<VersionReadResponse> {
    fn from(value: MessageBuilder<ResponseHeader>) -> Self {
        let header = value.code(CODE as u16).build().unwrap();
        Self {
            inner: header.try_into().unwrap(),
        }
    }
}

impl MessageBuilderOperation for VersionReadResponse {
    fn finalize(mut self) -> anyhow::Result<Self> {
        let len = self.header.length_type() + self.payload_wire_size() as u16;
        self.header.set_length_type(len);
        Ok(self)
    }
}
