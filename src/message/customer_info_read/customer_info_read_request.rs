use crate::message::MessageHeaderOperation;
use crate::message_builder::MessageBuilder;
use crate::message_builder::MessageBuilderOperation;
use crate::{
    message::{header::RequestHeader, MessageOperation},
    message_code::MessageCode,
};

#[derive(Debug)]
pub struct CustomerInfoReadRequest {
    pub header: RequestHeader,
}

const CODE: MessageCode = MessageCode::CustomerInfoRead;

impl Default for CustomerInfoReadRequest {
    fn default() -> Self {
        Self {
            header: RequestHeader {
                code: 0xF278,
                ..Default::default()
            },
        }
    }
}

impl MessageOperation for CustomerInfoReadRequest {
    type Output = Self;
    type Header = RequestHeader;

    fn message_code() -> MessageCode {
        MessageCode::CustomerInfoRead
    }

    fn wire_size(&self) -> usize {
        self.header.wire_size() + 0
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

impl TryFrom<RequestHeader> for CustomerInfoReadRequest {
    type Error = anyhow::Error;

    fn try_from(value: RequestHeader) -> Result<Self, Self::Error> {
        let code: MessageCode = value.code.try_into()?;
        if code != CODE {
            return Err(anyhow::anyhow!(
                "{}:{} code mismatch {}",
                file!(),
                line!(),
                value.code
            ));
        }

        Ok(Self { header: value })
    }
}

impl MessageBuilderOperation for CustomerInfoReadRequest {
    fn finalize(self) -> anyhow::Result<Self> {
        Ok(self)
    }
}

impl From<MessageBuilder<RequestHeader>> for MessageBuilder<CustomerInfoReadRequest> {
    fn from(value: MessageBuilder<RequestHeader>) -> Self {
        let header = value.code(0xF278).build().unwrap();
        Self {
            inner: header.try_into().unwrap(),
        }
    }
}
