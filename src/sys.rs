// Memory Map
pub const CM_DPLL: u32 = 0x44E05000;
pub const GPIO1: u32 = 0x4804C000;
pub const INTC: u32 = 0x48200000;

#[inline]
pub fn write_addr(address: u32, value: u32) {
    unsafe {
        core::ptr::write_volatile(address as *mut u32, value);
    }
}

#[inline]
pub fn read_addr(address: u32) -> u32 {
    unsafe { core::ptr::read_volatile(address as *const u32) }
}

#[inline]
pub fn set_bit(address: u32, bit: u32) {
    let value = read_addr(address);
    write_addr(address, value | (1 << bit));
}

#[inline]
pub fn clear_bit(address: u32, bit: u32) {
    let value = read_addr(address);
    write_addr(address, value & !(1 << bit));
}

#[inline]
pub fn read_bit(address: u32, bit: u32) -> bool {
    let value = read_addr(address);

    (value & (1 << bit)) >> bit == 1
}

pub fn noop() {}


