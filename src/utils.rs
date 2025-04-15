#[inline]
pub fn wreg(address: u32, data: u32) {
    unsafe {
        core::ptr::write_volatile(address as *mut u32, data);
    }
}

#[inline]
pub fn rreg(address: u32) -> u32 {
    unsafe { core::ptr::read_volatile(address as *const u32) }
}

#[inline]
pub fn wbit(address: u32, bit: u32, value: bool) {
    let data = rreg(address);
    if value {
        wreg(address, data | (1 << bit));
    } else {
        wreg(address, data & !(1 << bit));
    }
}

#[inline]
pub fn rbit(address: u32, bit: u32) -> bool {
    let data = rreg(address);
    (data & (1 << bit)) != 0
}

pub fn nop() {}
