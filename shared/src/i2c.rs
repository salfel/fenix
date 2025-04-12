use embedded_hal::i2c;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum I2cError {
    Nack,
    ArbitrationLoss,
    Overrun,
    Bus,
}

impl i2c::Error for I2cError {
    fn kind(&self) -> i2c::ErrorKind {
        match self {
            I2cError::Nack => i2c::ErrorKind::NoAcknowledge(i2c::NoAcknowledgeSource::Unknown),
            I2cError::ArbitrationLoss => i2c::ErrorKind::ArbitrationLoss,
            I2cError::Overrun => i2c::ErrorKind::Overrun,
            I2cError::Bus => i2c::ErrorKind::Bus,
        }
    }
}
