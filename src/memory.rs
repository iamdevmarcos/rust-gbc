const MEMORY_64KB: usize = 65536;
const ROM_START_ADDR: usize = 0x0000;

pub struct Memory {
  pub data: [u8; MEMORY_64KB]
}

impl Memory {
    pub fn new() -> Self {
      Memory {
        data: [0; MEMORY_64KB],
      }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
      self.data[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
      self.data[addr as usize] = value;
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
      let length = rom.len();
      self.data[ROM_START_ADDR..length].copy_from_slice(rom);
    }
}