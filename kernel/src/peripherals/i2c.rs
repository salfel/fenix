use core::{
    arch::asm,
    cmp::min,
    fmt::{self, Arguments, Write},
};

use crate::{
    internals::clock::{self, FuncClock},
    interrupts::{self, Interrupt, Mode},
};
use embedded_hal::i2c;
use shared::{
    alloc::vec::Vec,
    sys::clear_bit,
};
use shared::{
    i2c::I2cError,
    sys::{read_addr, set_bit, write_addr},
};


const SYS_CLOCK: u32 = 48_000_000;
const INTERNAL_CLOCK: u32 = 12_000_000;
const OUTPUT_CLOCK: u32 = 100_000;

const I2C_SYSC: u32 = 0x10;
const I2C_IRQSTATUS_RAW: u32 = 0x24;
const I2C_IRQSTATUS: u32 = 0x28;
const I2C_IRQSTATUS_SET: u32 = 0x2C;
const I2C_IRQSTATUS_CLR: u32 = 0x30;
const I2C_CNT: u32 = 0x98;
const I2C_DATA: u32 = 0x9C;
const I2C_CON: u32 = 0xA4;
const I2C_OA: u32 = 0xA8;
const I2C_SA: u32 = 0xAC;
const I2C_PSC: u32 = 0xB0;
const I2C_SCLL: u32 = 0xB4;
const I2C_SCLH: u32 = 0xB8;
const I2C_SYSTEST: u32 = 0xBC;
const I2C_SYSS: u32 = 0x90;
const I2C_BUF: u32 = 0x94;
const I2C_BUFSTAT: u32 = 0xC0;

const RECEIVE_THRESHOLD: u32 = 16;
const TRANSMIT_THRESHOLD: u32 = 16;

const TEST_ENABLE: u32 = 1 << 15;
const TEST_MODE: u32 = 12;

pub fn initialize() {
    let i2c = get_i2c();
    i2c.initialize();
}

#[allow(static_mut_refs)]
pub fn get_i2c() -> &'static mut I2C {
    unsafe { &mut I2C }
}

static mut I2C: I2C = I2C::new(I2cModule::I2C2);

pub struct I2C {
    module: I2cModule,
    ready: bool,
    error: Option<I2cError>,
    receive_buffer: Vec<u8>,
    transmit_buffer: Vec<u8>,
    transmit_index: usize,
}

impl i2c::I2c for I2C {
    /// Executes an I2C transaction with the specified slave device.
    ///
    /// This method initiates an I2C transaction by configuring the peripheral using the transaction mode
    /// derived from the first operation in the supplied slice. It sets the target slave address, clears
    /// the transmit buffer, and then processes each write operation by queuing its data for transmission.
    /// The method waits for the I2C interface to become ready between operations and manages interrupts
    /// to facilitate the non-blocking transmission. If any error is detected during communication,
    /// the transaction is aborted and the error is returned.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the transaction completes successfully, or an error of type `Self::Error`
    /// if a transmission error occurs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use kernel::peripherals::i2c::{I2C, i2c};
    /// let mut i2c = I2C::new();
    /// let mut operations = [i2c::Operation::Write(&[0x01, 0x02, 0x03])];
    ///
    /// if let Err(e) = i2c.transaction(0x50, &mut operations) {
    ///     eprintln!("I2C transaction failed: {:?}", e);
    /// }
    /// ```
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        let mode: I2cMode = operations.first().unwrap().into();

        self.enable();

        self.set_mode(mode);
        self.set_slave(address);
        self.clear_buffer();

        let mut started = false;

        while self.busy() {}

        for operation in operations {
            if let i2c::Operation::Write(buffer) = operation {
                if buffer.is_empty() {
                    continue;
                }

                for i in 0..buffer.len() {
                    self.transmit_buffer.push(buffer[i]);
                }

                self.set_count(buffer.len() as u32);
                self.ready = false;

                if !started {
                    self.start();
                    started = true;
                }

                self.enable_interrupts(I2cMode::Transmitter);
                self.wait_ready();
                self.disable_interrupts(I2cMode::Transmitter);

                if let Some(error) = self.error {
                    self.stop();
                    self.disable();

                    self.error = None;

                    return Err(error);
                }
            }
        }

