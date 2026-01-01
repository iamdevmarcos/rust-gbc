use crate::bus::MemoryBus;

const INITIAL_PC: u16 = 0x0100;
const INITIAL_SP: u16 = 0xFFFE;

const ZERO_FLAG: u8 = 0b1000_0000;
const SUBTRACT_FLAG: u8 = 0b0100_0000;
const HALF_CARRY_FLAG: u8 = 0b0010_0000;
const CARRY_FLAG: u8 = 0b0001_0000;

pub struct CPU {
  pub a: u8,
  pub b: u8,
  pub c: u8,
  pub d: u8,
  pub e: u8,
  pub h: u8,
  pub l: u8,
  pub f: u8,
  pub pc: u16,
  pub sp: u16,
  pub bus: MemoryBus,
  pub cycles: u64,
  pub halted: bool,
}

impl CPU {
  pub fn new() -> Self {
    CPU {
      a: 0,
      b: 0,
      c: 0,
      d: 0,
      e: 0,
      h: 0,
      l: 0,
      f: 0,
      pc: INITIAL_PC,
      sp: INITIAL_SP,
      bus: MemoryBus::new(),
      cycles: 0,
      halted: false,
    }
  }

  // Helper methods for 16-bit register pairs
  pub fn get_bc(&self) -> u16 {
    ((self.b as u16) << 8) | (self.c as u16)
  }

  pub fn set_bc(&mut self, value: u16) {
    self.b = (value >> 8) as u8;
    self.c = (value & 0xFF) as u8;
  }

  pub fn get_de(&self) -> u16 {
    ((self.d as u16) << 8) | (self.e as u16)
  }

  pub fn set_de(&mut self, value: u16) {
    self.d = (value >> 8) as u8;
    self.e = (value & 0xFF) as u8;
  }

  pub fn get_hl(&self) -> u16 {
    ((self.h as u16) << 8) | (self.l as u16)
  }

  pub fn set_hl(&mut self, value: u16) {
    self.h = (value >> 8) as u8;
    self.l = (value & 0xFF) as u8;
  }

  pub fn get_af(&self) -> u16 {
    ((self.a as u16) << 8) | (self.f as u16)
  }

  pub fn set_af(&mut self, value: u16) {
    self.a = (value >> 8) as u8;
    self.f = (value & 0xF0) as u8; // Lower 4 bits of F are always 0
  }

  pub fn set_zero_flag(&mut self, value: bool) {
    if value {
      self.f |= ZERO_FLAG;
    } else {
      self.f &= !ZERO_FLAG;
    }
  }

  pub fn set_half_carry_flag(&mut self, value: bool) {
    if value {
      self.f |= HALF_CARRY_FLAG;
    } else {
      self.f &= !HALF_CARRY_FLAG;
    }
  }

  pub fn set_carry_flag(&mut self, value: bool) {
    if value {
      self.f |= CARRY_FLAG;
    } else {
      self.f &= !CARRY_FLAG;
    }
  }

  pub fn is_zero_flag_set(&self) -> bool {
    self.f & ZERO_FLAG != 0
  }

  pub fn is_carry_flag_set(&self) -> bool {
    self.f & CARRY_FLAG != 0
  }

  pub fn is_half_carry_flag_set(&self) -> bool {
    self.f & HALF_CARRY_FLAG != 0
  }

  pub fn is_subtract_flag_set(&self) -> bool {
    self.f & SUBTRACT_FLAG != 0
  }

  pub fn set_subtract_flag(&mut self, value: bool) {
    if value {
      self.f |= SUBTRACT_FLAG;
    } else {
      self.f &= !SUBTRACT_FLAG;
    }
  }

  pub fn show_cpu_state(&self) {
    println!("A: {:02X}, B: {:02X}, C: {:02X}", self.a, self.b, self.c);
    println!("D: {:02X}, E: {:02X}, H: {:02X}, L: {:02X}, F: {:02X}", self.d, self.e, self.h, self.l, self.f);
    println!("PC: {:04X}, SP: {:04X}", self.pc, self.sp);
  }

  pub fn load_a(&mut self, value: u8) {
    self.a = value;
  }

  pub fn load_b(&mut self, value: u8) {
    self.b = value;
  }

  pub fn load_c(&mut self, value: u8) {
    self.c = value;
  }

  pub fn load_d(&mut self, value: u8) {
    self.d = value;
  }

  pub fn load_e(&mut self, value: u8) {
    self.e = value;
  }

  pub fn load_h(&mut self, value: u8) {
    self.h = value;
  }

  pub fn load_l(&mut self, value: u8) {
    self.l = value;
  }

  pub fn add_to_a(&mut self, value: u8) {
    let (result, carry) = self.a.overflowing_add(value);
    self.a = result;

    self.set_zero_flag(self.a == 0);
    self.set_carry_flag(carry);
    self.set_half_carry_flag((self.a & 0x0F) + (value & 0x0F) > 0x0F);
    self.set_subtract_flag(false);
  }

  pub fn sub_from_a(&mut self, value: u8) {
    let (result, carry) = self.a.overflowing_sub(value);
    self.a = result;

    self.set_zero_flag(self.a == 0);
    self.set_carry_flag(carry);
    self.set_half_carry_flag((self.a & 0x0F) + (value & 0x0F) > 0x0F);
    self.set_subtract_flag(true);
  }

