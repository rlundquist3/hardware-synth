#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code
)]

pub const BOARD_TUH_RHPORT: u8 = 1;

include!(concat!(env!("OUT_DIR"), "/tinyusb_bindings.rs"));
