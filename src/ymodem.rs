
use ::uart;
use ::timer;
use ::crc;
use core::prelude::*;
use core::mem;
use core::fmt::Write;

// Start of 128B packet
const SOH: u8 = 0x01;
// Start of 1KiB packet.
const STX: u8 = 0x02;
// End of transfer.
const EOT: u8 = 0x04;
const ACK: u8 = 0x06;
const BSP: u8 = 0x08;
const NAK: u8 = 0x15;
const CAN: u8 = 0x18;
const EOF: u8 = 0x1A;
const CRC: u8 = 'C' as u8;

const HEADER_SIZE: usize = 3;
const TRAILER_SIZE: usize = 2;
const PACKET_OVERHEAD: usize = HEADER_SIZE + TRAILER_SIZE;
const BUFFER_SIZE: usize = 1024_usize + PACKET_OVERHEAD;

const SEQNO: usize = 1;
const SEQNO_COMPLEMENT: usize = 2;

#[derive(PartialEq)]
enum PacketResult {
    Ok(u32),
    Abort,
    Timeout,
    Error,
}

fn getchar(uart: &mut uart::Uart) -> Result<u8, ()> {
    // 1s timeout.
    let mut timer = timer::Timer::new(1000000);
    while !timer.expired() {
        if uart.send_receive() > 0 {
            match uart.read_byte() {
                None => {
                    return Err(());
                }
                Some(b) => {
                    return Ok(b);
                }
            }
        }
    }
    Err(())
}

fn receive_packet(uart: &mut uart::Uart,
                  buffer: &mut [u8; BUFFER_SIZE],
                  max_can_count: u32) -> PacketResult {
    let mut packet_size: usize;
    match getchar(uart) {
        Ok(c) => {
            match c {
                SOH => {
                    packet_size = 128;
                }
                STX => {
                    packet_size = 1024;
                }
                EOT => {
                    return PacketResult::Ok(0);
                }
                CAN => {
                    if max_can_count > 0 {
                        return receive_packet(uart, buffer, max_can_count - 1);
                    }
                    return PacketResult::Abort;
                }
                _ => {
                    return PacketResult::Abort;
                }
            }
            buffer[0] = c;
        }
        Err(()) => {
            return PacketResult::Timeout;
        }
    }
    for i in 1..packet_size+PACKET_OVERHEAD {
        match getchar(uart) {
            Ok(c) => {
                buffer[i as usize] = c;
            }
            Err(()) => {
                return PacketResult::Timeout;
            }
        }
    }
    if buffer[SEQNO] != buffer[SEQNO_COMPLEMENT] ^ 0xff_u8 {
        return PacketResult::Error;
    }
    if crc::crc16(&buffer[3..packet_size+PACKET_OVERHEAD]) != 0 {
        return PacketResult::Error;
    }
    PacketResult::Ok(packet_size as u32)
}

fn abort_file(uart: &mut uart::Uart) {
    uart.write_byte(CAN);
    uart.write_byte(CAN);
    uart.write_byte(CAN);
    uart.write_byte(CAN);
    uart.write_byte(BSP);
    uart.write_byte(BSP);
    uart.write_byte(BSP);
    uart.write_byte(BSP);
    uart.flush_output();
    uart.discard_input();
}

pub fn receive_file(uart: &mut uart::Uart, dest: &mut [u8]) -> Result<u32, ()> {
    // Read the first header.
    let mut buffer: [u8; BUFFER_SIZE] = unsafe { mem::uninitialized() };
    let mut total_length: u32 = 0;
    loop {
        uart.discard_input();
        uart.write_byte(CRC);
        let packet = receive_packet(uart, &mut buffer, 3);
        match packet {
            PacketResult::Ok(_) => {
                if buffer[SEQNO] != 0 {
                    uart.write_byte(NAK);
                    continue;
                }
                uart.write_byte(ACK);
                uart.write_byte(CRC);
                let mut size_start: usize = 3;
                // Skip the filename.
                while buffer[size_start] != 0 {
                    size_start = size_start + 1;
                }
                size_start = size_start + 1;
                // Read the file size.
                while buffer[size_start] != ' ' as u8 {
                    total_length = total_length * 10 + (buffer[size_start] - '0' as u8) as u32;
                    size_start = size_start + 1;
                }
                break;
            }
            PacketResult::Error => {
                abort_file(uart);
                return Err(());
            }
            PacketResult::Abort => {
                uart.write_byte(ACK);
                return Err(());
            }
            PacketResult::Timeout => {
                continue;
            }
        }
    }
    // Read the data.
    let mut expected_packet: u8 = 1;
    let mut bytes_received: u32 = 0;
    loop {
        let packet = receive_packet(uart, &mut buffer, 0);
        match packet {
            PacketResult::Ok(length) => {
                if length == 0 {
                    uart.write_byte(ACK);
                    uart.write_byte(CRC);
                    uart.flush_output();
                    // Check received packet length.
                    if total_length > bytes_received {
                        abort_file(uart);
                        let _ = writeln!(uart, "error: Received {} bytes (expected: {} bytes)", bytes_received, total_length);
                        return Err(());
                    }
                    break;
                }
                if buffer[SEQNO] != expected_packet {
                    uart.write_byte(NAK);
                    uart.flush_output();
                    uart.discard_input();
                    continue;
                }
                if bytes_received >= total_length {
                    abort_file(uart);
                    let _ = writeln!(uart, "error: Received too much data, expected {} bytes", total_length);
                    return Err(());
                }
                expected_packet = expected_packet + 1;
                uart.write_byte(ACK);
                for i in 0..length {
                    dest[bytes_received as usize] = buffer[i as usize + HEADER_SIZE];
                    bytes_received = bytes_received + 1;
                }
            }
            PacketResult::Error => {
                abort_file(uart);
                return Err(());
            }
            PacketResult::Abort => {
                return Err(());
            }
            PacketResult::Timeout => {
                abort_file(uart);
                return Err(());
            }
        }
    }
    // Read the last header (should be empty, signals that there are no more
    // files following).
    let packet = receive_packet(uart, &mut buffer, 0);
    match packet {
        PacketResult::Ok(length) => {
            if length == 0 {
                let _ = writeln!(uart, "error: expected final packet, received EOF?");
                return Err(());
            }
            uart.write_byte(ACK);
            uart.flush_output();
            // TODO: Check final CRC checksum?
        }
        PacketResult::Error => {
            abort_file(uart);
            return Err(());
        }
        PacketResult::Abort => {
            return Err(());
        }
        PacketResult::Timeout => {
            abort_file(uart);
            return Err(());
        }
    }
    Ok(total_length)
}
