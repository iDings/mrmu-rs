use crate::message::header::RequestHeader;
use crate::message::{MessageHeaderOperation, MessageOperation};
use crate::message_builder::MessageBuilder;
use crate::message_builder::MessageBuilderOperation;
use crate::message_code::MessageCode;

const CODE: MessageCode = MessageCode::FwVersionGet;

#[derive(Debug)]
pub struct FwVersionRequest {
    header: RequestHeader,
}

impl Default for FwVersionRequest {
    fn default() -> Self {
        Self {
            header: RequestHeader {
                code: CODE as u16,
                ..Default::default()
            },
        }
    }
}

impl MessageOperation for FwVersionRequest {
    type Output = Self;
    type Header = RequestHeader;

    fn message_code() -> MessageCode {
        CODE
    }

    fn wire_size(&self) -> usize {
        self.header.wire_size()
    }

    fn marshal(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        self.header.marshal(buffer)
    }

    fn unmarshal(buffer: &[u8]) -> anyhow::Result<Self::Output> {
        let header = RequestHeader::unmarshal(buffer)?;
        Ok(Self { header })
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn header_mut(&mut self) -> &mut Self::Header {
        &mut self.header
    }
}

impl TryFrom<RequestHeader> for FwVersionRequest {
    type Error = anyhow::Error;

    fn try_from(value: RequestHeader) -> Result<Self, Self::Error> {
        let code: MessageCode = value.code.try_into()?;
        if code != CODE {
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

impl From<MessageBuilder<RequestHeader>> for MessageBuilder<FwVersionRequest> {
    fn from(value: MessageBuilder<RequestHeader>) -> Self {
        let header = value.code(CODE as u16).build().unwrap();
        Self {
            inner: header.try_into().unwrap(),
        }
    }
}

impl MessageBuilderOperation for FwVersionRequest {
    fn finalize(self) -> anyhow::Result<Self> {
        Ok(self)
    }
}
