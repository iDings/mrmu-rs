use crate::message::MessageHeaderOperation;
use crate::message::{header::RequestHeader, MessageOperation};
use crate::message_builder::MessageBuilder;
use crate::message_builder::MessageBuilderOperation;
use crate::message_code::MessageCode;

use crate::message::register::RegOpRequestList;

use super::RegOpRequest;

const CODE: MessageCode = MessageCode::RwRegister;

#[derive(Debug)]
pub struct RegisterRequest {
    pub header: RequestHeader,
    pub regops: RegOpRequestList,
}

impl Default for RegisterRequest {
    fn default() -> Self {
        Self {
            header: RequestHeader {
                code: CODE as u16,
                ..Default::default()
            },
            regops: Default::default(),
        }
    }
}

impl MessageOperation for RegisterRequest {
    type Output = Self;
    type Header = RequestHeader;

    fn message_code() -> MessageCode {
        MessageCode::RwRegister
    }

    fn wire_size(&self) -> usize {
        self.header.wire_size() + self.regops.wire_size()
    }

    fn marshal(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        let (hbuf, pbuf) = buffer.split_at_mut(self.header.wire_size());
        let mut size = self.header.marshal(hbuf)?;
        size = size + self.regops.marshal(pbuf)?;

        Ok(size)
    }

    fn unmarshal(buffer: &[u8]) -> anyhow::Result<Self::Output> {
        let header = RequestHeader::unmarshal(buffer)?;
        let (_, pbuf) = buffer.split_at(header.wire_size());
        let regops = RegOpRequestList::unmarshal(pbuf)?;

        Ok(Self { header, regops })
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn header_mut(&mut self) -> &mut Self::Header {
        &mut self.header
    }
}

impl TryFrom<RequestHeader> for RegisterRequest {
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

        Ok(Self {
            header: value,
            regops: Default::default(),
        })
    }
}

impl MessageBuilder<RegisterRequest> {
    pub fn add_regop(mut self, regop: RegOpRequest) -> Self {
        self.inner.regops.add_regop(regop);
        self
    }

    pub fn regops(mut self, regops: RegOpRequestList) -> Self {
        self.inner.regops = regops;
        self
    }
}

impl From<MessageBuilder<RequestHeader>> for MessageBuilder<RegisterRequest> {
    fn from(value: MessageBuilder<RequestHeader>) -> Self {
        let header = value.code(CODE as u16).build().unwrap();
        Self {
            inner: header.try_into().unwrap(),
        }
    }
}

impl MessageBuilderOperation for RegisterRequest {
    fn finalize(mut self) -> anyhow::Result<Self> {
        let len = self.header.length_type() + self.regops.wire_size() as u16;
        self.header.set_length_type(len);
        Ok(self)
    }
}