        self.stop();
        self.disable();

        Ok(())
    }
}

impl fmt::Write for I2C {
    /// Writes the provided string slice to the I2C interface for formatted output.
    ///
    /// This method implements the `fmt::Write` trait for the I2C peripheral by converting the string slice into bytes
    /// and writing them to a fixed I2C address (0x10). If the underlying write operation fails, a formatting error (`fmt::Error`)
    /// is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming a properly initialized I2C instance:
    /// let mut i2c = I2C::new();
    /// assert!(i2c.write_str("Hello, I2C!").is_ok());
    /// ```
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.write(0x10, s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}

/// Writes formatted arguments to the I2C interface.
/// 
/// Retrieves the global I2C instance and writes the provided formatted arguments via the I2C's
/// `write_fmt` method. This function panics if the write operation fails.
/// 
/// # Examples
///
/// ```
/// // Print a formatted message to the I2C interface.
/// print(format_args!("Sensor reading: {}", 42));
/// ```
pub fn print(args: Arguments<'_>) {
    let i2c = get_i2c();
    i2c.write_fmt(args).unwrap();
}

/// Writes formatted output followed by a newline to the I2C interface.
///
/// This function fetches the I2C instance using `get_i2c` and sends the provided formatted arguments,
/// appending a newline to the output. It will panic if the write operation fails.
///
/// # Examples
///
/// ```
/// use core::fmt::Arguments;
///
/// // Write "Hello, I2C!" followed by a newline to the I2C interface.
/// println(format_args!("Hello, I2C!"));
/// ```
pub fn println(args: Arguments<'_>) {
    let i2c = get_i2c();
    i2c.write_fmt(format_args!("{}\n", args)).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::i2c::print(format_args!($($arg)*))
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::i2c::println(format_args!($($arg)*))
    }
}

impl I2C {
    /// Creates a new I2C instance for the specified module.
    ///
    /// This constant constructor initializes an I2C instance by setting its
    /// ready state to true, clearing any errors, and initializing empty receive
    /// and transmit buffers with the transmit index reset to 0.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a new I2C instance using a specific I2C module variant.
    /// let i2c = I2C::new(I2cModule::I2C1);
    /// assert!(i2c.ready);
    /// assert!(i2c.error.is_none());
    /// assert!(i2c.receive_buffer.is_empty());
    /// assert!(i2c.transmit_buffer.is_empty());
    /// ```
    const fn new(module: I2cModule) -> Self {
        Self {
            module,
            ready: true,
            error: None,
            receive_buffer: Vec::new(),
            transmit_buffer: Vec::new(),
            transmit_index: 0,
        }
    }

    fn base(&self) -> u32 {
        self.module as u32
    }

    /// Initializes the I2C hardware interface.
    ///
    /// This function sets up the I2C peripheral for communication by enabling its clock and interrupts,
    /// performing a soft reset, initializing necessary clocks, setting the device's own address, and
    /// enabling the module. It waits for the reset to complete before configuring the transmission threshold.
    ///
    /// # Examples
    ///
    /// ```
    /// let i2c = I2C::new();
    /// i2c.initialize();
    /// // The I2C interface is now configured and ready for communication.
    /// ```
    fn initialize(&self) {
        clock::enable(FuncClock::I2C2);

        interrupts::enable_interrupt(Interrupt::I2C2INT, Mode::IRQ, 2); // enable irq
        interrupts::register_handler(irq_handler, Interrupt::I2C2INT); // register handler

        // config
        self.soft_reset();
        self.init_clocks();
        self.set_own_address();
        self.enable();
        self.wait_reset();

        // init
        self.setup_threshold();
    }

