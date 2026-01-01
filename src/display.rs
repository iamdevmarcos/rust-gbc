use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use crate::ppu::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::gameboy::GameBoy;

pub fn run_with_display(mut gb: GameBoy) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    
    let window = video_subsystem
        .window("Game Boy Color Emulator", (SCREEN_WIDTH * 4) as u32, (SCREEN_HEIGHT * 4) as u32)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_scale(4.0, 4.0).map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut frame_count = 0;
    let mut last_pc = gb.cpu.pc;
    let mut stuck_count = 0;
    
    println!("\nEmulator running! Press ESC to quit.\n");
    
    println!("Debug - LCDC: 0x{:02X}", gb.cpu.bus.ppu.read(0xFF40));
    println!("Debug - LY: 0x{:02X}", gb.cpu.bus.ppu.read(0xFF44));
    println!("Debug - PC: 0x{:04X}", gb.cpu.pc);
    
    for i in 0..10 {
        let addr = 0x8000 + i * 100;
        println!("VRAM[0x{:04X}]: {:02X}", addr, gb.cpu.bus.ppu.read(addr));
    }

    'running: loop {
        gb.run_frame();
        
        if gb.cpu.pc == last_pc {
            stuck_count += 1;
        } else {
            stuck_count = 0;
            last_pc = gb.cpu.pc;
        }
        
        if frame_count == 0 {
            println!("\nAfter first frame:");
            println!("LCDC: 0x{:02X}", gb.cpu.bus.ppu.read(0xFF40));
            println!("LY: 0x{:02X}", gb.cpu.bus.ppu.read(0xFF44));
            println!("SCX: 0x{:02X}, SCY: 0x{:02X}", 
                     gb.cpu.bus.ppu.read(0xFF43),
                     gb.cpu.bus.ppu.read(0xFF42));
            
            let mut non_white = 0;
            for i in 0..gb.cpu.bus.ppu.framebuffer.len() {
                if gb.cpu.bus.ppu.framebuffer[i] != 0xFF {
                    non_white += 1;
                }
            }
            println!("Non-white pixels: {}/{}", non_white, gb.cpu.bus.ppu.framebuffer.len());
        }
        
        texture.update(None, &gb.cpu.bus.ppu.framebuffer, SCREEN_WIDTH * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
        
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        frame_count += 1;
        if frame_count % 60 == 0 {
            println!("Frames: {}, Cycles: {}, PC: 0x{:04X}", 
                     frame_count, gb.cpu.cycles, gb.cpu.pc);
        }
    }

    println!("\nEmulator stopped after {} frames", frame_count);
    println!("Total cycles: {}", gb.cpu.cycles);
    
    Ok(())
}


