mod cpu;
use minifb::{Window, WindowOptions, Scale};

fn main() {

    let mut window = Window::new(
        "CHIP-8",
        64,
        32,
        WindowOptions {
            borderless: true,
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    ).unwrap();

    window.set_target_fps(60);
     
    let mut game_cpu = cpu::Cpu::new();
    game_cpu.load_rom("roms/IBMLogo.ch8");
    while window.is_open() {
        game_cpu.cycle();
        window.update_with_buffer(&game_cpu.get_display(), 64, 32).unwrap();
    }
    
    
}
