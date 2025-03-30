use std::ffi::CStr;

use crate::message_builder::MessageBuilderOperation;
use crate::{
    message::{header::ResponseHeader, MessageHeaderOperation, MessageOperation},
    message_builder::MessageBuilder,
    message_code::MessageCode,
};

const CODE: MessageCode = MessageCode::FwVersionGet;

pub struct FwVersionResponse {
    header: ResponseHeader,
    api_number: u16,
    variant_number: u16,
    release_number: u16,
    build_string: [u8; 256],
}

impl FwVersionResponse {
    pub fn api_number(&self) -> u16 {
        self.api_number
    }

    pub fn variant_number(&self) -> u16 {
        self.variant_number
    }

    pub fn release_number(&self) -> u16 {
        self.release_number
    }

    pub fn build_string(&self) -> String {
        let cstr = CStr::from_bytes_until_nul(&self.build_string).unwrap();
        String::from_utf8_lossy(cstr.to_bytes()).to_string()
    }

    pub fn payload_wire_size(&self) -> usize {
        let mut size = std::mem::size_of_val(&self.api_number);
        size += std::mem::size_of_val(&self.variant_number);
        size += std::mem::size_of_val(&self.build_string);

        size
    }

    pub fn set_api_number(&mut self, val: u16) -> &mut Self {
        self.api_number = val;
        self
    }

    pub fn set_variant_number(&mut self, val: u16) -> &mut Self {
        self.variant_number = val;
        self
    }

    pub fn set_release_number(&mut self, val: u16) -> &mut Self {
        self.release_number = val;
        self
    }

    pub fn set_build_string(&mut self, val: &str) -> &mut Self {
        let max_len = self.build_string.len();
        if val.len() >= max_len {
            self.build_string
                .copy_from_slice(&val.as_bytes()[..max_len - 1]);
            self.build_string[max_len - 1] = b'\0';
        } else {
            self.build_string.copy_from_slice(&val.as_bytes());
            self.build_string[val.len()] = b'\0';
        }

        self
    }
}

impl Default for FwVersionResponse {
    fn default() -> Self {
        Self {
            header: ResponseHeader {
                code: CODE as u16,
                ..Default::default()
            },
            api_number: 0,
            variant_number: 0,
            release_number: 0,
            build_string: [0; 256],
        }
    }
}

impl MessageOperation for FwVersionResponse {
    type Output = Self;
    type Header = ResponseHeader;

    fn message_code() -> MessageCode {
        CODE
    }

    fn wire_size(&self) -> usize {
        self.header.wire_size() + self.payload_wire_size()
    }

    fn marshal(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        let (hbuf, pbuf) = buffer.split_at_mut(self.header.wire_size());
        self.header.marshal(hbuf)?;

        pbuf[0..2].copy_from_slice(&self.api_number.to_be_bytes());
        pbuf[2..4].copy_from_slice(&self.variant_number.to_be_bytes());
        pbuf[4..6].copy_from_slice(&self.release_number.to_be_bytes());
        pbuf[6..262].copy_from_slice(&self.build_string);

        Ok(self.wire_size())
    }

    fn unmarshal(buffer: &[u8]) -> anyhow::Result<Self::Output> {
        let header = ResponseHeader::unmarshal(buffer)?;
        let (_, pbuf) = buffer.split_at(header.wire_size());

        Ok(Self {
            header,
            api_number: u16::from_be_bytes(pbuf[0..2].try_into().unwrap()),
            variant_number: u16::from_be_bytes(pbuf[2..4].try_into().unwrap()),
            release_number: u16::from_be_bytes(pbuf[4..6].try_into().unwrap()),
            build_string: pbuf[6..262].try_into().unwrap(),
        })
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn header_mut(&mut self) -> &mut Self::Header {
        &mut self.header
    }
}

impl TryFrom<ResponseHeader> for FwVersionResponse {
    type Error = anyhow::Error;

    fn try_from(value: ResponseHeader) -> Result<Self, Self::Error> {
        let code: MessageCode = value.code.try_into()?;
        if code != CODE {
            return Err(anyhow::anyhow!(
                "{}:{} message type mismatch {}",
                file!(),
                line!(),
                value.code,
            ));
        }

        Ok(Self {
            header: value,
            api_number: 0,
            variant_number: 0,
            release_number: 0,
            build_string: [0; 256],
        })
    }
}

impl MessageBuilder<FwVersionResponse> {
    pub fn api_number(mut self, val: u16) -> Self {
        self.inner.set_api_number(val);
        self
    }

    pub fn variant_number(mut self, val: u16) -> Self {
        self.inner.set_variant_number(val);
        self
    }

    pub fn release_number(mut self, val: u16) -> Self {
        self.inner.set_release_number(val);
        self
    }

    pub fn build_string(mut self, val: &str) -> Self {
        self.inner.set_build_string(val);
        self
    }
}

impl From<MessageBuilder<ResponseHeader>> for MessageBuilder<FwVersionResponse> {
    fn from(value: MessageBuilder<ResponseHeader>) -> Self {
        let header = value.code(CODE as u16).build().unwrap();
        Self {
            inner: header.try_into().unwrap(),
        }
    }
}

impl MessageBuilderOperation for FwVersionResponse {
    fn finalize(mut self) -> anyhow::Result<Self> {
        let len = self.header.length_type() + self.payload_wire_size() as u16;
        self.header.set_length_type(len);
        Ok(self)
    }
}
