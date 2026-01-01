pub struct Joypad {
    action_buttons: u8,
    direction_buttons: u8,
    select_action: bool,
    select_direction: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            action_buttons: 0x0F,
            direction_buttons: 0x0F,
            select_action: false,
            select_direction: false,
        }
    }

    pub fn read(&self) -> u8 {
        let mut result = 0xC0;

        if !self.select_action {
            result |= 0x20;
        }
        if !self.select_direction {
            result |= 0x10;
        }

        if self.select_action {
            result |= self.action_buttons & 0x0F;
        } else if self.select_direction {
            result |= self.direction_buttons & 0x0F;
        } else {
            result |= 0x0F;
        }

        result
    }

    pub fn write(&mut self, value: u8) {
        self.select_action = (value & 0x20) == 0;
        self.select_direction = (value & 0x10) == 0;
    }

    pub fn press_button(&mut self, button: Button) {
        match button {
            Button::A => self.action_buttons &= !0x01,
            Button::B => self.action_buttons &= !0x02,
            Button::Select => self.action_buttons &= !0x04,
            Button::Start => self.action_buttons &= !0x08,
            Button::Right => self.direction_buttons &= !0x01,
            Button::Left => self.direction_buttons &= !0x02,
            Button::Up => self.direction_buttons &= !0x04,
            Button::Down => self.direction_buttons &= !0x08,
        }
    }

    pub fn release_button(&mut self, button: Button) {
        match button {
            Button::A => self.action_buttons |= 0x01,
            Button::B => self.action_buttons |= 0x02,
            Button::Select => self.action_buttons |= 0x04,
            Button::Start => self.action_buttons |= 0x08,
            Button::Right => self.direction_buttons |= 0x01,
            Button::Left => self.direction_buttons |= 0x02,
            Button::Up => self.direction_buttons |= 0x04,
            Button::Down => self.direction_buttons |= 0x08,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Right,
    Left,
    Up,
    Down,
}