    /// Handles I2C interrupt events by processing transmit and receive operations.
    ///
    /// This method checks the I2C interrupt status register and processes any pending events:
    /// - For transmit ready interrupts (`XRDY` and `XDR`), it writes data from the transmit buffer.
    /// - For receive ready interrupts (`RRDY` and `RDR`), it reads incoming data.
    /// - When the transaction completion interrupt (`ARDY`) is set, it marks the interface as ready.
    /// - In case of a negative acknowledgment (`NACK`), it sets an error state and marks the interface as ready.
    ///
    /// After processing each event, the corresponding interrupt flag is cleared from the status register.
    ///
    /// # Examples
    ///
    /// ```
    /// // Example usage of irq_handler in an I2C peripheral context.
    /// let mut i2c = I2C::new();
    ///
    /// // Simulate an ARDY interrupt (transaction complete) by manually writing to the IRQ status register.
    /// // In an actual system, this register would be set by hardware.
    /// unsafe {
    ///     write_addr(i2c.base() + I2C_IRQSTATUS, I2cInterrupt::ARDY as u32);
    /// }
    ///
    /// // Process the interrupt.
    /// i2c.irq_handler();
    ///
    /// // The I2C interface should now be marked as ready.
    /// assert!(i2c.ready);
    /// ```
    fn irq_handler(&mut self) {
        let value = read_addr(self.base() + I2C_IRQSTATUS);

        if value & I2cInterrupt::XRDY as u32 != 0 {
            for _ in 0..min(TRANSMIT_THRESHOLD, self.transmit_bytes_left()) {
                self.write_data();
            }

            write_addr(self.base() + I2C_IRQSTATUS, I2cInterrupt::XRDY as u32);
            return;
        }

        if value & I2cInterrupt::XDR as u32 != 0 {
            for _ in 0..self.transmit_bytes_available() {
                self.write_data();
            }

            write_addr(self.base() + I2C_IRQSTATUS, I2cInterrupt::XDR as u32);
            return;
        }

        if value & I2cInterrupt::RRDY as u32 != 0 {
            for _ in 0..RECEIVE_THRESHOLD {
                self.read_data();
            }

            write_addr(self.base() + I2C_IRQSTATUS, I2cInterrupt::RRDY as u32);
            return;
        }

        if value & I2cInterrupt::RDR as u32 != 0 {
            for _ in 0..self.receive_bytes_available() {
                self.read_data();
            }

            write_addr(self.base() + I2C_IRQSTATUS, I2cInterrupt::RDR as u32);
            return;
        }

        if value & I2cInterrupt::ARDY as u32 != 0 {
            self.ready = true;

            write_addr(self.base() + I2C_IRQSTATUS, I2cInterrupt::ARDY as u32);
            return;
        }

        if value & I2cInterrupt::NACK as u32 != 0 {
            self.error = Some(I2cError::Nack);
            self.ready = true;

            write_addr(self.base() + I2C_IRQSTATUS, I2cInterrupt::NACK as u32);
        }
    }

    /// Performs a soft reset of the I2C module.
    ///
    /// This method triggers a software reset by reading the current value of the I2C_SYSC register,
    /// setting the soft reset bit (0x2), and writing the modified value back to the register.
    /// This operation reinitializes the internal state of the I2C peripheral without performing
    /// a full hardware reset.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `i2c` is a valid instance of the I2C interface.
    /// i2c.soft_reset();
    /// ```
    fn soft_reset(&self) {
        write_addr(
            self.base() + I2C_SYSC,
            read_addr(self.base() + I2C_SYSC) | 0x2,
        );
    }

    fn init_clocks(&self) {
        let prescaler = (SYS_CLOCK / INTERNAL_CLOCK) - 1;
        write_addr(self.base() + I2C_PSC, prescaler);

        let mut divider = INTERNAL_CLOCK / OUTPUT_CLOCK;
        divider /= 2;

        write_addr(self.base() + I2C_SCLL, divider - 7);
        write_addr(self.base() + I2C_SCLH, divider - 5);
    }

    fn set_own_address(&self) {
        write_addr(self.base() + I2C_OA, 0x50);
    }

    fn enable(&self) {
        set_bit(self.base() + I2C_CON, 15);
    }

