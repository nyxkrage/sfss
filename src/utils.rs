#[macro_export]
macro_rules! include_base_str {
    ($path:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))
    };
}

#[macro_export]
macro_rules! panic_dbg {
    ($val:ident) => {
        core::panic!(format!("{:?}", $val).as_ref());
    };
}

pub fn bools_to_u8(bools: [bool; 8]) -> u8 {
    // true true false...
    // 1100_0000
    let mut res: u8 = 0;
    for (i, b) in bools.iter().enumerate() {
        res |= (*b as u8) << (7 - i)
    }
    res
}

pub fn u8_to_bools(byte: u8) -> [bool; 8] {
    let mut res = [false; 8];
    for (i, b) in res.iter_mut().enumerate() {
        *b = byte & 1 << (7 - i) != 0;
    }
    res
}
