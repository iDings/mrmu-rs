use crate::message::MessageHeaderOperation;
use crate::message::{header::RequestHeader, MessageOperation};
use crate::message_builder::{MessageBuilder, MessageBuilderOperation};
use crate::message_code::MessageCode;

const CODE: MessageCode = MessageCode::VersionRead;

#[derive(Debug)]
pub struct VersionReadRequest {
    header: RequestHeader,
}

impl Default for VersionReadRequest {
    fn default() -> Self {
        Self {
            header: RequestHeader {
                code: 0xF270,
                ..Default::default()
            },
        }
    }
}

impl MessageOperation for VersionReadRequest {
    type Output = Self;
    type Header = RequestHeader;

    fn message_code() -> MessageCode {
        MessageCode::VersionRead
    }

    fn wire_size(&self) -> usize {
        self.header.wire_size() + 0
    }

    fn marshal(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        self.header.marshal(buffer)
    }

    fn unmarshal(buffer: &[u8]) -> anyhow::Result<Self::Output> {
        let header = RequestHeader::unmarshal(buffer)?;
        Ok(VersionReadRequest { header })
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn header_mut(&mut self) -> &mut Self::Header {
        &mut self.header
    }
}

impl TryFrom<RequestHeader> for VersionReadRequest {
    type Error = anyhow::Error;

    fn try_from(value: RequestHeader) -> Result<Self, Self::Error> {
        let code: MessageCode = value.code.try_into()?;
        if code != MessageCode::VersionRead {
            return Err(anyhow::anyhow!(
                "{}:{} message type mismatch {}",
                file!(),
                line!(),
                value.code,
            ));
        }

        Ok(Self { header: value })
    }
}

impl From<MessageBuilder<RequestHeader>> for MessageBuilder<VersionReadRequest> {
    fn from(value: MessageBuilder<RequestHeader>) -> Self {
        let header = value.code(CODE as u16).build().unwrap();
        Self {
            inner: header.try_into().unwrap(),
        }
    }
}

impl MessageBuilderOperation for VersionReadRequest {
    fn finalize(self) -> anyhow::Result<Self> {
        Ok(self)
    }
}
