pub fn wreg(address: u32, data: u32) {
    unsafe {
        core::ptr::write_volatile(address as *mut u32, data);
    }
}

pub fn rreg(address: u32) -> u32 {
    unsafe { core::ptr::read_volatile(address as *const u32) }
}

pub fn wbit(address: u32, bit: u32, value: bool) {
    let value = rreg(address);
    wreg(address, value | (1 << bit));
}

pub fn rbit(address: u32, bit: u32) -> bool {
    let data = rreg(address);
    (data & (1 << bit)) != 0
}
