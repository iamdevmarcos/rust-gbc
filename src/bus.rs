use crate::memory::Memory;
use crate::ppu::PPU;
use crate::timer::Timer;
use crate::joypad::Joypad;
use crate::interrupts::Interrupts;

pub struct MemoryBus {
    pub memory: Memory,
    pub ppu: PPU,
    pub timer: Timer,
    pub joypad: Joypad,
    pub interrupts: Interrupts,
}

impl MemoryBus {
    pub fn new() -> Self {
        MemoryBus {
            memory: Memory::new(),
            ppu: PPU::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            interrupts: Interrupts::new(),
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF40..=0xFF4B => self.ppu.read(addr),
            0xFF04..=0xFF07 => self.timer.read(addr),
            0xFF00 => self.joypad.read(),
            0xFF0F => self.interrupts.interrupt_flag,
            0xFFFF => self.interrupts.interrupt_enable,
            _ => self.memory.read_byte(addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF46 => self.dma_transfer(value),
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF40..=0xFF4B => self.ppu.write(addr, value),
            0xFF04..=0xFF07 => self.timer.write(addr, value),
            0xFF00 => self.joypad.write(value),
            0xFF0F => self.interrupts.interrupt_flag = value,
            0xFFFF => self.interrupts.interrupt_enable = value,
            _ => self.memory.write_byte(addr, value),
        }
    }

    fn dma_transfer(&mut self, value: u8) {
        let source = (value as u16) << 8;
        for i in 0..0xA0 {
            let byte = self.read_byte(source + i);
            self.ppu.write(0xFE00 + i, byte);
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.memory.load_rom(rom);
    }
}

