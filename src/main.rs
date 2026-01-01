mod cpu;
mod memory;
mod mbc;
mod bus;
mod interrupts;
mod timer;
mod joypad;
mod ppu;
mod gameboy;
mod display;
mod bootrom;

use std::env;
use std::fs;
use std::path::Path;
use gameboy::GameBoy;
use display::run_with_display;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Game Boy Color Emulator");
        println!("\nUsage: {} <rom_file>", args[0]);
        println!("\nRunning built-in tests...\n");
        run_tests();
        return;
    }

    let rom_path = &args[1];
    
    if !Path::new(rom_path).exists() {
        eprintln!("Error: ROM file '{}' not found", rom_path);
        std::process::exit(1);
    }

    match fs::read(rom_path) {
        Ok(rom_data) => {
            println!("Loading ROM: {}", rom_path);
            println!("ROM size: {} bytes ({} KB)", rom_data.len(), rom_data.len() / 1024);
            run_rom(rom_data);
        }
        Err(e) => {
            eprintln!("Error reading ROM file: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_rom(rom_data: Vec<u8>) {
    println!("\n=== ROM Info ===");
    if rom_data.len() > 0x150 {
        let title_bytes = &rom_data[0x134..0x144];
        let title = String::from_utf8_lossy(title_bytes)
            .trim_end_matches('\0')
            .to_string();
        println!("Title: {}", title);
        println!("Cartridge type: 0x{:02X}", rom_data[0x147]);
        
        let cart_type_name = match rom_data[0x147] {
            0x00 => "ROM ONLY",
            0x01 => "MBC1",
            0x02 => "MBC1+RAM",
            0x03 => "MBC1+RAM+BATTERY",
            0x0F => "MBC3+TIMER+BATTERY",
            0x10 => "MBC3+TIMER+RAM+BATTERY",
            0x11 => "MBC3",
            0x12 => "MBC3+RAM",
            0x13 => "MBC3+RAM+BATTERY",
            _ => "Unknown",
        };
        println!("Type: {}", cart_type_name);
        
        let rom_size = 32 * (1 << rom_data[0x148]);
        println!("ROM size: {} KB", rom_size);
        println!("RAM size: 0x{:02X}", rom_data[0x149]);
    }
    
    let mut gb = GameBoy::new();
    gb.load_rom(&rom_data);
    gb.reset();

    match run_with_display(gb) {
        Ok(_) => println!("\nEmulation completed successfully!"),
        Err(e) => eprintln!("\nError: {}", e),
    }
}

fn run_tests() {
    println!("=== Game Boy Color Emulator Tests ===\n");
    
    println!("Test 1: Basic Arithmetic");
    test_arithmetic();
    
    println!("\nTest 2: 16-bit Operations");
    test_16bit_ops();
    
    println!("\nTest 3: Stack Operations");
    test_stack();
    
    println!("\nTest 4: Jumps and Calls");
    test_jumps();
    
    println!("\nTest 5: Bit Operations");
    test_bit_ops();
    
    println!("\nTest 6: Simple Program");
    test_simple_program();
    
    println!("\n=== All tests passed! ===");
}


fn test_arithmetic() {
    let mut cpu = cpu::CPU::new();
    
    let mut program = vec![0; 0x100];
    program.extend_from_slice(&[
        0x3E, 0x0A,
        0xC6, 0x05,
        0xD6, 0x03,
    ]);
    
    cpu.bus.load_rom(&program);
    
    cpu.step();
    assert_eq!(cpu.a, 10, "A should be 10");
    
    cpu.step();
    assert_eq!(cpu.a, 15, "A should be 15");
    
    cpu.step();
    assert_eq!(cpu.a, 12, "A should be 12");
    
    println!("  [OK] Arithmetic operations work correctly");
    println!("    Final A = {}", cpu.a);
}

fn test_16bit_ops() {
    let mut cpu = cpu::CPU::new();
    
    let mut program = vec![0; 0x100];
    program.extend_from_slice(&[
        0x01, 0x34, 0x12,
        0x03,
    ]);
    
    cpu.bus.load_rom(&program);
    
    cpu.step();
    assert_eq!(cpu.get_bc(), 0x1234, "BC should be 0x1234");
    
    cpu.step();
    assert_eq!(cpu.get_bc(), 0x1235, "BC should be 0x1235");
    
    println!("  [OK] 16-bit operations work correctly");
    println!("    Final BC = 0x{:04X}", cpu.get_bc());
}

fn test_stack() {
    let mut cpu = cpu::CPU::new();
    
    let mut program = vec![0; 0x100];
    program.extend_from_slice(&[
        0x01, 0xCD, 0xAB,
        0xC5,
        0x01, 0x00, 0x00,
        0xC1,
    ]);
    
    cpu.bus.load_rom(&program);
    
    cpu.step();
    cpu.step();
    let sp_after_push = cpu.sp;
    
    cpu.step();
    assert_eq!(cpu.get_bc(), 0x0000, "BC should be 0");
    
    cpu.step();
    assert_eq!(cpu.get_bc(), 0xABCD, "BC should be 0xABCD again");
    assert_eq!(cpu.sp, sp_after_push + 2, "SP should be restored");
    
    println!("  [OK] Stack operations work correctly");
    println!("    Pushed and popped BC = 0x{:04X}", cpu.get_bc());
}

fn test_jumps() {
    let mut cpu = cpu::CPU::new();
    
    let mut program = vec![0; 0x100];
    program.extend_from_slice(&[
        0xAF,
        0x28, 0x02,
        0x3E, 0x63,
        0x3E, 0x2A,
    ]);
    
    cpu.bus.load_rom(&program);
    
    cpu.step();
    assert_eq!(cpu.a, 0, "A should be 0");
    assert!(cpu.is_zero_flag_set(), "Z flag should be set");
    
    cpu.step();
    assert_eq!(cpu.pc, 0x0105, "PC should jump to 0x0105");
    
    cpu.step();
    assert_eq!(cpu.a, 42, "A should be 42");
    
    println!("  [OK] Jump operations work correctly");
    println!("    Jump succeeded, A = {}", cpu.a);
}

fn test_bit_ops() {
    let mut cpu = cpu::CPU::new();
    
    let mut program = vec![0; 0x100];
    program.extend_from_slice(&[
        0x3E, 0xAA,
        0xCB, 0x7F,
        0xCB, 0xBF,
    ]);
    
    cpu.bus.load_rom(&program);
    
    cpu.step();
    assert_eq!(cpu.a, 0xAA, "A should be 0xAA");
    
    cpu.step();
    assert!(!cpu.is_zero_flag_set(), "Bit 7 should be set, Z should be 0");
    
    cpu.step();
    assert_eq!(cpu.a, 0x2A, "A should be 0x2A (bit 7 cleared)");
    
    println!("  [OK] Bit operations work correctly");
    println!("    After RES 7, A = 0x{:02X}", cpu.a);
}

fn test_simple_program() {
    let mut cpu = cpu::CPU::new();
    
    let mut program = vec![0; 0x100];
    program.extend_from_slice(&[
        0x3E, 0x00,
        0x06, 0x05,
        0x80,
        0x05,
        0x20, 0xFC,
        0x76,
    ]);
    
    cpu.bus.load_rom(&program);
    
    let max_steps = 100;
    let mut steps = 0;
    while !cpu.halted && steps < max_steps {
        cpu.step();
        steps += 1;
    }
    
    assert_eq!(cpu.a, 15, "A should be 15 (sum of 1 to 5)");
    assert!(cpu.halted, "CPU should be halted");
    
    println!("  [OK] Simple program executed correctly");
    println!("    Sum of 1 to 5 = {}", cpu.a);
    println!("    Executed in {} steps, {} cycles", steps, cpu.cycles);
}




