use crate::interrupts::{Interrupts, InterruptType};

const DIVIDER_REGISTER: u16 = 0xFF04;
const TIMER_COUNTER: u16 = 0xFF05;
const TIMER_MODULO: u16 = 0xFF06;
const TIMER_CONTROL: u16 = 0xFF07;

pub struct Timer {
    divider: u16,
    counter: u8,
    modulo: u8,
    control: u8,
    counter_cycles: u32,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            divider: 0,
            counter: 0,
            modulo: 0,
            control: 0,
            counter_cycles: 0,
        }
    }

    pub fn tick(&mut self, cycles: u32, interrupts: &mut Interrupts) {
        self.divider = self.divider.wrapping_add(cycles as u16);

        if !self.is_enabled() {
            return;
        }

        self.counter_cycles += cycles;
        let threshold = self.get_frequency_threshold();

        while self.counter_cycles >= threshold {
            self.counter_cycles -= threshold;
            self.counter = self.counter.wrapping_add(1);

            if self.counter == 0 {
                self.counter = self.modulo;
                interrupts.request_interrupt(InterruptType::Timer);
            }
        }
    }

    fn is_enabled(&self) -> bool {
        self.control & 0b00000100 != 0
    }

    fn get_frequency_threshold(&self) -> u32 {
        match self.control & 0b00000011 {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => unreachable!(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            DIVIDER_REGISTER => (self.divider >> 8) as u8,
            TIMER_COUNTER => self.counter,
            TIMER_MODULO => self.modulo,
            TIMER_CONTROL => self.control,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            DIVIDER_REGISTER => self.divider = 0,
            TIMER_COUNTER => self.counter = value,
            TIMER_MODULO => self.modulo = value,
            TIMER_CONTROL => self.control = value & 0b00000111,
            _ => {}
        }
    }
}

