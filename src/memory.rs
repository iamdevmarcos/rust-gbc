use crate::mbc::{MemoryBankController, create_mbc};
use crate::bootrom;

pub struct Memory {
    mbc: Box<dyn MemoryBankController>,
    wram: [u8; 0x2000],
    hram: [u8; 0x7F],
    boot_rom_enabled: bool,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            mbc: create_mbc(vec![0; 0x8000]),
            wram: [0; 0x2000],
            hram: [0; 0x7F],
            boot_rom_enabled: true,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x00FF if self.boot_rom_enabled => {
                bootrom::get_boot_rom()[addr as usize]
            }
            0x0000..=0x7FFF => self.mbc.read_rom(addr),
            0xA000..=0xBFFF => self.mbc.read_ram(addr),
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize],
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            _ => 0xFF,
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => self.mbc.write_rom(addr, value),
            0xA000..=0xBFFF => self.mbc.write_ram(addr, value),
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize] = value,
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize] = value,
            0xFF50 if value != 0 => {
                self.boot_rom_enabled = false;
            }
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = value,
            _ => {}
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.mbc = create_mbc(rom.to_vec());
    }
}


