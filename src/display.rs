pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

#[derive(Default)]
pub struct Display {
    pub buffer: [[u8; WIDTH]; HEIGHT],
    pub needs_redraw: bool,
}

impl Display {
    pub fn new() -> Self {
        Self {
            buffer: [[0; WIDTH]; HEIGHT],
            needs_redraw: false,
        }
    }

    pub fn clear(&mut self) {
        self.buffer = [[0; WIDTH]; HEIGHT];
        self.needs_redraw = true;
    }

    /// The rendering backend can call this
    pub fn take_frame(&mut self) -> [[u8; WIDTH]; HEIGHT] {
        self.needs_redraw = false;
        self.buffer
    }

    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }
}
