use embedded_hal::i2c;

pub const PRINT_ADDRESS: u8 = 0x10;

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum I2cError {
    Success = 0,
    Nack = 1,
    ArbitrationLoss = 2,
}

impl i2c::Error for I2cError {
    fn kind(&self) -> i2c::ErrorKind {
        match self {
            I2cError::Nack => i2c::ErrorKind::NoAcknowledge(i2c::NoAcknowledgeSource::Unknown),
            I2cError::ArbitrationLoss => i2c::ErrorKind::ArbitrationLoss,
            I2cError::Success => i2c::ErrorKind::Other,
        }
    }
}

impl From<u32> for I2cError {
    fn from(value: u32) -> Self {
        match value {
            0 => I2cError::Success,
            1 => I2cError::Nack,
            2 => I2cError::ArbitrationLoss,
            _ => I2cError::Success,
        }
    }
}
