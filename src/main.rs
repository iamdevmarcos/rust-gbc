mod cpu;

fn main() {
    let mut cpu = cpu::CPU::new();
    cpu.show_cpu_state();

    cpu.add(10);
    cpu.show_cpu_state();
}
