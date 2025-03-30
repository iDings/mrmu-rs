use crate::{
    message::{header::ResponseHeader, MessageHeaderOperation, MessageOperation},
    message_builder::{MessageBuilder, MessageBuilderOperation},
    message_code::MessageCode,
};

use super::RegOpResponseList;

const CODE: MessageCode = MessageCode::RwRegister;

#[derive(Debug)]
pub struct RegisterResponse {
    pub header: ResponseHeader,
    pub regops: RegOpResponseList,
}

impl Default for RegisterResponse {
    fn default() -> Self {
        Self {
            header: ResponseHeader {
                code: CODE as u16,
                ..Default::default()
            },
            regops: Default::default(),
        }
    }
}

impl MessageOperation for RegisterResponse {
    type Output = Self;
    type Header = ResponseHeader;

    fn message_code() -> MessageCode {
        MessageCode::RwRegister
    }

    fn wire_size(&self) -> usize {
        self.header.wire_size() + self.regops.wire_size()
    }

    fn marshal(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        let (hbuf, pbuf) = buffer.split_at_mut(self.header.wire_size());
        let mut size = self.header.marshal(hbuf)?;
        size += self.regops.marshal(pbuf)?;

        Ok(size)
    }

    fn unmarshal(buffer: &[u8]) -> anyhow::Result<Self::Output> {
        let header = ResponseHeader::unmarshal(buffer)?;
        let (_, pbuf) = buffer.split_at(header.wire_size());
        let regops = RegOpResponseList::unmarshal(pbuf)?;

        Ok(Self { header, regops })
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn header_mut(&mut self) -> &mut Self::Header {
        &mut self.header
    }
}

impl TryFrom<ResponseHeader> for RegisterResponse {
    type Error = anyhow::Error;

    fn try_from(value: ResponseHeader) -> Result<Self, Self::Error> {
        let code: MessageCode = value.code.try_into()?;
        if code != CODE {
            return Err(anyhow::anyhow!(
                "{}:{} code mismatch {}",
                file!(),
                line!(),
                value.code
            ));
        }

        Ok(Self {
            header: value,
            regops: Default::default(),
        })
    }
}

impl From<MessageBuilder<ResponseHeader>> for MessageBuilder<RegisterResponse> {
    fn from(value: MessageBuilder<ResponseHeader>) -> Self {
        let header = value.code(CODE as u16).build().unwrap();
        Self {
            inner: header.try_into().unwrap(),
        }
    }
}

impl MessageBuilderOperation for RegisterResponse {
    fn finalize(mut self) -> anyhow::Result<Self> {
        let len = self.header.length_type() + self.regops.wire_size() as u16;
        self.header.set_length_type(len);
        Ok(self)
    }
}