    fn disable(&self) {
        clear_bit(self.base() + I2C_CON, 15);
    }

    /// Blocks until the I2C hardware reset process is complete.
    ///
    /// This function continuously polls the I2C system status register until the reset
    /// completion flag is detected. It is used to ensure that the I2C module is ready for
    /// further configuration after a reset.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume an I2C instance is created and a hardware reset has been initiated.
    /// // wait_reset() will block until the I2C module indicates that the reset is complete.
    /// let i2c = I2C::new();
    /// i2c.wait_reset();
    /// ```
    fn wait_reset(&self) {
        while read_addr(self.base() + I2C_SYSS) & 0x1 == 0 {}
    }

    /// Configures the I2C interface for transmitter or receiver operation.
    ///
    /// This method updates the I2C control register by setting the general mode enable bit
    /// and adjusting the transmitter selection bit based on the specified mode.
    /// When the mode is `I2cMode::Transmitter`, the transmitter bit is set; otherwise, it is cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// # use kernel::peripherals::i2c::{I2C, I2cMode};
    /// let i2c = I2C::new();
    /// i2c.set_mode(I2cMode::Transmitter);
    /// ```
    fn set_mode(&self, mode: I2cMode) {
        let value = read_addr(self.base() + I2C_CON);
        let is_transmitter = matches!(mode, I2cMode::Transmitter);
        write_addr(
            self.base() + I2C_CON,
            value | 1 << 10 | (is_transmitter as u32) << 9,
        );
    }

    fn setup_threshold(&self) {
        write_addr(
            self.base() + I2C_BUF,
            (RECEIVE_THRESHOLD - 1) << 8 | (TRANSMIT_THRESHOLD - 1),
        );
    }

    fn enable_irq(&self, irq: I2cInterrupt) {
        let value = read_addr(self.base() + I2C_IRQSTATUS_SET);
        write_addr(self.base() + I2C_IRQSTATUS_SET, value | irq as u32);
    }

    fn disable_irq(&self, irq: I2cInterrupt) {
        let value = read_addr(self.base() + I2C_IRQSTATUS_CLR);
        write_addr(self.base() + I2C_IRQSTATUS_CLR, value | irq as u32);
    }

    #[allow(unused)]
    /// Enables test mode for the I2C peripheral.
    /// 
    /// This method configures the I2C hardware for testing by setting the test enable flag and applying a specific test mode
    /// configuration in the system test register. This mode is intended for diagnostic and validation purposes.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let i2c = I2C::new(...); // Create or obtain an I2C instance as required.
    /// i2c.enable_test_mode();
    /// ```
    fn enable_test_mode(&self) {
        let value = read_addr(self.base() + I2C_SYSTEST);
        write_addr(
            self.base() + I2C_SYSTEST,
            value | TEST_ENABLE | (0x3 << TEST_MODE),
        );
    }

    /// Writes a buffer of data to the specified I2C slave device.
    ///
    /// This function initiates an I2C write transaction to the device at the provided address,
    /// sending the bytes in `data`. It returns `Ok(())` if the transmission is successful,
    /// or an `I2cError` if an error occurs.
    ///
    /// # Examples
    ///
    /// ```
    /// use kernel::peripherals::i2c::{I2C, I2cError};
    ///
    /// let mut i2c = I2C::new();
    /// let payload = [0x01, 0x02, 0x03];
    /// i2c.write(0x3C, &payload).expect("I2C write failed");
    /// ```
    pub fn write(&mut self, address: u8, data: &[u8]) -> Result<(), I2cError> {
        <Self as i2c::I2c>::write(self, address, data)
    }

