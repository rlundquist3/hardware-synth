// Pull in TinyUSB bindings from build.rs

#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code
)]

// USB-A Port on pins D29 (D-) / D30 (D+) (onboard USB-C is port 0)
pub const BOARD_TUH_RHPORT: u8 = 1;

include!(concat!(env!("OUT_DIR"), "/tinyusb_bindings.rs"));
