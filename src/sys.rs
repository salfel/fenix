// Memory Map
pub const CM_PER: u32 = 0x44E00000;
pub const GPIO1: u32 = 0x4804C000;

pub fn write_addr(address: u32, value: u32) {
    unsafe {
        core::ptr::write_volatile(address as *mut u32, value);
    }
}

pub fn read_addr(address: u32) -> u32 {
    unsafe { core::ptr::read_volatile(address as *const u32) }
}

pub fn set_bit(address: u32, bit: u32) {
    let value = read_addr(address);
    write_addr(address, value | (1 << bit));
}

pub fn clear_bit(address: u32, bit: u32) {
    let value = read_addr(address);
    write_addr(address, value & !(1 << bit));
}

pub fn read_bit(address: u32, bit: u32) -> bool {
    let value = read_addr(address);

    (value & (1 << bit)) >> bit == 1
}
