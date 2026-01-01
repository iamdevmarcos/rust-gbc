use crate::cpu::CPU;

pub struct GameBoy {
    pub cpu: CPU,
}

impl GameBoy {
    pub fn new() -> Self {
        GameBoy {
            cpu: CPU::new(),
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.cpu.bus.load_rom(rom);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn step(&mut self) -> u32 {
        let cycles = if self.cpu.bus.interrupts.has_pending_interrupt() {
            if let Some(addr) = self.cpu.bus.interrupts.get_next_interrupt() {
                self.cpu.halted = false;
                self.cpu.bus.interrupts.ime = false;
                self.cpu.push(self.cpu.pc);
                self.cpu.pc = addr;
                20
            } else {
                self.cpu.step()
            }
        } else {
            self.cpu.step()
        };

        self.cpu.bus.timer.tick(cycles, &mut self.cpu.bus.interrupts);
        self.cpu.bus.ppu.tick(cycles, &mut self.cpu.bus.interrupts);

        cycles
    }

    pub fn run_frame(&mut self) {
        while !self.cpu.bus.ppu.is_frame_ready() {
            self.step();
        }
    }
}


