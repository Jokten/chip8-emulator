use std::fs::File;
use std::io::Read;


pub const MEMORY_SIZE: usize = 4096;
pub const NUM_REGS: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const FONT_START: usize = 0x50;

pub struct Cpu {
    memory: [u8; MEMORY_SIZE],
    v: [u8; NUM_REGS],
    i: u16,
    pc: u16,
    sp: u8,
    stack: [u16; STACK_SIZE],
    delay_timer: u8,
    sound_timer: u8,
    display: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    keypad: [bool; 16],   
    //TODO: Add flags for ambiguous instructions
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Cpu{
            memory : [0; MEMORY_SIZE],
            v: [0; NUM_REGS],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            display: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            keypad: [false; 16],
        };
        cpu.load_default_font();
        cpu

    }

    pub fn cycle(&mut self) {
        let opcode = self.fetch();
        self.decode_and_execute(opcode);
    }

    pub fn load_rom(&mut self, path: &str) -> std::io::Result<()> {
        let mut file = File::open(path).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("Failed to open ROM '{}': {}", path, e)
            )
        })?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("Failed to read ROM '{}': {}", path, e)
            )
        })?;

        for (i, byte) in buffer.iter().enumerate() {
            if 0x200 + i >= MEMORY_SIZE {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "ROM '{}' is too large ({} bytes). Max allowed is {} bytes.",
                        path,
                        buffer.len(),
                        MEMORY_SIZE - 0x200
                    ),
                ));
            }
            self.memory[0x200 + i] = *byte;
        }

        Ok(())
    }


    pub fn clear_keys(&mut self) {
        self.keypad = [false; 16];
    }

    pub fn set_key(&mut self, key: isize) {
        self.keypad[key as usize] = true;
        println!("Key pressed {key}");
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            // optionally trigger a beep here
    }
}
    
    fn load_default_font(&mut self) {
        let font =  [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
            ];
        for (i, byte) in font.iter().enumerate() {
            self.memory[FONT_START+i] = *byte;
        }

    }
    pub fn get_display(&self) -> Vec<u32> {
        self.display
            .iter()
            .flatten()
            .map(|&pixel| if pixel != 0 { 0xFFFFFFFF } else { 0 })
            .collect()
    }

    fn fetch(&mut self) -> u16{
        let first_half = self.memory[self.pc as usize];
        let second_half = self.memory[(self.pc + 1) as usize];
        self.pc += 2;
        let opcode: u16 = ((first_half as u16) << 8) | second_half as u16;
        opcode
    }

    fn decode_and_execute(&mut self, opcode: u16) {
        let nibble = (
        (opcode & 0xF000) >> 12,
        (opcode & 0x0F00) >> 8,
        (opcode & 0x00F0) >> 4,
        (opcode & 0x000F) >> 0,
        );

        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n  =  (opcode & 0x000F) as u8;
        let nn =  (opcode & 0x00FF) as u8;
        let nnn =  opcode & 0x0FFF;

        match nibble {
            (0x0, 0x0, 0xE, 0x0) => self.op_clear_screen(),
            (0x0, 0x0, 0xE, 0xE) => self.op_return(),
            (0x0,   _,   _,   _) => {},
            (0x1,   _,   _,   _) => self.op_jmp(nnn),
            (0x2,   _,   _,   _) => self.op_call(nnn),
            (0x3,   _,   _,   _) => self.op_skip_eq_byte(x, nn),
            (0x4,   _,   _,   _) => self.op_skip_neq_byte(x, nn),
            (0x5,   _,   _, 0x0) => self.op_skip_eq_reg(x, y),
            (0x6,   _,   _,   _) => self.op_load_byte(x, nn),
            (0x7,   _,   _,   _) => self.op_add_byte(x, nn),
            (0x8,   _,   _, 0x0) => self.op_load_reg(x, y),
            (0x8,   _,   _, 0x1) => self.op_or(x, y),
            (0x8,   _,   _, 0x2) => self.op_and(x, y),
            (0x8,   _,   _, 0x3) => self.op_xor(x, y),
            (0x8,   _,   _, 0x4) => self.op_add_reg(x, y),
            (0x8,   _,   _, 0x5) => self.op_sub_xy(x, y),
            (0x8,   _,   _, 0x6) => self.op_shift_right(x, y),
            (0x8,   _,   _, 0x7) => self.op_sub_yx(x, y),
            (0x8,   _,   _, 0xE) => self.op_shift_left(x, y),
            (0x9,   _,   _, 0x0) => self.op_skip_neq_reg(x, y),
            (0xA,   _,   _,   _) => self.op_load_i(nnn),
            (0xB,   _,   _,   _) => self.op_jmp_off(nnn),
            (0xC,   _,   _,   _) => self.op_rand(x, nn),
            (0xD,   _,   _,   _) => self.op_disp(x, y, n),
            (0xE,   _, 0x9, 0xE) => self.op_skip_key_press(x),
            (0xE,   _, 0xA, 0x1) => self.op_skip_key_npress(x),
            (0xF,   _, 0x0, 0x7) => self.op_load_del(x),
            (0xF,   _, 0x1, 0x5) => self.op_set_del(x),
            (0xF,   _, 0x1, 0x8) => self.op_set_snd(x),
            (0xF,   _, 0x1, 0xE) => self.op_add_i_reg(x),
            (0xF,   _, 0x0, 0xA) => self.op_wait_key(x),
            (0xF,   _, 0x2, 0x9) => self.op_font(x),
            (0xF,   _, 0x3, 0x3) => self.op_conv(x),
            (0xF,   _, 0x5, 0x5) => self.op_store_regs(x),
            (0xF,   _, 0x6, 0x5) => self.op_load_regs(x),
            _ => println!("Invalid instruction!")
        }

    }

    fn op_clear_screen(&mut self) {
        self.display = [[0;DISPLAY_WIDTH]; DISPLAY_HEIGHT];
    }

    fn op_return(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn op_jmp(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn op_call(&mut self, addr: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.pc = addr;
        self.sp += 1;
    }

    fn op_skip_eq_byte(&mut self, x: usize, byte: u8) {
        if byte == self.v[x] {
            self.pc += 2;
        }
    }

    fn op_skip_neq_byte(&mut self, x: usize, byte: u8) {
        if byte != self.v[x] {
            self.pc += 2;
        }
    }

    fn op_skip_eq_reg(&mut self, x: usize, y: usize) {
        if self.v[y] == self.v[x] {
            self.pc += 2;
        }
    }

    fn op_load_byte(&mut self, x: usize, byte: u8) {
        self.v[x] = byte;
    }

    fn op_add_byte(&mut self, x: usize, byte: u8) {
        self.v[x] = self.v[x].wrapping_add(byte);
    }

    fn op_load_reg(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }

    fn op_or(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
    }

    fn op_and(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
    }

    fn op_xor(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
    }

    fn op_add_reg(&mut self, x: usize, y: usize) {
        let (sum, overflow) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = sum;
        self.v[0xF] = if overflow {1} else {0};
    }

    fn op_sub_xy(&mut self, x: usize, y: usize) {
        let (sum, overflow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = sum;
        self.v[0xF] = if overflow {0} else {1};
    }

    fn op_shift_right(&mut self, x: usize, y: usize) {
        // TODO: Amb flag imp
        self.v[0xF] = self.v[y] & 0x1;
        self.v[x] = self.v[y] >> 1;
    }

    fn op_sub_yx(&mut self, x: usize, y: usize) {
        let (sum, overflow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = sum;
        self.v[0xF] = if overflow {0} else {1};
    }

    fn op_shift_left(&mut self, x: usize, y: usize) {
        self.v[0xF] = (self.v[y] & 0x80) >> 7;
        self.v[x] = self.v[y] << 1;
    }

    fn op_skip_neq_reg(&mut self, x: usize, y: usize) {
        if self.v[y] != self.v[x] {
            self.pc += 2;
        }
    }

    fn op_load_i(&mut self, val: u16) {
        self.i = val;
    }

    fn op_jmp_off(&mut self, addr: u16) {
        self.pc = addr + self.v[0] as u16;
    }

    fn op_rand(&mut self, x: usize, byte: u8) {
        let value: u8 = rand::random();// random byte, 0–255
        self.v[x] = value & byte;
    }

    fn op_disp(&mut self, x: usize, y: usize, n: u8) {
        let x_val = self.v[x] % (DISPLAY_WIDTH as u8);
        let y_val = self.v[y] % (DISPLAY_HEIGHT  as u8);
        self.v[0xf] = 0;
        for row in 0..n {
            let sprite = self.memory[(self.i + row as u16) as usize];
            for bit in 0..8 {
                let px = (x_val + bit) % (DISPLAY_WIDTH as u8);
                let py = (y_val + row) % (DISPLAY_HEIGHT  as u8);

                let sprite_pixel = (sprite >> (7 - bit)) & 1;
                if sprite_pixel == 0 {
                    continue;
                }
                if self.display[py as usize][px as usize] == 1 {
                    self.v[0xf] = 1;
                }
                self.display[py as usize][px as usize] ^= 1;
            }
        }

    }

    fn op_skip_key_press(&mut self, x: usize) {
        if self.keypad[self.v[x] as usize] == true {
            self.pc += 2;
        }
    }

    fn op_skip_key_npress(&mut self, x: usize) {
        if self.keypad[self.v[x] as usize] == false {
            self.pc += 2;
        }
    }

    fn op_load_del(&mut self, x: usize) {
        self.v[x] = self.delay_timer;
    }

    fn op_set_del(&mut self, x: usize) {
        self.delay_timer = self.v[x];
    }

    fn op_set_snd(&mut self, x: usize) {
        self.sound_timer = self.v[x];
    }

    fn op_add_i_reg(&mut self, x: usize) {
        self.i += self.v[x] as u16;
    }

    fn op_wait_key(&mut self, x: usize) {
        for key in 0..16 {
        if self.keypad[key] {
            self.v[x] = key as u8;
            return;
            }
        }

        // No key pressed → repeat this instruction next cycle
        self.pc -= 2;
    }

    fn op_font(&mut self, x: usize) {
        self.i = FONT_START as u16 + self.v[x] as u16;
    }

    fn op_conv(&mut self, x: usize) {
        let val = self.v[x];
        self.memory[self.i as usize] = (val/100) as u8;
        self.memory[(self.i + 1) as usize] = ((val%100)/10) as u8;
        self.memory[(self.i + 2) as usize] = (val%10) as u8;
    }

    fn op_store_regs(&mut self, x: usize) {
        for e in 0..(x+1){
            self.memory[self.i as usize+e] = self.v[e];
        }
    }

    fn op_load_regs(&mut self, x: usize) {
        for e in 0..(x+1){
            self.v[e] = self.memory[self.i as usize+e];
        }
    }
}