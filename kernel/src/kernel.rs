use core::convert::TryInto;

use crate::{
    internals::{
        sysclock::millis,
        tasks::{scheduler, TaskState},
    },
    peripherals::{
        gpio::{self},
        i2c,
    },
};
use shared::interrupts;
use shared::kernel::Syscall;

struct SyscallError {}

#[repr(C)]
struct TrapFrame {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
}

impl<'a> TryInto<Syscall<'a>> for &TrapFrame {
    type Error = SyscallError;

    fn try_into(self) -> Result<Syscall<'a>, Self::Error> {
        match self.r12 {
            0 => Ok(Syscall::Exit),
            1 => Ok(Syscall::Yield {
                sp: self.r0,
                pc: self.r1,
                until: match self.r2 {
                    0 => None,
                    until => Some(until),
                },
            }),
            2 => Ok(Syscall::Millis),
            3 => Ok(Syscall::GpioRead {
                pin: (self.r1, self.r0.into()),
            }),
            4 => Ok(Syscall::GpioWrite {
                pin: (self.r1, self.r0.into()),
                value: self.r2 != 0,
            }),
            5 => Ok(Syscall::I2cBegin {
                slave_address: self.r0,
            }),
            6 => Ok(Syscall::I2cWrite {
                data: unsafe { core::slice::from_raw_parts(self.r0 as *mut u8, self.r1 as usize) },
            }),
            7 => Ok(Syscall::I2cEnd),
            8 => Ok(Syscall::Panic),
            _ => Err(SyscallError {}),
        }
    }
}

#[repr(C)]
struct SyscallReturn {
    exit: bool,
    value: u32,
}

impl SyscallReturn {
    fn exit() -> Self {
        SyscallReturn {
            exit: true,
            value: 0,
        }
    }

    fn value(value: u32) -> Self {
        SyscallReturn { exit: false, value }
    }
}

#[no_mangle]
extern "C" fn swi_handler(frame: &TrapFrame) -> SyscallReturn {
    let syscall: Syscall = match frame.try_into() {
        Ok(syscall) => syscall,
        Err(_) => panic!("invalid syscall"),
    };

    match syscall {
        Syscall::Exit => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                task.terminate();
            }

            scheduler.cycle();

            SyscallReturn::exit()
        }
        Syscall::Yield {
            sp,
            pc,
            until: None,
        } => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                task.context.pc = pc;
                task.context.sp = sp;
                task.state = TaskState::Stored;
            }

            scheduler.cycle();

            SyscallReturn::exit()
        }
        Syscall::Yield {
            sp,
            pc,
            until: Some(until),
        } => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                task.context.pc = pc;
                task.context.sp = sp;
                task.state = TaskState::Waiting { until };
            }

            scheduler.cycle();

            SyscallReturn::exit()
        }
        Syscall::Millis => SyscallReturn::value(millis()),
        Syscall::GpioWrite { pin, value } => {
            gpio::write(pin, value);

            SyscallReturn::value(0)
        }
        Syscall::GpioRead { pin } => {
            let value = gpio::read(pin);

            SyscallReturn::value(value as u32)
        }
        Syscall::I2cBegin { slave_address } => {
            let i2c = i2c::get_i2c();
            i2c.begin_transmission(slave_address);

            SyscallReturn::value(0)
        }
        Syscall::I2cWrite { data } => {
            let i2c = i2c::get_i2c();
            i2c.write_buf(data);

            SyscallReturn::value(0)
        }
        Syscall::I2cEnd => {
            interrupts::enabled(|| {
                let i2c = i2c::get_i2c();
                i2c.end_transmission();
            });

            SyscallReturn::value(0)
        }
        Syscall::Panic => {
            let scheduler = scheduler();

            if let Some(task) = scheduler.current() {
                task.terminate();
            }

            scheduler.cycle();

            SyscallReturn::exit()
        }
    }
}

#[no_mangle]
pub fn kernel_loop() {
    loop {
        let scheduler = scheduler();
        scheduler.switch();
    }
}
