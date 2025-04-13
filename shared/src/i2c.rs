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
    /// Returns the i2c::ErrorKind corresponding to this I2cError variant.
    ///
    /// Converts the current error variant into an i2c::ErrorKind value:
    /// - `I2cError::Nack` maps to `i2c::ErrorKind::NoAcknowledge` with an unknown source.
    /// - `I2cError::ArbitrationLoss` maps to `i2c::ErrorKind::ArbitrationLoss`.
    /// - `I2cError::Success` maps to `i2c::ErrorKind::Other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use shared::i2c::I2cError;
    /// use embedded_hal::i2c;
    ///
    /// let error = I2cError::Nack;
    /// assert_eq!(error.kind(), i2c::ErrorKind::NoAcknowledge(i2c::NoAcknowledgeSource::Unknown));
    /// ```
    fn kind(&self) -> i2c::ErrorKind {
        match self {
            I2cError::Nack => i2c::ErrorKind::NoAcknowledge(i2c::NoAcknowledgeSource::Unknown),
            I2cError::ArbitrationLoss => i2c::ErrorKind::ArbitrationLoss,
            I2cError::Success => i2c::ErrorKind::Other,
        }
    }
}

impl From<u32> for I2cError {
    /// Converts a u32 value into its corresponding I2cError variant.
    ///
    /// The conversion maps specific numeric values to their related I2cError variants:
    /// - `0` becomes `I2cError::Success`
    /// - `1` becomes `I2cError::Nack`
    /// - `2` becomes `I2cError::ArbitrationLoss`
    ///
    /// Any other value defaults to `I2cError::Success`.
    ///
    /// # Examples
    ///
    /// ```
    /// use shared::i2c::I2cError;
    ///
    /// assert_eq!(I2cError::from(0), I2cError::Success);
    /// assert_eq!(I2cError::from(1), I2cError::Nack);
    /// assert_eq!(I2cError::from(2), I2cError::ArbitrationLoss);
    /// assert_eq!(I2cError::from(42), I2cError::Success);
    /// ```
    fn from(value: u32) -> Self {
        match value {
            0 => I2cError::Success,
            1 => I2cError::Nack,
            2 => I2cError::ArbitrationLoss,
            _ => I2cError::Success,
        }
    }
}
