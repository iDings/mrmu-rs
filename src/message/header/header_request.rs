use std::usize;

use crate::message::MessageHeaderOperation;
use crate::message_builder::{MessageBuilder, MessageBuilderOperation};
use crate::message_code::MessageCode;

#[derive(Debug)]
pub struct RequestHeader {
    pub destination_address: [u8; 6],
    pub source_address: [u8; 6],
    pub ether_type: u16,
    pub device_id: u8,
    pub priority: u8,
    pub sequence_number: u8,
    pub length_type: u16,
    pub format: u16,
    pub code: u16,
}

impl RequestHeader {
    pub fn destination_address(&self) -> [u8; 6] {
        self.destination_address
    }
    pub fn set_destination_address(&mut self, addr: &[u8; 6]) -> &mut Self {
        self.destination_address[..].copy_from_slice(addr);
        self
    }

    pub fn source_address(&self) -> [u8; 6] {
        self.source_address
    }
    pub fn set_source_address(&mut self, addr: &[u8; 6]) -> &mut Self {
        self.source_address[..].copy_from_slice(addr);
        self
    }

    pub fn ether_type(&self) -> u16 {
        self.ether_type
    }
    pub fn set_ether_type(&mut self, val: u16) -> &mut Self {
        self.ether_type = val;
        self
    }

    pub fn device_id(&self) -> u8 {
        self.device_id
    }
    pub fn set_device_id(&mut self, val: u8) -> &mut Self {
        self.device_id = val;
        self
    }

    pub fn priority(&self) -> u8 {
        self.priority
    }
    pub fn set_priority(&mut self, val: u8) -> &mut Self {
        self.priority = val;
        self
    }

    pub fn sequence_number(&self) -> u8 {
        self.sequence_number
    }
    pub fn set_sequence_number(&mut self, val: u8) -> &mut Self {
        self.sequence_number = val;
        self
    }

    pub fn length_type(&self) -> u16 {
        self.length_type
    }
    pub fn set_length_type(&mut self, val: u16) -> &mut Self {
        self.length_type = val;
        self
    }

    pub fn format(&self) -> u16 {
        self.format
    }
    pub fn set_format(&mut self, val: u16) -> &mut Self {
        self.format = val;
        self
    }

    pub fn code(&self) -> u16 {
        self.code
    }
    pub fn set_code(&mut self, val: u16) -> &mut Self {
        self.code = val;
        self
    }
}

impl Default for RequestHeader {
    fn default() -> Self {
        RequestHeader {
            destination_address: [0; 6],
            source_address: [0; 6],
            ether_type: 0x9101,
            device_id: 0x00,
            priority: 0x00,
            sequence_number: 0x00,
            length_type: 0x0006,
            format: 0x0001,
            code: 0x0000,
        }
    }
}

impl MessageHeaderOperation for RequestHeader {
    type Output = Self;

    fn wire_size(&self) -> usize {
        28
    }

    fn marshal(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        buffer[0..6].copy_from_slice(&self.destination_address[..]);
        buffer[6..12].copy_from_slice(&self.source_address[..]);
        buffer[12..14].copy_from_slice(&self.ether_type.to_be_bytes());
        buffer[14..16].copy_from_slice(&0x0000_u16.to_be_bytes());
        buffer[16] = (self.device_id & 0b00011111) | 0x40;
        buffer[17] = 0xfa;
        buffer[18] = ((self.priority & 0b0000111) << 5) | 0x0f;
        buffer[19] = self.sequence_number;
        buffer[20..22].copy_from_slice(&self.length_type.to_be_bytes());
        buffer[22..24].copy_from_slice(&self.format.to_be_bytes());
        buffer[24..26].copy_from_slice(&0x0000_u16.to_be_bytes());
        buffer[26..28].copy_from_slice(&self.code.to_be_bytes());

        Ok(self.wire_size())
    }

    fn unmarshal(buffer: &[u8]) -> anyhow::Result<Self::Output> {
        let destination_address: [u8; 6] = buffer[0..6].try_into()?;
        let source_address: [u8; 6] = buffer[6..12].try_into()?;
        let ether_type = u16::from_be_bytes(buffer[12..14].try_into().unwrap());
        let device_id = buffer[16] & 0b00011111;
        let priority = (buffer[18] & 0b11100000) >> 5;
        let sequence_number = buffer[19];
        let length_type = u16::from_be_bytes(buffer[20..22].try_into().unwrap());
        let format = u16::from_be_bytes(buffer[22..24].try_into().unwrap());
        let code = u16::from_be_bytes(buffer[26..28].try_into().unwrap());

        Ok(RequestHeader {
            destination_address,
            source_address,
            ether_type,
            device_id,
            priority,
            sequence_number,
            length_type,
            format,
            code,
        })
    }

    fn message_code(&self) -> MessageCode {
        self.code.try_into().unwrap()
    }
}

impl MessageBuilder<RequestHeader> {
    pub fn destination_address(mut self, addr: &[u8; 6]) -> Self {
        self.inner.set_destination_address(addr);
        self
    }

    pub fn source_address(mut self, addr: &[u8; 6]) -> Self {
        self.inner.set_source_address(addr);
        self
    }

    pub fn ether_type(mut self, val: u16) -> Self {
        self.inner.set_ether_type(val);
        self
    }

    pub fn device_id(mut self, val: u8) -> Self {
        self.inner.set_device_id(val);
        self
    }

    pub fn priority(mut self, val: u8) -> Self {
        self.inner.set_priority(val);
        self
    }

    pub fn sequence_number(mut self, val: u8) -> Self {
        self.inner.set_sequence_number(val);
        self
    }

    pub fn length_type(mut self, val: u16) -> Self {
        self.inner.set_length_type(val);
        self
    }

    pub fn format(mut self, val: u16) -> Self {
        self.inner.set_format(val);
        self
    }

    pub fn code(mut self, val: u16) -> Self {
        self.inner.set_code(val);
        self
    }
}

impl MessageBuilderOperation for RequestHeader {
    fn finalize(self) -> anyhow::Result<Self> {
        Ok(self)
    }
}
