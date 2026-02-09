pub const PAGE_SIZE: u16 = 16384; // 16KB page size

pub fn mach_read_from_2(buf: &[u8]) -> u16 {
    (((buf[0] as u64) << 8) | (buf[1] as u64)) as u16
}

pub fn mach_read_from_4(buf: &[u8]) -> u32 {
    ((buf[0] as u32) << 24) | ((buf[1] as u32) << 16) | ((buf[2] as u32) << 8) | (buf[3] as u32)
}

pub fn mach_read_from_8(buf: &[u8]) -> u64 {
    let mut u64_1 = mach_read_from_4(&buf) as u64;
    u64_1 <<= 32;
    u64_1 |= mach_read_from_4(&buf[4..]) as u64;
    u64_1
}

pub fn fil_page_get_type(buf: &[u8]) -> u16 {
    mach_read_from_2(&buf)
}

pub fn _mach_write_to_4(buf: &mut [u8], n: u32) {
    buf[0] = (n >> 24) as u8;
    buf[1] = (n >> 16) as u8;
    buf[2] = (n >> 8) as u8;
    buf[3] = (n) as u8;
}

pub fn _mach_read_from_1(buf: &[u8]) -> u8 {
    buf[0]
}

pub fn ut_align_offset(off: u32, align: u32) -> u32 {
    off & (align - 1)
}