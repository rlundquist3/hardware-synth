use bindgen;
use cc;
use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=tinyusb");

    let files = [
        "tinyusb/src/tusb.c",
        "tinyusb/src/common/tusb_fifo.c",
        "tinyusb/src/host/usbh.c",
        "tinyusb/src/class/midi/midi_host.c",
        "tinyusb/src/portable/synopsys/dwc2/hcd_dwc2.c",
        "tinyusb/src/portable/synopsys/dwc2/dwc2_common.c",
    ];
    let includes = [
        "tinyusb",
        "tinyusb/src",
        "tinyusb/CMSIS_5/CMSIS/Core/Include",
        "tinyusb/cmsis-device-h7/Include",
    ];
    let flags = [
        "-DSTM32H750xx",
        "-mcpu=cortex-m7",
        "-mthumb",
        "-mfloat-abi=hard",
        "-mfpu=fpv5-d16",
        "-Os",
        "-ffunction-sections",
        "-fdata-sections",
    ];

    // Compile TinyUSB
    let mut cc_builder = cc::Build::new();
    for f in files {
        cc_builder.file(f);
    }
    cc_builder
        .includes(includes)
        .flags(flags)
        .compile("tinyusb");

    // Create necessary Rust bindings for TinyUSB
    let gcc_location_command = Command::new("arm-none-eabi-gcc")
        .arg("-print-sysroot")
        .output()
        .expect("arm-none-eabi-gcc not on PATH");
    let sysroot = String::from_utf8(gcc_location_command.stdout)
        .unwrap()
        .trim()
        .to_string();

    let bindings_builder = bindgen::Builder::default()
        .header("tinyusb/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("tusb_.*|tuh_.*")
        .use_core()
        .ctypes_prefix("core::ffi")
        .clang_args([
            "-DSTM32H750xx",
            "--target=thumbv7em-none-eabihf",
            &format!("--sysroot={}", sysroot),
            &format!("-isystem{}/include", sysroot),
        ])
        .clang_args(includes.iter().map(|i| format!("-I{}", i)));

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings_builder
        .generate()
        .expect("Unable to generate bindings for tinyusb")
        .write_to_file(out_path.join("tinyusb_bindings.rs"))
        .expect("Unable to write bindings file for tinyusb");
}
