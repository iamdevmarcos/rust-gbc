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
    }
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

  pub fn set_subtract_flag(&mut self, value: bool) {
    if value {
      self.f |= SUBTRACT_FLAG;
    } else {
      self.f &= !SUBTRACT_FLAG;
    }
  }

  pub fn add(&mut self, value: u8) {
    let (result, carry) = self.a.overflowing_add(value);
    self.a = result;

    self.set_zero_flag(self.a == 0);
    self.set_carry_flag(carry);
    self.set_half_carry_flag((self.a & 0x0F) + (value & 0x0F) > 0x0F);
    self.set_subtract_flag(false);
  }

  pub fn show_cpu_state(&self) {
    println!("A: {:02X}, B: {:02X}, C: {:02X}", self.a, self.b, self.c);
    println!("D: {:02X}, E: {:02X}, H: {:02X}, L: {:02X}, F: {:02X}", self.d, self.e, self.h, self.l, self.f);
    println!("PC: {:04X}, SP: {:04X}", self.pc, self.sp);
  }
}