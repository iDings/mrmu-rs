use std::ffi::CStr;
use std::ffi::CString;

use anyhow::Context;

use crate::message::MessageHeaderOperation;
use crate::message::{header::ResponseHeader, MessageOperation};
use crate::message_builder::MessageBuilder;
use crate::message_builder::MessageBuilderOperation;
use crate::message_code::MessageCode;

pub struct CustomerInfoReadResponse {
    header: ResponseHeader,
    pub info: CString,
}

const CODE: MessageCode = MessageCode::CustomerInfoRead;

impl CustomerInfoReadResponse {
    pub fn payload_wire_size(&self) -> usize {
        32
    }
}

impl Default for CustomerInfoReadResponse {
    fn default() -> Self {
        Self {
            header: ResponseHeader {
                code: 0xF278,
                ..Default::default()
            },
            info: Default::default(),
        }
    }
}

impl MessageOperation for CustomerInfoReadResponse {
    type Output = Self;
    type Header = ResponseHeader;

    fn message_code() -> MessageCode {
        MessageCode::CustomerInfoRead
    }

    fn wire_size(&self) -> usize {
        self.header.wire_size() + self.payload_wire_size()
    }

    fn marshal(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        let (hbuf, pbuf) = buffer.split_at_mut(self.header.wire_size());
        let _size = self.header.marshal(hbuf)?;

        let sbytes = self.info.to_bytes_with_nul();
        if sbytes.len() > 32 {
            pbuf[..31].copy_from_slice(&sbytes[..31]);
            pbuf[31] = b'\0';
        } else {
            pbuf.copy_from_slice(sbytes);
        }

        Ok(self.wire_size())
    }

    fn unmarshal(buffer: &[u8]) -> anyhow::Result<Self::Output> {
        let header = ResponseHeader::unmarshal(buffer).context("unmarshal fail")?;
        let (_, pbuf) = buffer.split_at(header.wire_size());
        let info: CString = CStr::from_bytes_until_nul(pbuf).unwrap().into();

        Ok(Self { header, info })
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn header_mut(&mut self) -> &mut Self::Header {
        &mut self.header
    }
}

impl TryFrom<ResponseHeader> for CustomerInfoReadResponse {
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
            info: Default::default(),
        })
    }
}

impl MessageBuilder<CustomerInfoReadResponse> {
    pub fn info(mut self, val: &str) -> Self {
        self.inner.info = CString::new(val).expect("CString::new fail");
        self
    }
}

impl MessageBuilderOperation for CustomerInfoReadResponse {
    fn finalize(mut self) -> anyhow::Result<Self> {
        let mut len = self.header.length_type();
        len += self.payload_wire_size() as u16;
        self.header.set_length_type(len);
        Ok(self)
    }
}

impl From<MessageBuilder<ResponseHeader>> for MessageBuilder<CustomerInfoReadResponse> {
    fn from(value: MessageBuilder<ResponseHeader>) -> Self {
        let header = value.code(0xF278).build().unwrap();
        Self {
            inner: header.try_into().unwrap(),
        }
    }
}
