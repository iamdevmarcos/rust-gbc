pub trait MemoryBankController {
    fn read_rom(&self, addr: u16) -> u8;
    fn write_rom(&mut self, addr: u16, value: u8);
    fn read_ram(&self, addr: u16) -> u8;
    fn write_ram(&mut self, addr: u16, value: u8);
}

pub struct NoMBC {
    rom: Vec<u8>,
}

impl NoMBC {
    pub fn new(rom: Vec<u8>) -> Self {
        NoMBC { rom }
    }
}

impl MemoryBankController for NoMBC {
    fn read_rom(&self, addr: u16) -> u8 {
        if (addr as usize) < self.rom.len() {
            self.rom[addr as usize]
        } else {
            0xFF
        }
    }

    fn write_rom(&mut self, _addr: u16, _value: u8) {}

    fn read_ram(&self, _addr: u16) -> u8 {
        0xFF
    }

    fn write_ram(&mut self, _addr: u16, _value: u8) {}
}

pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
    banking_mode: u8,
}

impl MBC1 {
    pub fn new(rom: Vec<u8>) -> Self {
        let ram_size = 32 * 1024;
        MBC1 {
            rom,
            ram: vec![0; ram_size],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            banking_mode: 0,
        }
    }
}

impl MemoryBankController for MBC1 {
    fn read_rom(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                let bank = if self.banking_mode == 1 {
                    (self.ram_bank << 5) % (self.rom.len() / 0x4000)
                } else {
                    0
                };
                let rom_addr = bank * 0x4000 + (addr as usize);
                if rom_addr < self.rom.len() {
                    self.rom[rom_addr]
                } else {
                    0xFF
                }
            }
            0x4000..=0x7FFF => {
                let bank = if self.banking_mode == 1 {
                    ((self.ram_bank << 5) | self.rom_bank) % (self.rom.len() / 0x4000)
                } else {
                    self.rom_bank
                };
                let rom_addr = bank * 0x4000 + ((addr - 0x4000) as usize);
                if rom_addr < self.rom.len() {
                    self.rom[rom_addr]
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    fn write_rom(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                let bank = (value & 0x1F) as usize;
                self.rom_bank = if bank == 0 { 1 } else { bank };
            }
            0x4000..=0x5FFF => {
                self.ram_bank = (value & 0x03) as usize;
            }
            0x6000..=0x7FFF => {
                self.banking_mode = value & 0x01;
            }
            _ => {}
        }
    }

    fn read_ram(&self, addr: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }
        let bank = if self.banking_mode == 1 {
            self.ram_bank
        } else {
            0
        };
        let ram_addr = bank * 0x2000 + ((addr - 0xA000) as usize);
        if ram_addr < self.ram.len() {
            self.ram[ram_addr]
        } else {
            0xFF
        }
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }
        let bank = if self.banking_mode == 1 {
            self.ram_bank
        } else {
            0
        };
        let ram_addr = bank * 0x2000 + ((addr - 0xA000) as usize);
        if ram_addr < self.ram.len() {
            self.ram[ram_addr] = value;
        }
    }
}

pub struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
    rtc_enabled: bool,
}

impl MBC3 {
    pub fn new(rom: Vec<u8>) -> Self {
        let ram_size = 32 * 1024;
        MBC3 {
            rom,
            ram: vec![0; ram_size],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            rtc_enabled: false,
        }
    }
}

impl MemoryBankController for MBC3 {
    fn read_rom(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                if (addr as usize) < self.rom.len() {
                    self.rom[addr as usize]
                } else {
                    0xFF
                }
            }
            0x4000..=0x7FFF => {
                let rom_addr = (self.rom_bank * 0x4000) + ((addr - 0x4000) as usize);
                if rom_addr < self.rom.len() {
                    self.rom[rom_addr]
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    fn write_rom(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                let bank = (value & 0x7F) as usize;
                self.rom_bank = if bank == 0 { 1 } else { bank };
            }
            0x4000..=0x5FFF => {
                if value <= 0x03 {
                    self.ram_bank = value as usize;
                    self.rtc_enabled = false;
                } else if value >= 0x08 && value <= 0x0C {
                    self.rtc_enabled = true;
                }
            }
            0x6000..=0x7FFF => {
            }
            _ => {}
        }
    }

    fn read_ram(&self, addr: u16) -> u8 {
        if !self.ram_enabled || self.rtc_enabled {
            return 0xFF;
        }

        let ram_addr = (self.ram_bank * 0x2000) + ((addr - 0xA000) as usize);
        if ram_addr < self.ram.len() {
            self.ram[ram_addr]
        } else {
            0xFF
        }
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled || self.rtc_enabled {
            return;
        }

        let ram_addr = (self.ram_bank * 0x2000) + ((addr - 0xA000) as usize);
        if ram_addr < self.ram.len() {
            self.ram[ram_addr] = value;
        }
    }
}

pub fn create_mbc(rom: Vec<u8>) -> Box<dyn MemoryBankController> {
    if rom.len() > 0x147 {
        let cartridge_type = rom[0x147];
        match cartridge_type {
            0x00 => Box::new(NoMBC::new(rom)),
            0x01 | 0x02 | 0x03 => Box::new(MBC1::new(rom)),
            0x0F..=0x13 => Box::new(MBC3::new(rom)),
            _ => {
                println!("Warning: Unsupported cartridge type 0x{:02X}, using MBC1", cartridge_type);
                Box::new(MBC1::new(rom))
            }
        }
    } else {
        Box::new(NoMBC::new(rom))
    }
}