    /// Clears the I2C transmit buffer and resets its transmission state.
    ///
    /// This method empties the internal transmit buffer, resets the transmit index to zero,
    /// and clears the hardware FIFO to remove any lingering data. Use this function to ensure
    /// the I2C interface is in a known state before starting a new transmission.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Create a new I2C instance (assuming `I2C::new()` is available).
    /// let mut i2c = I2C::new();
    ///
    /// // Simulate buffered data.
    /// i2c.transmit_buffer.push(0xAA);
    /// i2c.transmit_buffer.push(0xBB);
    /// i2c.transmit_index = 2;
    ///
    /// // Clear the transmit buffer and FIFO.
    /// i2c.clear_buffer();
    ///
    /// // Verify that the buffer is cleared and the index reset.
    /// assert!(i2c.transmit_buffer.is_empty());
    /// assert_eq!(i2c.transmit_index, 0);
    /// ```
    fn clear_buffer(&mut self) {
        self.transmit_buffer.clear();
        self.transmit_index = 0;
        self.clear_transmit_fifo();
    }

    /// Writes a string to an I2C device.
    /// 
    /// This method converts the provided string into its byte representation and sends it to the I2C device at the given address,
    /// using the lower-level write operation.
    /// 
    /// # Arguments
    /// 
    /// * `address` - The I2C slave address of the target device.
    /// * `data` - The string slice to be transmitted.
    /// 
    /// # Errors
    /// 
    /// Returns an `I2cError` if the write operation fails.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// // Assume `i2c` is an initialized I2C instance.
    /// let mut i2c = I2C::new();
    /// assert!(i2c.write_str(0x1A, "Hello, I2C!").is_ok());
    /// ```
    pub fn write_str(&mut self, address: u8, data: &str) -> Result<(), I2cError> {
        self.write(address, data.as_bytes())
    }

    /// Writes a single character to an I2C device at the specified address.
    ///
    /// Converts the provided character into its byte representation (using the lower 8 bits)
    /// and transmits it via the I2C interface. Note that this method is intended for writing simple
    /// ASCII characters; any higher-order bits are discarded.
    ///
    /// # Arguments
    ///
    /// * `address` - The I2C address of the target device.
    /// * `data` - The character to write.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the character is successfully sent, or an `I2cError` if the write fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use kernel::peripherals::i2c::{I2C, I2cError};
    /// let mut i2c = I2C::new();
    /// // Write character 'A' (0x41) to the device at address 0x50.
    /// i2c.write_char(0x50, 'A').unwrap();
    /// ```
    pub fn write_char(&mut self, address: u8, data: char) -> Result<(), I2cError> {
        self.write(address, &[data as u8])
    }

    /// Enables all interrupts associated with the specified I2C mode.
    /// 
    /// Iterates over the interrupts defined by the provided I2C mode and enables each one, ensuring
    /// the peripheral is set up to handle events corresponding to that mode of operation.
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Example usage:
    /// let mode = I2cMode::Transmitter; // or I2cMode::Receiver as needed
    /// let i2c = I2C::new();
    /// i2c.enable_interrupts(mode);
    /// ```
    fn enable_interrupts(&self, mode: I2cMode) {
        for interrupt in mode.interrupts() {
            self.enable_irq(*interrupt);
        }
    }

    /// Disables all I2C interrupts associated with the specified mode.
    ///
    /// This method retrieves the list of interrupts from the given `I2cMode` and disables each one
    /// by invoking the internal `disable_irq` method.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Assume I2C and I2cMode are accessible in this scope.
    /// let i2c = I2C::new();
    /// // Disable interrupts associated with the Transmitter mode.
    /// i2c.disable_interrupts(I2cMode::Transmitter);
    /// ```
    fn disable_interrupts(&self, mode: I2cMode) {
        for interrupt in mode.interrupts() {
            self.disable_irq(*interrupt);
        }
    }

    /// Sets the active I2C slave address for subsequent communications.
    ///
    /// This method writes the specified `address` into the I2C slave address register,
    /// configuring the I2C controller to target the designated slave device.
    ///
    /// # Examples
    ///
    /// ```
    /// let i2c = I2C::new();
    /// i2c.set_slave(0x1A); // Configures the I2C interface to communicate with slave device at address 0x1A
    /// ```
    fn set_slave(&self, address: u8) {
        write_addr(self.base() + I2C_SA, address as u32);
    }

