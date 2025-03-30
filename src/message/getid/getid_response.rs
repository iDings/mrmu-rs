use crate::message::MessageHeaderOperation;
use crate::message::{header::ResponseHeader, MessageOperation};
use crate::message_builder::MessageBuilder;
use crate::message_builder::MessageBuilderOperation;
use crate::message_code::MessageCode;

#[derive(Debug)]
pub struct GetIdResponse {
    header: ResponseHeader,
}

impl Default for GetIdResponse {
    fn default() -> Self {
        GetIdResponse {
            header: Default::default(),
        }
    }
}

impl MessageOperation for GetIdResponse {
    type Output = Self;
    type Header = ResponseHeader;

    fn message_code() -> MessageCode {
        MessageCode::GetId
    }

    fn wire_size(&self) -> usize {
        self.header.wire_size() + 0
    }

    fn marshal(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        self.header.marshal(buffer)
    }

    fn unmarshal(buffer: &[u8]) -> anyhow::Result<Self::Output> {
        let header = ResponseHeader::unmarshal(buffer)?;
        Ok(Self { header })
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn header_mut(&mut self) -> &mut Self::Header {
        &mut self.header
    }
}

impl TryFrom<ResponseHeader> for GetIdResponse {
    type Error = anyhow::Error;

    fn try_from(value: ResponseHeader) -> Result<Self, Self::Error> {
        let msgcode: MessageCode = value.code.try_into()?;
        if msgcode != MessageCode::GetId {
            return Err(anyhow::anyhow!(
                "{}:{} message type mismatch {}",
                file!(),
                line!(),
                value.code
            ));
        }

        Ok(Self { header: value })
    }
}

impl From<MessageBuilder<ResponseHeader>> for MessageBuilder<GetIdResponse> {
    fn from(value: MessageBuilder<ResponseHeader>) -> Self {
        let header = value.code(0x0000).build().unwrap();
        Self {
            inner: header.try_into().unwrap(),
        }
    }
}

impl MessageBuilderOperation for GetIdResponse {
    fn finalize(self) -> anyhow::Result<Self> {
        Ok(self)
    }
}
