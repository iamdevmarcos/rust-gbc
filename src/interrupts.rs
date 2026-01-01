const VBLANK: u8 = 0b00000001;
const LCD_STAT: u8 = 0b00000010;
const TIMER: u8 = 0b00000100;
const SERIAL: u8 = 0b00001000;
const JOYPAD: u8 = 0b00010000;

pub struct Interrupts {
    pub interrupt_enable: u8,
    pub interrupt_flag: u8,
    pub ime: bool,
}

impl Interrupts {
    pub fn new() -> Self {
        Interrupts {
            interrupt_enable: 0,
            interrupt_flag: 0,
            ime: false,
        }
    }

    pub fn request_interrupt(&mut self, interrupt: InterruptType) {
        self.interrupt_flag |= interrupt as u8;
    }

    pub fn has_pending_interrupt(&self) -> bool {
        self.ime && (self.interrupt_enable & self.interrupt_flag) != 0
    }

    pub fn get_next_interrupt(&mut self) -> Option<u16> {
        if !self.ime {
            return None;
        }

        let pending = self.interrupt_enable & self.interrupt_flag;
        
        if pending & VBLANK != 0 {
            self.interrupt_flag &= !VBLANK;
            Some(0x0040)
        } else if pending & LCD_STAT != 0 {
            self.interrupt_flag &= !LCD_STAT;
            Some(0x0048)
        } else if pending & TIMER != 0 {
            self.interrupt_flag &= !TIMER;
            Some(0x0050)
        } else if pending & SERIAL != 0 {
            self.interrupt_flag &= !SERIAL;
            Some(0x0058)
        } else if pending & JOYPAD != 0 {
            self.interrupt_flag &= !JOYPAD;
            Some(0x0060)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub enum InterruptType {
    VBlank = 0b00000001,
    LcdStat = 0b00000010,
    Timer = 0b00000100,
    Serial = 0b00001000,
    Joypad = 0b00010000,
}

