
use core::fmt;
use core::intrinsics;
use core::mem;
use core::prelude::*;

pub struct Uart {
    send_buffer: [u8; 2048],
    receive_buffer: [u8; 2048],
    send_start: u32,
    send_end: u32,
    receive_start: u32,
    receive_end: u32,
}

const DBGU_SR: *mut u32 = 0xFFFFF214 as *mut u32;
const DBGU_RHR: *mut u32 = 0xFFFFF218 as *mut u32;
const DBGU_THR: *mut u32 = 0xFFFFF21c as *mut u32;

const DBGU_SR_RXRDY: u32 = 0x1;
const DBGU_SR_TXRDY: u32 = 0x2;

impl Uart {
    pub fn new() -> Uart {
        unsafe {
            Uart {
                send_buffer: mem::uninitialized(),
                receive_buffer: mem::uninitialized(),
                send_start: 0,
                send_end: 0,
                receive_start: 0,
                receive_end: 0,
            }
        }
    }

    pub fn send_receive(&mut self) -> u32 {
        if self.can_send() && self.send_start != self.send_end {
            self.send();
        }
        if self.can_receive() {
            self.receive();
            if self.receive_end < self.receive_start {
                2048 + self.receive_end - self.receive_start
            } else {
                self.receive_end - self.receive_start
            }
        } else {
            0
        }
    }

    pub fn flush_output(&mut self) {
        while self.send_start != self.send_end {
            self.send_receive();
        }
    }

    pub fn discard_input(&mut self) {
        self.receive_start = self.receive_end;
    }

    pub fn write_byte(&mut self, c: u8) {
        let next_index = (self.send_end + 1) & 2047;
        if next_index == self.send_start {
            return;
        }
        self.send_buffer[self.send_end as usize] = c;
        self.send_end = next_index;
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        if self.receive_start == self.receive_end {
            return None;
        }
        let b = self.receive_buffer[self.receive_start as usize];
        self.receive_start = (self.receive_start + 1) & 2047;
        Some(b)
    }

    fn can_send(&mut self) -> bool {
        unsafe {
            (intrinsics::volatile_load(DBGU_SR) & DBGU_SR_TXRDY) != 0
        }
    }

    fn send(&mut self) {
        if self.send_start == self.send_end {
            return;
        }
        unsafe {
            intrinsics::volatile_store(DBGU_THR, self.send_buffer[self.send_start as usize] as u32);
        }
        self.send_start = (self.send_start + 1) & 2047;
    }

    fn can_receive(&mut self) -> bool {
        unsafe {
            (intrinsics::volatile_load(DBGU_SR) & DBGU_SR_RXRDY) != 0
        }
    }

    fn receive(&mut self) {
        let next_index = (self.receive_end + 1) & 2047;
        if next_index == self.receive_start {
            return;
        }
        unsafe {
            self.receive_buffer[self.receive_end as usize] = intrinsics::volatile_load(DBGU_RHR) as u8;
        }
        self.receive_end = next_index;
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            if c == '\n' as u8 {
                self.write_byte('\r' as u8);
            }
            self.write_byte(c);
        }
        Ok(())
    }
}

