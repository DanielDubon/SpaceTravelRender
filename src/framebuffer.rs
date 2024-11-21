// framebuffer.rs

pub struct Framebuffer {
    pub buffer: Vec<u32>,
    pub z_buffer: Vec<f32>,
    pub width: usize,
    pub height: usize,
    current_color: u32,
    background_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            buffer: vec![0; width * height],
            z_buffer: vec![f32::INFINITY; width * height],
            width,
            height,
            current_color: 0,
            background_color: 0,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.fill(self.background_color);
        self.z_buffer.fill(f32::INFINITY);
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn should_draw(&self, x: usize, y: usize, depth: f32) -> bool {
        let index = y * self.width + x;
        depth < self.z_buffer[index]
    }

    pub fn point(&mut self, x: usize, y: usize, depth: f32) {
        let index = y * self.width + x;
        if depth < self.z_buffer[index] {
            self.buffer[index] = self.current_color;
            self.z_buffer[index] = depth;
        }
    }
}