    /// Sets the number of data bytes to be transferred in an I2C transaction.
    ///
    /// This function writes the specified count value to the hardware register responsible
    /// for managing the byte count during an I2C operation.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `i2c` is an initialized instance of the I2C interface:
    /// i2c.set_count(8);
    /// ```
    fn set_count(&self, count: u32) {
        write_addr(self.base() + I2C_CNT, count);
    }

    fn clear_transmit_fifo(&self) {
        set_bit(self.base() + I2C_BUF, 6);
    }

    /// Checks if the I2C interface is currently busy.
    ///
    /// This method reads the raw interrupt status register and returns `true` if the busy flag,
    /// corresponding to bit 12, is set. This indicates that the I2C controller is actively processing
    /// an operation and is not available for new transactions.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create an instance of the I2C interface (assumes an appropriate constructor like I2C::new exists)
    /// let i2c = I2C::new();
    ///
    /// // Use the busy method to check if the I2C interface is available
    /// if i2c.busy() {
    ///     // I2C is currently busy; handle accordingly
    ///     println!("I2C is busy.");
    /// } else {
    ///     // I2C is idle and ready for a new transaction
    ///     println!("I2C is ready.");
    /// }
    /// ```
    fn busy(&self) -> bool {
        let value = read_addr(self.base() + I2C_IRQSTATUS_RAW);
        value & (1 << 12) != 0
    }

    /// Blocks until the I2C peripheral is ready for the next operation.
    ///
    /// This method continuously polls the `ready` flag and executes a no-operation
    /// (NOP) instruction on each iteration to ensure that the busy-wait loop is not
    /// optimized away by the compiler. It only returns once the I2C module indicates
    /// readiness, ensuring that subsequent operations do not start prematurely.
    ///
    /// # Examples
    ///
    /// ```
    /// // This example creates a dummy I2C-like structure to demonstrate usage of `wait_ready`.
    /// struct DummyI2C {
    ///     ready: bool,
    /// }
    ///
    /// impl DummyI2C {
    ///     fn wait_ready(&self) {
    ///         while !self.ready {
    ///             // In the real implementation, a NOP is executed to prevent optimizations.
    ///         }
    ///     }
    /// }
    ///
    /// // Create a dummy instance that is immediately ready.
    /// let i2c = DummyI2C { ready: true };
    /// i2c.wait_ready();
    /// ```
    fn wait_ready(&self) {
        loop {
            if self.ready {
                break;
            }

            // added nop instruction to remove compiler optimizations
            unsafe {
                asm!("nop");
            }
        }
    }

    /// Initiates an I2C start condition on the bus.
    ///
    /// This method sets the START bit in the I2C control register, triggering the
    /// beginning of an I2C transaction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Create a new I2C instance and begin a transaction.
    /// let i2c = I2C::new();
    /// i2c.start();
    /// ```
    fn start(&self) {
        let value = read_addr(self.base() + I2C_CON);
        write_addr(self.base() + I2C_CON, value | 0x1);
    }

    fn stop(&mut self) {
        let value = read_addr(self.base() + I2C_CON);
        write_addr(self.base() + I2C_CON, value | 0x2);
    }

    fn write_data(&mut self) {
        let data = self.transmit_buffer.get(self.transmit_index).unwrap();
        write_addr(self.base() + I2C_DATA, *data as u32);
        self.transmit_index += 1;
    }

    fn read_data(&mut self) {
        let data = read_addr(self.base() + I2C_DATA) as u8;
        self.receive_buffer.push(data);
    }

    fn transmit_bytes_available(&self) -> u32 {
        read_addr(self.base() + I2C_BUFSTAT) & 0x3F
    }

    fn transmit_bytes_left(&self) -> u32 {
        self.transmit_buffer.len() as u32 - self.transmit_index as u32
    }

