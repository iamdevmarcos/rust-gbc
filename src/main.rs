mod cpu;
mod memory;
fn main() {
    let mut cpu = cpu::CPU::new();
    cpu.show_cpu_state();

    cpu.add_to_a(10);
    cpu.show_cpu_state();
}
