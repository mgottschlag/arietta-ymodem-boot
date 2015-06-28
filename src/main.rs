#![feature(no_std, lang_items, core, core_intrinsics, core_prelude, core_str_ext, ptr_as_ref)]

#![no_std]
#![no_main]

use core::fmt::Write;
use core::mem;
use core::intrinsics;
use core::prelude::*;

#[macro_use]
extern crate core;

pub mod support;
mod crc;
mod uart;
mod timer;
mod ymodem;

const UIMAGE_MAGIC: u32 = 0x27051956;

#[repr(C, packed)]
struct ImageHeader {
    magic: u32,
    header_crc: u32,
    time: u32,
    size: u32,
    load_address: u32,
    entry_point: u32,
    data_crc: u32,
    os: u8,
    arch: u8,
    image_type: u8,
    compression_type: u8,
    name: [u8; 32],
}

#[no_mangle]
pub extern "C" fn main() {
    timer::Timer::init();
    let mut uart = uart::Uart::new();
    let _ = writeln!(uart, " ymodem bootloader");
    let _ = writeln!(uart, "===================\n");
    let _ = writeln!(uart, "downloading uImage to 0x22000000...");
    let payload: *mut [u8; 0x1000000] = 0x22000000 as *mut [u8; 0x1000000];
    let mut payload_length: u32;
    loop {
        match ymodem::receive_file(&mut uart, unsafe { payload.as_mut().unwrap() }) {
            Ok(length) => {
                payload_length = length;
                break;
            }
            Err(()) => {
                let _ = writeln!(uart, "download failed, retrying...");
                uart.flush_output();
            }
        }
    }
    let _ = writeln!(uart, "file received ({} bytes)", payload_length);
    uart.flush_output();
    let header: &ImageHeader = unsafe { mem::transmute(payload as *const ImageHeader) };
    if u32::from_be(header.magic) != UIMAGE_MAGIC {
        let _ = writeln!(uart, "error: invalid uimage magic 0x{:08x} (expected: 0x{:08x})", u32::from_be(header.magic), UIMAGE_MAGIC);
        uart.flush_output();
        loop {}
    }
    if payload_length < mem::size_of::<ImageHeader>() as u32 + u32::from_be(header.size) {
        let _ = writeln!(uart, "error: invalid uimage size {} (received only {} bytes)", u32::from_be(header.size), payload_length);
        uart.flush_output();
        loop {}
    }
    let dest: *mut u8 = u32::from_be(header.load_address) as *mut u8;
    unsafe {
        intrinsics::copy(0x22000040 as *const u8, dest, u32::from_be(header.size) as usize);
    }
    let _ = writeln!(uart, "relocated file from 0x{:08x} to 0x{:08x}", payload as u32, dest as u32);
    let _ = writeln!(uart, "jumping to 0x{:08x}...\n", u32::from_be(header.entry_point));
    uart.flush_output();
    let entry: extern "C" fn() = unsafe { mem::transmute(u32::from_be(header.entry_point)) };
    entry();
}

