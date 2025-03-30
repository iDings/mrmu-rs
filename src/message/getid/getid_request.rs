use crate::message::header::RequestHeader;
use crate::message::MessageHeaderOperation;
use crate::message::MessageOperation;
use crate::message_builder::MessageBuilder;
use crate::message_builder::MessageBuilderOperation;
use crate::message_code::MessageCode;

#[derive(Debug)]
pub struct GetIdRequest {
    header: RequestHeader,
}

impl Default for GetIdRequest {
    fn default() -> Self {
        GetIdRequest {
            header: RequestHeader {
                format: 0x0000,
                ..Default::default()
            },
        }
    }
}

impl MessageOperation for GetIdRequest {
    type Output = Self;
    type Header = RequestHeader;

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
        let header = RequestHeader::unmarshal(buffer)?;
        Ok(GetIdRequest { header })
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn header_mut(&mut self) -> &mut Self::Header {
        &mut self.header
    }
}

impl TryFrom<RequestHeader> for GetIdRequest {
    type Error = anyhow::Error;

    fn try_from(value: RequestHeader) -> Result<Self, Self::Error> {
        let code: MessageCode = value.code.try_into()?;
        if code != MessageCode::GetId {
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

impl From<MessageBuilder<RequestHeader>> for MessageBuilder<GetIdRequest> {
    fn from(value: MessageBuilder<RequestHeader>) -> Self {
        let header = value.code(0x0000).build().unwrap();
        Self {
            inner: header.try_into().unwrap(),
        }
    }
}

impl MessageBuilderOperation for GetIdRequest {
    fn finalize(self) -> anyhow::Result<Self> {
        Ok(self)
    }
}