    /// Returns the number of bytes currently available in the receive buffer.
    ///
    /// This method reads the I2C buffer status register at an offset from the base address,
    /// extracts the relevant bits that indicate how many bytes are ready to be read, and returns that count.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume `i2c` is an initialized instance of the I2C struct.
    /// let available = i2c.receive_bytes_available();
    /// // The number of available bytes should not exceed the maximum representable value (0x3F).
    /// assert!(available <= 0x3F);
    /// ```
    fn receive_bytes_available(&self) -> u32 {
        (read_addr(self.base() + I2C_BUFSTAT) >> 8) & 0x3F
    }
}

impl i2c::ErrorType for I2C {
    type Error = I2cError;
}

/// Delegates I2C interrupt handling to the global I2C instance.
///
/// This function retrieves the static I2C instance using `get_i2c()` and invokes its interrupt handler,
/// allowing the I2C peripheral to process pending interrupts.
///
/// # Examples
///
/// ```
/// irq_handler();
/// ```
fn irq_handler() {
    let i2c = get_i2c();
    i2c.irq_handler()
}

#[derive(Clone, Copy)]
enum I2cModule {
    I2C2 = 0x4819_C000,
}

#[allow(unused)]
enum I2cMode {
    Transmitter,
    Receiver,
}

impl From<&i2c::Operation<'_>> for I2cMode {
    /// Converts a reference to an `i2c::Operation` into its corresponding `I2cMode`.
    ///
    /// Maps a read operation to `I2cMode::Receiver` and a write operation to `I2cMode::Transmitter`.
    ///
    /// # Examples
    ///
    /// ```
    /// use i2c::Operation;
    /// use crate::I2cMode;
    ///
    /// let op_read = Operation::Read(&[0x12, 0x34]);
    /// assert_eq!(I2cMode::from(&op_read), I2cMode::Receiver);
    ///
    /// let op_write = Operation::Write(&[0xAB, 0xCD]);
    /// assert_eq!(I2cMode::from(&op_write), I2cMode::Transmitter);
    /// ```
    fn from(value: &i2c::Operation) -> Self {
        match value {
            i2c::Operation::Read(_) => I2cMode::Receiver,
            i2c::Operation::Write(_) => I2cMode::Transmitter,
        }
    }
}

impl I2cMode {
    /// Returns a slice of I2C interrupts corresponding to the current mode.
    ///
    /// In transmitter mode, the slice includes interrupts for transmit readiness (`XRDY`),
    /// transmit data (`XDR`), transfer complete (`ARDY`), and negative acknowledgment (`NACK`).
    /// In receiver mode, it includes interrupts for receive readiness (`RRDY`), receive data (`RDR`),
    /// transfer complete (`ARDY`), and negative acknowledgment (`NACK`).
    ///
    /// # Examples
    ///
    /// ```
    /// use kernel::peripherals::i2c::{I2cMode, I2cInterrupt};
    ///
    /// let tx_interrupts = I2cMode::Transmitter.interrupts();
    /// assert_eq!(tx_interrupts, &[I2cInterrupt::XRDY, I2cInterrupt::XDR, I2cInterrupt::ARDY, I2cInterrupt::NACK]);
    ///
    /// let rx_interrupts = I2cMode::Receiver.interrupts();
    /// assert_eq!(rx_interrupts, &[I2cInterrupt::RRDY, I2cInterrupt::RDR, I2cInterrupt::ARDY, I2cInterrupt::NACK]);
    /// ```
    fn interrupts(&self) -> &[I2cInterrupt] {
        match self {
            I2cMode::Transmitter => &[
                I2cInterrupt::XRDY,
                I2cInterrupt::XDR,
                I2cInterrupt::ARDY,
                I2cInterrupt::NACK,
            ],
            I2cMode::Receiver => &[
                I2cInterrupt::RRDY,
                I2cInterrupt::RDR,
                I2cInterrupt::ARDY,
                I2cInterrupt::NACK,
            ],
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy)]
enum I2cInterrupt {
    XDR = 1 << 14, // Transmit Draining
    RDR = 1 << 13, // Receive Draining
    XRDY = 1 << 4, // Transmit Ready
    RRDY = 1 << 3, // Receive Ready
    ARDY = 1 << 2, // Access Ready
    NACK = 1 << 1, // No Acknowledge
}
