mod cpu;
use minifb::{Window, WindowOptions, Scale, Key};
use std::time::{Instant, Duration};


fn main() {

    let timer_rate = Duration::from_micros(16666); // ~1/60 sec = 16.666 ms
    let mut last_timer_update = Instant::now();

    let keymap = vec![
    (0x1, Key::Key1),
    (0x2, Key::Key2),
    (0x3, Key::Key3),
    (0xC, Key::Key4),
    (0x4, Key::Q),
    (0x5, Key::W),
    (0x6, Key::E),
    (0xD, Key::R),
    (0x7, Key::A),
    (0x8, Key::S),
    (0x9, Key::D),
    (0xE, Key::F),
    (0xA, Key::Z),
    (0x0, Key::X),
    (0xB, Key::C),
    (0xF, Key::V),
    ];


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
    game_cpu.load_rom("roms/CONNECT4.ch8").unwrap();
    while window.is_open() {
        game_cpu.clear_keys();

        for (chip8_key, pc_key) in keymap.iter() {
            if window.is_key_down(*pc_key) {
                game_cpu.set_key(*chip8_key);
            }
        }
        if last_timer_update.elapsed() >= timer_rate {
            game_cpu.update_timers();          // decrement delay & sound timers
            last_timer_update = Instant::now();
        }

        for _ in 0..10 {
            game_cpu.cycle();
        }
        window.update_with_buffer(&game_cpu.get_display(), 64, 32).unwrap();





    }
    
}


