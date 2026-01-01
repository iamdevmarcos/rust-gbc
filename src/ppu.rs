use crate::interrupts::{Interrupts, InterruptType};

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

const SCANLINE_CYCLES: u32 = 456;
const VBLANK_START: u8 = 144;
const VBLANK_END: u8 = 154;

#[derive(PartialEq, Copy, Clone)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamSearch = 2,
    PixelTransfer = 3,
}

pub struct PPU {
    pub framebuffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
    pub vram: [u8; 0x2000],
    pub oam: [u8; 0xA0],
    
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,
    
    mode: Mode,
    cycles: u32,
    frame_ready: bool,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            framebuffer: [0xFF; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            lcdc: 0x91,
            stat: 0x00,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            mode: Mode::OamSearch,
            cycles: 0,
            frame_ready: false,
        }
    }

    pub fn tick(&mut self, cycles: u32, interrupts: &mut Interrupts) {
        if !self.is_lcd_enabled() {
            return;
        }

        self.cycles += cycles;

        match self.mode {
            Mode::OamSearch => {
                if self.cycles >= 80 {
                    self.cycles -= 80;
                    self.mode = Mode::PixelTransfer;
                }
            }
            Mode::PixelTransfer => {
                if self.cycles >= 172 {
                    self.cycles -= 172;
                    self.mode = Mode::HBlank;
                    self.render_scanline();
                }
            }
            Mode::HBlank => {
                if self.cycles >= 204 {
                    self.cycles -= 204;
                    self.ly += 1;

                    if self.ly >= VBLANK_START {
                        self.mode = Mode::VBlank;
                        interrupts.request_interrupt(InterruptType::VBlank);
                        self.frame_ready = true;
                    } else {
                        self.mode = Mode::OamSearch;
                    }
                }
            }
            Mode::VBlank => {
                if self.cycles >= SCANLINE_CYCLES {
                    self.cycles -= SCANLINE_CYCLES;
                    self.ly += 1;

                    if self.ly >= VBLANK_END {
                        self.ly = 0;
                        self.mode = Mode::OamSearch;
                    }
                }
            }
        }

        self.update_stat();
    }

    fn render_scanline(&mut self) {
        if !self.is_bg_enabled() {
            return;
        }

        let y = self.ly;
        let scroll_y = self.scy.wrapping_add(y);
        let tile_y = (scroll_y / 8) as u16;

        for x in 0..SCREEN_WIDTH {
            let scroll_x = self.scx.wrapping_add(x as u8);
            let tile_x = (scroll_x / 8) as u16;
            
            let tile_map_addr = if self.is_bg_tile_map_high() {
                0x1C00 + tile_y * 32 + tile_x
            } else {
                0x1800 + tile_y * 32 + tile_x
            };

            let tile_index = self.vram[tile_map_addr as usize];
            
            let tile_data_addr = if self.is_tile_data_unsigned() {
                tile_index as u16 * 16
            } else {
                if tile_index < 128 {
                    0x1000 + tile_index as u16 * 16
                } else {
                    0x1000 + ((tile_index as i8) as i16 * 16) as u16
                }
            };

            let line = (scroll_y % 8) as u16;
            let byte1 = self.vram[(tile_data_addr + line * 2) as usize];
            let byte2 = self.vram[(tile_data_addr + line * 2 + 1) as usize];

            let pixel_x = 7 - (scroll_x % 8);
            let color_bit = ((byte2 >> pixel_x) & 1) << 1 | ((byte1 >> pixel_x) & 1);
            let color = self.get_bg_color(color_bit);

            let pixel_index = (y as usize * SCREEN_WIDTH + x) * 3;
            self.framebuffer[pixel_index] = color;
            self.framebuffer[pixel_index + 1] = color;
            self.framebuffer[pixel_index + 2] = color;
        }
    }

    fn get_bg_color(&self, color_num: u8) -> u8 {
        match (self.bgp >> (color_num * 2)) & 0x03 {
            0 => 0xFF,
            1 => 0xAA,
            2 => 0x55,
            3 => 0x00,
            _ => unreachable!(),
        }
    }

    fn is_lcd_enabled(&self) -> bool {
        self.lcdc & 0x80 != 0
    }

    fn is_bg_enabled(&self) -> bool {
        self.lcdc & 0x01 != 0
    }

    fn is_bg_tile_map_high(&self) -> bool {
        self.lcdc & 0x08 != 0
    }

    fn is_tile_data_unsigned(&self) -> bool {
        self.lcdc & 0x10 != 0
    }

    fn update_stat(&mut self) {
        self.stat = (self.stat & 0xFC) | (self.mode as u8);
    }

    pub fn is_frame_ready(&mut self) -> bool {
        let ready = self.frame_ready;
        self.frame_ready = false;
        ready
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = value,
            0xFF40 => self.lcdc = value,
            0xFF41 => self.stat = (self.stat & 0x07) | (value & 0xF8),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {},
            0xFF45 => self.lyc = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => {}
        }
    }
}