  pub fn execute_instruction(&mut self, opcode: u8) -> u32 {
    match opcode {
      // 8-bit loads: LD r, n
      0x3E => {
        let value = self.fetch_byte();
        self.load_a(value);
        8
      }
      0x06 => {
        let value = self.fetch_byte();
        self.load_b(value);
        8
      }
      0x0E => {
        let value = self.fetch_byte();
        self.load_c(value);
        8
      }
      0x16 => {
        let value = self.fetch_byte();
        self.load_d(value);
        8
      }
      0x1E => {
        let value = self.fetch_byte();
        self.load_e(value);
        8
      }
      0x26 => {
        let value = self.fetch_byte();
        self.load_h(value);
        8
      }
      0x2E => {
        let value = self.fetch_byte();
        self.load_l(value);
        8
      }

      // 8-bit loads: LD r, r'
      0x7F => { self.a = self.a; 4 } // LD A, A
      0x78 => { self.a = self.b; 4 } // LD A, B
      0x79 => { self.a = self.c; 4 } // LD A, C
      0x7A => { self.a = self.d; 4 } // LD A, D
      0x7B => { self.a = self.e; 4 } // LD A, E
      0x7C => { self.a = self.h; 4 } // LD A, H
      0x7D => { self.a = self.l; 4 } // LD A, L

      0x47 => { self.b = self.a; 4 } // LD B, A
      0x40 => { self.b = self.b; 4 } // LD B, B
      0x41 => { self.b = self.c; 4 } // LD B, C
      0x42 => { self.b = self.d; 4 } // LD B, D
      0x43 => { self.b = self.e; 4 } // LD B, E
      0x44 => { self.b = self.h; 4 } // LD B, H
      0x45 => { self.b = self.l; 4 } // LD B, L

      0x4F => { self.c = self.a; 4 } // LD C, A
      0x48 => { self.c = self.b; 4 } // LD C, B
      0x49 => { self.c = self.c; 4 } // LD C, C
      0x4A => { self.c = self.d; 4 } // LD C, D
      0x4B => { self.c = self.e; 4 } // LD C, E
      0x4C => { self.c = self.h; 4 } // LD C, H
      0x4D => { self.c = self.l; 4 } // LD C, L

      0x57 => { self.d = self.a; 4 } // LD D, A
      0x50 => { self.d = self.b; 4 } // LD D, B
      0x51 => { self.d = self.c; 4 } // LD D, C
      0x52 => { self.d = self.d; 4 } // LD D, D
      0x53 => { self.d = self.e; 4 } // LD D, E
      0x54 => { self.d = self.h; 4 } // LD D, H
      0x55 => { self.d = self.l; 4 } // LD D, L

      0x5F => { self.e = self.a; 4 } // LD E, A
      0x58 => { self.e = self.b; 4 } // LD E, B
      0x59 => { self.e = self.c; 4 } // LD E, C
      0x5A => { self.e = self.d; 4 } // LD E, D
      0x5B => { self.e = self.e; 4 } // LD E, E
      0x5C => { self.e = self.h; 4 } // LD E, H
      0x5D => { self.e = self.l; 4 } // LD E, L

      0x67 => { self.h = self.a; 4 } // LD H, A
      0x60 => { self.h = self.b; 4 } // LD H, B
      0x61 => { self.h = self.c; 4 } // LD H, C
      0x62 => { self.h = self.d; 4 } // LD H, D
      0x63 => { self.h = self.e; 4 } // LD H, E
      0x64 => { self.h = self.h; 4 } // LD H, H
      0x65 => { self.h = self.l; 4 } // LD H, L

      0x6F => { self.l = self.a; 4 } // LD L, A
      0x68 => { self.l = self.b; 4 } // LD L, B
      0x69 => { self.l = self.c; 4 } // LD L, C
      0x6A => { self.l = self.d; 4 } // LD L, D
      0x6B => { self.l = self.e; 4 } // LD L, E
      0x6C => { self.l = self.h; 4 } // LD L, H
      0x6D => { self.l = self.l; 4 } // LD L, L

      // LD r, (HL)
      0x7E => { self.a = self.bus.read_byte(self.get_hl()); 8 }
      0x46 => { self.b = self.bus.read_byte(self.get_hl()); 8 }
      0x4E => { self.c = self.bus.read_byte(self.get_hl()); 8 }
      0x56 => { self.d = self.bus.read_byte(self.get_hl()); 8 }
      0x5E => { self.e = self.bus.read_byte(self.get_hl()); 8 }
      0x66 => { self.h = self.bus.read_byte(self.get_hl()); 8 }
      0x6E => { self.l = self.bus.read_byte(self.get_hl()); 8 }

      // LD (HL), r
      0x77 => { self.bus.write_byte(self.get_hl(), self.a); 8 }
      0x70 => { self.bus.write_byte(self.get_hl(), self.b); 8 }
      0x71 => { self.bus.write_byte(self.get_hl(), self.c); 8 }
      0x72 => { self.bus.write_byte(self.get_hl(), self.d); 8 }
      0x73 => { self.bus.write_byte(self.get_hl(), self.e); 8 }
      0x74 => { self.bus.write_byte(self.get_hl(), self.h); 8 }
      0x75 => { self.bus.write_byte(self.get_hl(), self.l); 8 }
      0x36 => {
        let value = self.fetch_byte();
        self.bus.write_byte(self.get_hl(), value);
        12
      }

      // LD A, (BC/DE/nn)
      0x0A => { self.a = self.bus.read_byte(self.get_bc()); 8 }
      0x1A => { self.a = self.bus.read_byte(self.get_de()); 8 }
      0xFA => {
        let addr = self.fetch_word();
        self.a = self.bus.read_byte(addr);
        16
      }

      // LD (BC/DE/nn), A
      0x02 => { self.bus.write_byte(self.get_bc(), self.a); 8 }
      0x12 => { self.bus.write_byte(self.get_de(), self.a); 8 }
      0xEA => {
        let addr = self.fetch_word();
        self.bus.write_byte(addr, self.a);
        16
      }

      // LD A, (C) - Load A from address 0xFF00 + C
      0xF2 => {
        let addr = 0xFF00 + (self.c as u16);
        self.a = self.bus.read_byte(addr);
        8
      }

      // LD (C), A - Store A at address 0xFF00 + C
      0xE2 => {
        let addr = 0xFF00 + (self.c as u16);
        self.bus.write_byte(addr, self.a);
        8
      }

      // LDD A, (HL) - Load A from (HL) and decrement HL
      0x3A => {
        self.a = self.bus.read_byte(self.get_hl());
        self.set_hl(self.get_hl().wrapping_sub(1));
        8
      }

      // LDD (HL), A - Store A at (HL) and decrement HL
      0x32 => {
        self.bus.write_byte(self.get_hl(), self.a);
        self.set_hl(self.get_hl().wrapping_sub(1));
        8
      }

      // LDI A, (HL) - Load A from (HL) and increment HL
      0x2A => {
        self.a = self.bus.read_byte(self.get_hl());
        self.set_hl(self.get_hl().wrapping_add(1));
        8
      }

      // LDI (HL), A - Store A at (HL) and increment HL
      0x22 => {
        self.bus.write_byte(self.get_hl(), self.a);
        self.set_hl(self.get_hl().wrapping_add(1));
        8
      }

      // LDH (n), A - Store A at address 0xFF00 + n
      0xE0 => {
        let offset = self.fetch_byte();
        let addr = 0xFF00 + (offset as u16);
        self.bus.write_byte(addr, self.a);
        12
      }

      // LDH A, (n) - Load A from address 0xFF00 + n
      0xF0 => {
        let offset = self.fetch_byte();
        let addr = 0xFF00 + (offset as u16);
        self.a = self.bus.read_byte(addr);
        12
      }

      // 16-bit loads
      0x01 => {
        let value = self.fetch_word();
        self.set_bc(value);
        12
      }
      0x11 => {
        let value = self.fetch_word();
        self.set_de(value);
        12
      }
      0x21 => {
        let value = self.fetch_word();
        self.set_hl(value);
        12
      }
      0x31 => {
        self.sp = self.fetch_word();
        12
      }

      // LD SP, HL
      0xF9 => {
        self.sp = self.get_hl();
        8
      }

      // LD (nn), SP
      0x08 => {
        let addr = self.fetch_word();
        let sp = self.sp;
        self.bus.write_byte(addr, (sp & 0xFF) as u8);
        self.bus.write_byte(addr + 1, (sp >> 8) as u8);
        20
      }

      // PUSH
      0xF5 => { self.push(self.get_af()); 16 } // PUSH AF
      0xC5 => { self.push(self.get_bc()); 16 } // PUSH BC
      0xD5 => { self.push(self.get_de()); 16 } // PUSH DE
      0xE5 => { self.push(self.get_hl()); 16 } // PUSH HL

      // POP
      0xF1 => { let val = self.pop(); self.set_af(val); 12 } // POP AF
      0xC1 => { let val = self.pop(); self.set_bc(val); 12 } // POP BC
      0xD1 => { let val = self.pop(); self.set_de(val); 12 } // POP DE
      0xE1 => { let val = self.pop(); self.set_hl(val); 12 } // POP HL

      // 8-bit arithmetic: ADD A, r
      0x87 => { self.add_to_a(self.a); 4 }
      0x80 => { self.add_to_a(self.b); 4 }
      0x81 => { self.add_to_a(self.c); 4 }
      0x82 => { self.add_to_a(self.d); 4 }
      0x83 => { self.add_to_a(self.e); 4 }
      0x84 => { self.add_to_a(self.h); 4 }
      0x85 => { self.add_to_a(self.l); 4 }
      0x86 => {
        let value = self.bus.read_byte(self.get_hl());
        self.add_to_a(value);
        8
      }
      0xC6 => {
        let value = self.fetch_byte();
        self.add_to_a(value);
        8
      }

      // ADC A, r (Add with carry)
      0x8F => { let a = self.a; self.adc(a); 4 }
      0x88 => { let b = self.b; self.adc(b); 4 }
      0x89 => { let c = self.c; self.adc(c); 4 }
      0x8A => { let d = self.d; self.adc(d); 4 }
      0x8B => { let e = self.e; self.adc(e); 4 }
      0x8C => { let h = self.h; self.adc(h); 4 }
      0x8D => { let l = self.l; self.adc(l); 4 }
      0x8E => {
        let value = self.bus.read_byte(self.get_hl());
        self.adc(value);
        8
      }
      0xCE => {
        let value = self.fetch_byte();
        self.adc(value);
        8
      }

      // SUB A, r
      0x97 => { let a = self.a; self.sub_from_a(a); 4 }
      0x90 => { self.sub_from_a(self.b); 4 }
      0x91 => { self.sub_from_a(self.c); 4 }
      0x92 => { self.sub_from_a(self.d); 4 }
      0x93 => { self.sub_from_a(self.e); 4 }
      0x94 => { self.sub_from_a(self.h); 4 }
      0x95 => { self.sub_from_a(self.l); 4 }
      0x96 => {
        let value = self.bus.read_byte(self.get_hl());
        self.sub_from_a(value);
        8
      }
      0xD6 => {
        let value = self.fetch_byte();
        self.sub_from_a(value);
        8
      }

      // SBC A, r (Subtract with carry)
      0x9F => { let a = self.a; self.sbc(a); 4 }
      0x98 => { let b = self.b; self.sbc(b); 4 }
      0x99 => { let c = self.c; self.sbc(c); 4 }
      0x9A => { let d = self.d; self.sbc(d); 4 }
      0x9B => { let e = self.e; self.sbc(e); 4 }
      0x9C => { let h = self.h; self.sbc(h); 4 }
      0x9D => { let l = self.l; self.sbc(l); 4 }
      0x9E => {
        let value = self.bus.read_byte(self.get_hl());
        self.sbc(value);
        8
      }
      0xDE => {
        let value = self.fetch_byte();
        self.sbc(value);
        8
      }

      // AND r
      0xA7 => { self.and(self.a); 4 }
      0xA0 => { self.and(self.b); 4 }
      0xA1 => { self.and(self.c); 4 }
      0xA2 => { self.and(self.d); 4 }
      0xA3 => { self.and(self.e); 4 }
      0xA4 => { self.and(self.h); 4 }
      0xA5 => { self.and(self.l); 4 }
      0xA6 => {
        let value = self.bus.read_byte(self.get_hl());
        self.and(value);
        8
      }
      0xE6 => {
        let value = self.fetch_byte();
        self.and(value);
        8
      }

      // OR r
      0xB7 => { self.or(self.a); 4 }
      0xB0 => { self.or(self.b); 4 }
      0xB1 => { self.or(self.c); 4 }
      0xB2 => { self.or(self.d); 4 }
      0xB3 => { self.or(self.e); 4 }
      0xB4 => { self.or(self.h); 4 }
      0xB5 => { self.or(self.l); 4 }
      0xB6 => {
        let value = self.bus.read_byte(self.get_hl());
        self.or(value);
        8
      }
      0xF6 => {
        let value = self.fetch_byte();
        self.or(value);
        8
      }

      // XOR r
      0xAF => { self.xor(self.a); 4 }
      0xA8 => { self.xor(self.b); 4 }
      0xA9 => { self.xor(self.c); 4 }
      0xAA => { self.xor(self.d); 4 }
      0xAB => { self.xor(self.e); 4 }
      0xAC => { self.xor(self.h); 4 }
      0xAD => { self.xor(self.l); 4 }
      0xAE => {
        let value = self.bus.read_byte(self.get_hl());
        self.xor(value);
        8
      }
      0xEE => {
        let value = self.fetch_byte();
        self.xor(value);
        8
      }

      // CP r (Compare - like SUB but doesn't store result)
      0xBF => { self.cp(self.a); 4 }
      0xB8 => { self.cp(self.b); 4 }
      0xB9 => { self.cp(self.c); 4 }
      0xBA => { self.cp(self.d); 4 }
      0xBB => { self.cp(self.e); 4 }
      0xBC => { self.cp(self.h); 4 }
      0xBD => { self.cp(self.l); 4 }
      0xBE => {
        let value = self.bus.read_byte(self.get_hl());
        self.cp(value);
        8
      }
      0xFE => {
        let value = self.fetch_byte();
        self.cp(value);
        8
      }

      // INC r
      0x3C => { self.a = self.inc(self.a); 4 }
      0x04 => { self.b = self.inc(self.b); 4 }
      0x0C => { self.c = self.inc(self.c); 4 }
      0x14 => { self.d = self.inc(self.d); 4 }
      0x1C => { self.e = self.inc(self.e); 4 }
      0x24 => { self.h = self.inc(self.h); 4 }
      0x2C => { self.l = self.inc(self.l); 4 }
      0x34 => {
        let addr = self.get_hl();
        let value = self.bus.read_byte(addr);
        let result = self.inc(value);
        self.bus.write_byte(addr, result);
        12
      }

      // DEC r
      0x3D => { self.a = self.dec(self.a); 4 }
      0x05 => { self.b = self.dec(self.b); 4 }
      0x0D => { self.c = self.dec(self.c); 4 }
      0x15 => { self.d = self.dec(self.d); 4 }
      0x1D => { self.e = self.dec(self.e); 4 }
      0x25 => { self.h = self.dec(self.h); 4 }
      0x2D => { self.l = self.dec(self.l); 4 }
      0x35 => {
        let addr = self.get_hl();
        let value = self.bus.read_byte(addr);
        let result = self.dec(value);
        self.bus.write_byte(addr, result);
        12
      }

      // 16-bit arithmetic
      // ADD HL, rr
      0x09 => { let bc = self.get_bc(); self.add_hl(bc); 8 }
      0x19 => { let de = self.get_de(); self.add_hl(de); 8 }
      0x29 => { let hl = self.get_hl(); self.add_hl(hl); 8 }
      0x39 => { self.add_hl(self.sp); 8 }

      // ADD SP, n
      0xE8 => {
        let value = self.fetch_byte() as i8 as i16 as u16;
        let sp = self.sp;
        self.sp = sp.wrapping_add(value);
        
        self.set_zero_flag(false);
        self.set_subtract_flag(false);
        self.set_half_carry_flag((sp & 0x0F) + (value & 0x0F) > 0x0F);
        self.set_carry_flag((sp & 0xFF) + (value & 0xFF) > 0xFF);
        16
      }

      // INC rr
      0x03 => { let bc = self.get_bc(); self.set_bc(bc.wrapping_add(1)); 8 }
      0x13 => { let de = self.get_de(); self.set_de(de.wrapping_add(1)); 8 }
      0x23 => { let hl = self.get_hl(); self.set_hl(hl.wrapping_add(1)); 8 }
      0x33 => { self.sp = self.sp.wrapping_add(1); 8 }

      // DEC rr
      0x0B => { let bc = self.get_bc(); self.set_bc(bc.wrapping_sub(1)); 8 }
      0x1B => { let de = self.get_de(); self.set_de(de.wrapping_sub(1)); 8 }
      0x2B => { let hl = self.get_hl(); self.set_hl(hl.wrapping_sub(1)); 8 }
      0x3B => { self.sp = self.sp.wrapping_sub(1); 8 }

      // Jumps
      0xC3 => {
        let addr = self.fetch_word();
        self.pc = addr;
        16
      }
      0xC2 => { // JP NZ, nn
        let addr = self.fetch_word();
        if !self.is_zero_flag_set() {
          self.pc = addr;
          16
        } else {
          12
        }
      }
      0xCA => { // JP Z, nn
        let addr = self.fetch_word();
        if self.is_zero_flag_set() {
          self.pc = addr;
          16
        } else {
          12
        }
      }
      0xD2 => { // JP NC, nn
        let addr = self.fetch_word();
        if !self.is_carry_flag_set() {
          self.pc = addr;
          16
        } else {
          12
        }
      }
      0xDA => { // JP C, nn
        let addr = self.fetch_word();
        if self.is_carry_flag_set() {
          self.pc = addr;
          16
        } else {
          12
        }
      }
      0xE9 => { // JP (HL)
        self.pc = self.get_hl();
        4
      }

      // Relative jumps
      0x18 => {
        let offset = self.fetch_byte() as i8;
        self.pc = self.pc.wrapping_add(offset as u16);
        12
      }
      0x20 => { // JR NZ, n
        let offset = self.fetch_byte() as i8;
        if !self.is_zero_flag_set() {
          self.pc = self.pc.wrapping_add(offset as u16);
          12
        } else {
          8
        }
      }
      0x28 => { // JR Z, n
        let offset = self.fetch_byte() as i8;
        if self.is_zero_flag_set() {
          self.pc = self.pc.wrapping_add(offset as u16);
          12
        } else {
          8
        }
      }
      0x30 => { // JR NC, n
        let offset = self.fetch_byte() as i8;
        if !self.is_carry_flag_set() {
          self.pc = self.pc.wrapping_add(offset as u16);
          12
        } else {
          8
        }
      }
      0x38 => { // JR C, n
        let offset = self.fetch_byte() as i8;
        if self.is_carry_flag_set() {
          self.pc = self.pc.wrapping_add(offset as u16);
          12
        } else {
          8
        }
      }

      // Calls
      0xCD => {
        let addr = self.fetch_word();
        self.push(self.pc);
        self.pc = addr;
        24
      }
      0xC4 => { // CALL NZ, nn
        let addr = self.fetch_word();
        if !self.is_zero_flag_set() {
          self.push(self.pc);
          self.pc = addr;
          24
        } else {
          12
        }
      }
      0xCC => { // CALL Z, nn
        let addr = self.fetch_word();
        if self.is_zero_flag_set() {
          self.push(self.pc);
          self.pc = addr;
          24
        } else {
          12
        }
      }
      0xD4 => { // CALL NC, nn
        let addr = self.fetch_word();
        if !self.is_carry_flag_set() {
          self.push(self.pc);
          self.pc = addr;
          24
        } else {
          12
        }
      }
      0xDC => { // CALL C, nn
        let addr = self.fetch_word();
        if self.is_carry_flag_set() {
          self.push(self.pc);
          self.pc = addr;
          24
        } else {
          12
        }
      }

      // Returns
      0xC9 => {
        self.pc = self.pop();
        16
      }
      0xC0 => { // RET NZ
        if !self.is_zero_flag_set() {
          self.pc = self.pop();
          20
        } else {
          8
        }
      }
      0xC8 => { // RET Z
        if self.is_zero_flag_set() {
          self.pc = self.pop();
          20
        } else {
          8
        }
      }
      0xD0 => { // RET NC
        if !self.is_carry_flag_set() {
          self.pc = self.pop();
          20
        } else {
          8
        }
      }
      0xD8 => { // RET C
        if self.is_carry_flag_set() {
          self.pc = self.pop();
          20
        } else {
          8
        }
      }
      0xD9 => {
        self.pc = self.pop();
        16
      }

      // Restarts (RST)
      0xC7 => { self.push(self.pc); self.pc = 0x00; 16 }
      0xCF => { self.push(self.pc); self.pc = 0x08; 16 }
      0xD7 => { self.push(self.pc); self.pc = 0x10; 16 }
      0xDF => { self.push(self.pc); self.pc = 0x18; 16 }
      0xE7 => { self.push(self.pc); self.pc = 0x20; 16 }
      0xEF => { self.push(self.pc); self.pc = 0x28; 16 }
      0xF7 => { self.push(self.pc); self.pc = 0x30; 16 }
      0xFF => { self.push(self.pc); self.pc = 0x38; 16 }

      // Rotates & Shifts
      0x07 => { self.rlca(); 4 } // RLCA
      0x17 => { self.rla(); 4 }  // RLA
      0x0F => { self.rrca(); 4 } // RRCA
      0x1F => { self.rra(); 4 }  // RRA

      // Misc
      0x00 => 4, // NOP
      0x76 => { // HALT
        self.halted = true;
        4
      }
      0x10 => { // STOP
        self.fetch_byte(); // STOP is 2 bytes
        self.halted = true;
        4
      }
      0xF3 => {
        4
      }
      0xFB => {
        4
      }

      // DAA (Decimal Adjust Accumulator)
      0x27 => {
        self.daa();
        4
      }

      // CPL (Complement A)
      0x2F => {
        self.a = !self.a;
        self.set_subtract_flag(true);
        self.set_half_carry_flag(true);
        4
      }

      // CCF (Complement Carry Flag)
      0x3F => {
        self.set_carry_flag(!self.is_carry_flag_set());
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        4
      }

      // SCF (Set Carry Flag)
      0x37 => {
        self.set_carry_flag(true);
        self.set_subtract_flag(false);
        self.set_half_carry_flag(false);
        4
      }

      // CB prefix (extended instructions)
      0xCB => {
        let cb_opcode = self.fetch_byte();
        self.execute_cb_instruction(cb_opcode)
      }

      _ => {
        println!("Unimplemented opcode: {:02X} at PC: {:04X}", opcode, self.pc - 1);
        4
      }
    }
  }

