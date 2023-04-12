#![no_main]

use std::collections::HashMap;

use libfuzzer_sys::fuzz_target;

const BLOCK_SIZE: usize = 512;

struct FuzzDevice(HashMap<u64, [u8; BLOCK_SIZE]>);

fuzz_target!(|data: &[u8]| {

});
