#[derive(PartialEq, Debug)]
pub enum MessageCode {
    GetId = 0x0000,
    VersionRead = 0xF270,
    RwRegister = 0x2000,
    CustomerInfoRead = 0xF278,
    FwVersionGet = 0xF293,
    ErrorResponseEx = 0xFFFE,
    ErrorResponse = 0xFFFF,
}

impl TryFrom<u16> for MessageCode {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0000 => Ok(MessageCode::GetId),
            0x2000 => Ok(MessageCode::RwRegister),
            0xF270 => Ok(MessageCode::VersionRead),
            0xF278 => Ok(MessageCode::CustomerInfoRead),
            0xF293 => Ok(MessageCode::FwVersionGet),
            0xFFFE => Ok(MessageCode::ErrorResponseEx),
            0xFFFF => Ok(MessageCode::ErrorResponse),
            _ => Err(anyhow::anyhow!(
                "{}:{} Unkown type 0x{:04X}",
                file!(),
                line!(),
                value
            )),
        }
    }
}