  pub fn fetch_byte(&mut self) -> u8 {
    let byte = self.bus.read_byte(self.pc);
    self.pc += 1;
    byte
  }

  pub fn fetch_word(&mut self) -> u16 {
    let low = self.fetch_byte() as u16;
    let high = self.fetch_byte() as u16;
    (high << 8) | low
  }

  pub fn push(&mut self, value: u16) {
    self.sp = self.sp.wrapping_sub(1);
    self.bus.write_byte(self.sp, (value >> 8) as u8);
    self.sp = self.sp.wrapping_sub(1);
    self.bus.write_byte(self.sp, (value & 0xFF) as u8);
  }

  pub fn pop(&mut self) -> u16 {
    let low = self.bus.read_byte(self.sp) as u16;
    self.sp = self.sp.wrapping_add(1);
    let high = self.bus.read_byte(self.sp) as u16;
    self.sp = self.sp.wrapping_add(1);
    (high << 8) | low
  }

  // ADC - Add with carry
  pub fn adc(&mut self, value: u8) {
    let carry = if self.is_carry_flag_set() { 1 } else { 0 };
    let result = self.a as u16 + value as u16 + carry;
    
    self.set_zero_flag((result & 0xFF) == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag((self.a & 0x0F) + (value & 0x0F) + carry as u8 > 0x0F);
    self.set_carry_flag(result > 0xFF);
    
    self.a = result as u8;
  }

  // SBC - Subtract with carry
  pub fn sbc(&mut self, value: u8) {
    let carry = if self.is_carry_flag_set() { 1 } else { 0 };
    let result = (self.a as i16) - (value as i16) - (carry as i16);
    
    self.set_zero_flag((result & 0xFF) == 0);
    self.set_subtract_flag(true);
    self.set_half_carry_flag((self.a & 0x0F) < (value & 0x0F) + carry as u8);
    self.set_carry_flag(result < 0);
    
    self.a = result as u8;
  }

  // AND
  pub fn and(&mut self, value: u8) {
    self.a &= value;
    self.set_zero_flag(self.a == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(true);
    self.set_carry_flag(false);
  }

  // OR
  pub fn or(&mut self, value: u8) {
    self.a |= value;
    self.set_zero_flag(self.a == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(false);
  }

  // XOR
  pub fn xor(&mut self, value: u8) {
    self.a ^= value;
    self.set_zero_flag(self.a == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(false);
  }

  // CP - Compare
  pub fn cp(&mut self, value: u8) {
    let result = self.a.wrapping_sub(value);
    self.set_zero_flag(result == 0);
    self.set_subtract_flag(true);
    self.set_half_carry_flag((self.a & 0x0F) < (value & 0x0F));
    self.set_carry_flag(self.a < value);
  }

  // INC - Increment
  pub fn inc(&mut self, value: u8) -> u8 {
    let result = value.wrapping_add(1);
    self.set_zero_flag(result == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag((value & 0x0F) == 0x0F);
    result
  }

  // DEC - Decrement
  pub fn dec(&mut self, value: u8) -> u8 {
    let result = value.wrapping_sub(1);
    self.set_zero_flag(result == 0);
    self.set_subtract_flag(true);
    self.set_half_carry_flag((value & 0x0F) == 0);
    result
  }

  // ADD HL, rr
  pub fn add_hl(&mut self, value: u16) {
    let hl = self.get_hl();
    let result = hl.wrapping_add(value);
    
    self.set_subtract_flag(false);
    self.set_half_carry_flag((hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF);
    self.set_carry_flag(hl > 0xFFFF - value);
    
    self.set_hl(result);
  }

  // Rotate left through carry
  pub fn rlca(&mut self) {
    let carry = (self.a & 0x80) != 0;
    self.a = (self.a << 1) | (if carry { 1 } else { 0 });
    
    self.set_zero_flag(false);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(carry);
  }

  // Rotate left
  pub fn rla(&mut self) {
    let old_carry = if self.is_carry_flag_set() { 1 } else { 0 };
    let new_carry = (self.a & 0x80) != 0;
    self.a = (self.a << 1) | old_carry;
    
    self.set_zero_flag(false);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(new_carry);
  }

  // Rotate right through carry
  pub fn rrca(&mut self) {
    let carry = (self.a & 0x01) != 0;
    self.a = (self.a >> 1) | (if carry { 0x80 } else { 0 });
    
    self.set_zero_flag(false);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(carry);
  }

  // Rotate right
  pub fn rra(&mut self) {
    let old_carry = if self.is_carry_flag_set() { 0x80 } else { 0 };
    let new_carry = (self.a & 0x01) != 0;
    self.a = (self.a >> 1) | old_carry;
    
    self.set_zero_flag(false);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(new_carry);
  }

  // DAA - Decimal Adjust Accumulator (for BCD arithmetic)
  pub fn daa(&mut self) {
    let mut a = self.a as u16;
    
    if !self.is_subtract_flag_set() {
      if self.is_carry_flag_set() || a > 0x99 {
        a = a.wrapping_add(0x60);
        self.set_carry_flag(true);
      }
      if self.is_half_carry_flag_set() || (a & 0x0F) > 0x09 {
        a = a.wrapping_add(0x06);
      }
    } else {
      if self.is_carry_flag_set() {
        a = a.wrapping_sub(0x60);
      }
      if self.is_half_carry_flag_set() {
        a = a.wrapping_sub(0x06);
      }
    }
    
    self.a = a as u8;
    self.set_zero_flag(self.a == 0);
    self.set_half_carry_flag(false);
  }

  // CB-prefixed instructions
  pub fn execute_cb_instruction(&mut self, opcode: u8) -> u32 {
    match opcode {
      // RLC r - Rotate left
      0x07 => { self.a = self.rlc(self.a); 8 }
      0x00 => { self.b = self.rlc(self.b); 8 }
      0x01 => { self.c = self.rlc(self.c); 8 }
      0x02 => { self.d = self.rlc(self.d); 8 }
      0x03 => { self.e = self.rlc(self.e); 8 }
      0x04 => { self.h = self.rlc(self.h); 8 }
      0x05 => { self.l = self.rlc(self.l); 8 }
      0x06 => {
        let addr = self.get_hl();
        let value = self.bus.read_byte(addr);
        let result = self.rlc(value);
        self.bus.write_byte(addr, result);
        16
      }

      // RRC r - Rotate right
      0x0F => { self.a = self.rrc(self.a); 8 }
      0x08 => { self.b = self.rrc(self.b); 8 }
      0x09 => { self.c = self.rrc(self.c); 8 }
      0x0A => { self.d = self.rrc(self.d); 8 }
      0x0B => { self.e = self.rrc(self.e); 8 }
      0x0C => { self.h = self.rrc(self.h); 8 }
      0x0D => { self.l = self.rrc(self.l); 8 }
      0x0E => {
        let addr = self.get_hl();
        let value = self.bus.read_byte(addr);
        let result = self.rrc(value);
        self.bus.write_byte(addr, result);
        16
      }

      // RL r - Rotate left through carry
      0x17 => { self.a = self.rl(self.a); 8 }
      0x10 => { self.b = self.rl(self.b); 8 }
      0x11 => { self.c = self.rl(self.c); 8 }
      0x12 => { self.d = self.rl(self.d); 8 }
      0x13 => { self.e = self.rl(self.e); 8 }
      0x14 => { self.h = self.rl(self.h); 8 }
      0x15 => { self.l = self.rl(self.l); 8 }
      0x16 => {
        let addr = self.get_hl();
        let value = self.bus.read_byte(addr);
        let result = self.rl(value);
        self.bus.write_byte(addr, result);
        16
      }

      // RR r - Rotate right through carry
      0x1F => { self.a = self.rr(self.a); 8 }
      0x18 => { self.b = self.rr(self.b); 8 }
      0x19 => { self.c = self.rr(self.c); 8 }
      0x1A => { self.d = self.rr(self.d); 8 }
      0x1B => { self.e = self.rr(self.e); 8 }
      0x1C => { self.h = self.rr(self.h); 8 }
      0x1D => { self.l = self.rr(self.l); 8 }
      0x1E => {
        let addr = self.get_hl();
        let value = self.bus.read_byte(addr);
        let result = self.rr(value);
        self.bus.write_byte(addr, result);
        16
      }

      // SLA r - Shift left arithmetic
      0x27 => { self.a = self.sla(self.a); 8 }
      0x20 => { self.b = self.sla(self.b); 8 }
      0x21 => { self.c = self.sla(self.c); 8 }
      0x22 => { self.d = self.sla(self.d); 8 }
      0x23 => { self.e = self.sla(self.e); 8 }
      0x24 => { self.h = self.sla(self.h); 8 }
      0x25 => { self.l = self.sla(self.l); 8 }
      0x26 => {
        let addr = self.get_hl();
        let value = self.bus.read_byte(addr);
        let result = self.sla(value);
        self.bus.write_byte(addr, result);
        16
      }

      // SRA r - Shift right arithmetic
      0x2F => { self.a = self.sra(self.a); 8 }
      0x28 => { self.b = self.sra(self.b); 8 }
      0x29 => { self.c = self.sra(self.c); 8 }
      0x2A => { self.d = self.sra(self.d); 8 }
      0x2B => { self.e = self.sra(self.e); 8 }
      0x2C => { self.h = self.sra(self.h); 8 }
      0x2D => { self.l = self.sra(self.l); 8 }
      0x2E => {
        let addr = self.get_hl();
        let value = self.bus.read_byte(addr);
        let result = self.sra(value);
        self.bus.write_byte(addr, result);
        16
      }

      // SWAP r - Swap nibbles
      0x37 => { self.a = self.swap(self.a); 8 }
      0x30 => { self.b = self.swap(self.b); 8 }
      0x31 => { self.c = self.swap(self.c); 8 }
      0x32 => { self.d = self.swap(self.d); 8 }
      0x33 => { self.e = self.swap(self.e); 8 }
      0x34 => { self.h = self.swap(self.h); 8 }
      0x35 => { self.l = self.swap(self.l); 8 }
      0x36 => {
        let addr = self.get_hl();
        let value = self.bus.read_byte(addr);
        let result = self.swap(value);
        self.bus.write_byte(addr, result);
        16
      }

      // SRL r - Shift right logical
      0x3F => { self.a = self.srl(self.a); 8 }
      0x38 => { self.b = self.srl(self.b); 8 }
      0x39 => { self.c = self.srl(self.c); 8 }
      0x3A => { self.d = self.srl(self.d); 8 }
      0x3B => { self.e = self.srl(self.e); 8 }
      0x3C => { self.h = self.srl(self.h); 8 }
      0x3D => { self.l = self.srl(self.l); 8 }
      0x3E => {
        let addr = self.get_hl();
        let value = self.bus.read_byte(addr);
        let result = self.srl(value);
        self.bus.write_byte(addr, result);
        16
      }

      // BIT b, r - Test bit
      0x40..=0x7F => {
        let bit = (opcode - 0x40) / 8;
        let reg = opcode & 0x07;
        let value = match reg {
          0 => self.b,
          1 => self.c,
          2 => self.d,
          3 => self.e,
          4 => self.h,
          5 => self.l,
          6 => self.bus.read_byte(self.get_hl()),
          7 => self.a,
          _ => unreachable!(),
        };
        self.bit(bit, value);
        if reg == 6 { 12 } else { 8 }
      }

      // RES b, r - Reset bit
      0x80..=0xBF => {
        let bit = (opcode - 0x80) / 8;
        let reg = opcode & 0x07;
        match reg {
          0 => self.b = self.res(bit, self.b),
          1 => self.c = self.res(bit, self.c),
          2 => self.d = self.res(bit, self.d),
          3 => self.e = self.res(bit, self.e),
          4 => self.h = self.res(bit, self.h),
          5 => self.l = self.res(bit, self.l),
          6 => {
            let addr = self.get_hl();
            let value = self.bus.read_byte(addr);
            let result = self.res(bit, value);
            self.bus.write_byte(addr, result);
          }
          7 => self.a = self.res(bit, self.a),
          _ => unreachable!(),
        }
        if reg == 6 { 16 } else { 8 }
      }

      // SET b, r - Set bit
      0xC0..=0xFF => {
        let bit = (opcode - 0xC0) / 8;
        let reg = opcode & 0x07;
        match reg {
          0 => self.b = self.set_bit(bit, self.b),
          1 => self.c = self.set_bit(bit, self.c),
          2 => self.d = self.set_bit(bit, self.d),
          3 => self.e = self.set_bit(bit, self.e),
          4 => self.h = self.set_bit(bit, self.h),
          5 => self.l = self.set_bit(bit, self.l),
          6 => {
            let addr = self.get_hl();
            let value = self.bus.read_byte(addr);
            let result = self.set_bit(bit, value);
            self.bus.write_byte(addr, result);
          }
          7 => self.a = self.set_bit(bit, self.a),
          _ => unreachable!(),
        }
        if reg == 6 { 16 } else { 8 }
      }
    }
  }

  // Helper functions for CB instructions
  fn rlc(&mut self, value: u8) -> u8 {
    let carry = (value & 0x80) != 0;
    let result = (value << 1) | (if carry { 1 } else { 0 });
    
    self.set_zero_flag(result == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(carry);
    
    result
  }

  fn rrc(&mut self, value: u8) -> u8 {
    let carry = (value & 0x01) != 0;
    let result = (value >> 1) | (if carry { 0x80 } else { 0 });
    
    self.set_zero_flag(result == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(carry);
    
    result
  }

  fn rl(&mut self, value: u8) -> u8 {
    let old_carry = if self.is_carry_flag_set() { 1 } else { 0 };
    let new_carry = (value & 0x80) != 0;
    let result = (value << 1) | old_carry;
    
    self.set_zero_flag(result == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(new_carry);
    
    result
  }

  fn rr(&mut self, value: u8) -> u8 {
    let old_carry = if self.is_carry_flag_set() { 0x80 } else { 0 };
    let new_carry = (value & 0x01) != 0;
    let result = (value >> 1) | old_carry;
    
    self.set_zero_flag(result == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(new_carry);
    
    result
  }

  fn sla(&mut self, value: u8) -> u8 {
    let carry = (value & 0x80) != 0;
    let result = value << 1;
    
    self.set_zero_flag(result == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(carry);
    
    result
  }

  fn sra(&mut self, value: u8) -> u8 {
    let carry = (value & 0x01) != 0;
    let result = (value >> 1) | (value & 0x80);
    
    self.set_zero_flag(result == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(carry);
    
    result
  }

  fn srl(&mut self, value: u8) -> u8 {
    let carry = (value & 0x01) != 0;
    let result = value >> 1;
    
    self.set_zero_flag(result == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(carry);
    
    result
  }

  fn swap(&mut self, value: u8) -> u8 {
    let result = (value >> 4) | (value << 4);
    
    self.set_zero_flag(result == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(false);
    self.set_carry_flag(false);
    
    result
  }

  fn bit(&mut self, bit: u8, value: u8) {
    let result = value & (1 << bit);
    self.set_zero_flag(result == 0);
    self.set_subtract_flag(false);
    self.set_half_carry_flag(true);
  }

  fn res(&self, bit: u8, value: u8) -> u8 {
    value & !(1 << bit)
  }

  fn set_bit(&self, bit: u8, value: u8) -> u8 {
    value | (1 << bit)
  }

  // Execute one instruction and return cycles elapsed
  pub fn step(&mut self) -> u32 {
    if self.halted {
      return 4;
    }

    let opcode = self.fetch_byte();
    let cycles = self.execute_instruction(opcode);
    self.cycles += cycles as u64;
    cycles
  }

  // Run for a specific number of cycles
  pub fn run_cycles(&mut self, target_cycles: u32) {
    let start_cycles = self.cycles;
    while (self.cycles - start_cycles) < target_cycles as u64 {
      self.step();
    }
  }

  // Load a ROM into memory
  pub fn load_rom(&mut self, rom: &[u8]) {
    self.bus.load_rom(rom);
  }

  // Reset CPU to initial state
  pub fn reset(&mut self) {
    self.a = 0x00;
    self.f = 0x00;
    self.b = 0x00;
    self.c = 0x00;
    self.d = 0x00;
    self.e = 0x00;
    self.h = 0x00;
    self.l = 0x00;
    self.pc = 0x0000;
    self.sp = 0x0000;
    self.cycles = 0;
    self.halted = false;
  }
}