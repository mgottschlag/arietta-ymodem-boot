
use core::intrinsics;

const PIT_MR: *mut u32 = 0xFFFFFE30 as *mut u32;

const PIT_MR_PITEN : u32 = 1 << 24;
const PIT_MR_PIV_MASK : u32 = 0xfffff;

const PIT_PIIR: *mut u32 = 0xFFFFFE3C as *mut u32;

const PMC_PCR: *mut u32 = 0xFFFFFD0C as *mut u32;

const PMC_PCR_EN: u32 = 1 << 28;
const PMC_PCR_CMD_WRITE: u32 = 1 << 12;
const PERIPH_ID_SYS: u32 = 1;

fn usecs_to_pit_steps(usecs: u32) -> u32 {
    // MCLK is around 133MHz, and the clock is further divided by 16 before
    // passed to the PIT. Therefore, one microsecond equals around 133/16
    // counter steps.
    usecs * 133 / 16
}

pub struct Timer {
    duration: u32,
    start: u32,
}

impl Timer {
    pub fn init() {
        unsafe {
            intrinsics::volatile_store(PIT_MR, PIT_MR_PIV_MASK | PIT_MR_PITEN);
            // Enable the SYS clock.
            intrinsics::volatile_store(PMC_PCR, PMC_PCR_EN | PMC_PCR_CMD_WRITE | PERIPH_ID_SYS);
        }
    }
    pub fn new(usecs: u32) -> Timer {
        unsafe {
            Timer{
                duration: usecs_to_pit_steps(usecs),
                start: intrinsics::volatile_load(PIT_PIIR),
            }
        }
    }

    pub fn reset(&mut self) {
        unsafe {
            self.start = intrinsics::volatile_load(PIT_PIIR)
        }
    }

    pub fn expired(&mut self) -> bool {
        unsafe {
            (intrinsics::volatile_load(PIT_PIIR) - self.start) >= self.duration
        }
    }
}
