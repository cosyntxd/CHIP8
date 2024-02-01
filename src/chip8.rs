use fastrand::Rng;
use std::{
    fs::File, io::{Error, Read}, ops::Index, path::PathBuf
};
use winit::event::{ElementState, Event, VirtualKeyCode as VKC, WindowEvent};
use crate::{font::FONT_SET};


pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

const PIXEL_ON: &[u8] = &[255, 255, 255, 255];
const PIXEL_OFF: &[u8] = &[0, 0, 0, 255];

const CONTROLS: [VKC; 16] = [
    VKC::Key1,  VKC::Key2,  VKC::Key3,  VKC::Key4,
    VKC::Q,     VKC::W,     VKC::E,     VKC::R,
    VKC::A,     VKC::S,     VKC::D,     VKC::F,
    VKC::Z,     VKC::X,     VKC::C,     VKC::V,
];
pub struct Chip8Interpreter {
    memory: [u8; 4096],
    registers: [u8; 16],
    address: u16,
    program_counter: u16,
    stack: Vec<u16>,
    stack_ptr: usize,
    rng: Rng,
    vram: [u8; WIDTH * HEIGHT],
    delay_timer: u8,
    sound_timer: u8,
    should_execute: bool,
    keyboard: [bool; 16],
    pub total_dt: f32,
    beep: bool,
}
impl Chip8Interpreter {
    pub fn new() -> Self {
        let mut mem = [0; 4096];
        mem[..FONT_SET.len()].copy_from_slice(&FONT_SET);
        Self {
            memory: mem,
            stack_ptr: 0,
            registers: [0; 16],
            address: 0,
            program_counter: 0x200,
            stack: vec![],
            delay_timer: 0,
            sound_timer: 0,
            vram: [0; WIDTH * HEIGHT],
            rng: Rng::new(),
            should_execute: false,
            keyboard: [true; 16],
            beep: false,
            total_dt: 0.0,
        }
    }
    // Gets keyboard events from winit's eventloop, only processing the keys listed in `CONTROLS`
    // Maps the keys to an index in `self.keyboard` and sets it to the state of the key
    pub fn process_keys(&mut self, event: &Event<()>) {
        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::KeyboardInput { input, .. } = event {
                if let Some(key) = input.virtual_keycode {
                    if let Some(position) = CONTROLS.iter().position(|k| k == &key) {
                        self.keyboard[position] = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                    }
                }
            }
        }
    }
    // Given a path to a file, load it into memory and execute it
    pub fn load_rom(&mut self, f: PathBuf) -> Result<(), Error> {
        let mut file = File::open(f)?;
        file.read(&mut self.memory[0x200..])?;
        self.should_execute = true;
        Ok(())
    }

    pub fn clear_display(&mut self, pixels: &mut [u8]) {
        for pixel in pixels.chunks_exact_mut(4) {
            pixel[0..4].copy_from_slice(&PIXEL_OFF);
        }
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, n: u16, _pixels: &mut [u8]) {
        self.registers[0xf] = 0;
        for byte in 0..n {
            let y = (self.registers[y as usize] as usize + byte as usize) % HEIGHT;
            for bit in 0..8 {
                let x = (self.registers[x as usize] as usize + bit) % WIDTH;
                let color = (self.memory[self.address as usize + byte as usize] >> (7 - bit)) & 1;
                self.registers[0x0f] |= color & self.vram[y * WIDTH + x];
                self.vram[y * WIDTH + x] ^= color;
                let state = match self.vram[y * WIDTH + x] {
                    0 => PIXEL_OFF,
                    1 => PIXEL_ON,
                    _ => unreachable!(),
                };
                _pixels[(y * WIDTH + x) * 4..(y * WIDTH + x) * 4 + 4].copy_from_slice(&state);
            }
        }
    }
    fn update_timer(&mut self, dt: f32) {
        if self.delay_timer > 0 {
            self.total_dt += dt;
            const TIMER_PERIOD: f32 = 1.0 / 60.0;
            while self.total_dt > TIMER_PERIOD {
                self.total_dt -= TIMER_PERIOD;
                self.delay_timer = self.delay_timer.wrapping_sub(1);
            }
        }
    }

    pub fn execute_cycle(&mut self, pixels: &mut [u8]) {
        if !self.should_execute {
            return;
        }
        // TODO
        self.update_timer(1.0/60.0);

        let opcode = {
            let location = self.program_counter as usize;
            let mem = [self.memory[location], self.memory[location + 1]];
            u16::from_be_bytes(mem)
        };
        self.program_counter += 2;
        
        self.handle_opcode(opcode, pixels);

    }
}

macro_rules! opcode_equals {
    ($opcode:expr, $pattern:expr) => {
        {
            // Convert pattern to hex with wildcards zeroed
            let match_operation = u16::from_str_radix(&$pattern
                .replace("x", "0")
                .replace("y", "0")
                .replace("k", "0")
                .replace("n", "0"), 16)
                .unwrap();
            // Create a bitmask for all non wildcards
            let mut match_mask = 0;
            for (i,c) in $pattern.chars().rev().enumerate() {
                if !matches!(c, 'x' | 'y' | 'k' | 'n') {
                    match_mask += 0b1111 << i * 4
                }
            }
            // Compares opcodes when wildcards have been zeroed
            $opcode & match_mask == match_operation
        }
    };
}
macro_rules! opcode_handler {
    ($opcode:expr, $($pattern:literal => $code:block),+) => {
        match true {
            $(
                true if opcode_equals!($opcode, $pattern) => $code
            )*
            _ => {panic!("Unknown opcode: {:#4x}",$opcode)}
        }
    };

}
impl Chip8Interpreter {
    fn handle_opcode(&mut self, opcode: u16, pixels: &mut [u8]) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let byte = opcode & 0x00FF;
        let address = opcode & 0x07FF;
        let nimble = opcode & 0x000F;

        opcode_handler!(opcode,
            "00e0" => {
                self.clear_display(pixels)
            },
            "00ee" => {
                if let Some(adr) = self.stack.pop() {
                    self.program_counter = adr
                }
            },
            "1nnn" => {
                self.program_counter = address;
            },
            "2nnn" => {
                self.stack.push(self.program_counter);
                self.program_counter = address;
            },
            "3xkk" => {
                if self.registers[x] as u16 == byte {
                    self.program_counter += 2;
                }
            },
            "4xkk" => {
                if self.registers[x] as u16 != byte {
                    self.program_counter += 2;
                }
            },
            "5xy0" => {
                if self.registers[x] == self.registers[y] {
                    self.program_counter += 2;
                }
            },
            "6xkk" => {
                self.registers[x] = byte as u8;
            },
            "7xkk" => {
                self.registers[x] += byte as u8;
            },
            "8xy0" => {
                self.registers[x] = self.registers[y];
            },
            "8xy1" => {
                self.registers[x] |= self.registers[y];
            },
            "8xy2" => {
                self.registers[x] &= self.registers[y];
            },
            "8xy3" => {
                self.registers[x] ^= self.registers[y];
            },
            "8xy4" => {
                let intermediate = self.registers[x] as u16 + self.registers[y] as u16;
                self.registers[0xf] = (intermediate > 0xff) as u8;
                self.registers[x] = (intermediate & 0xff) as u8;
            },
            "8xy5" => {
                self.registers[0xf] = (self.registers[x] > self.registers[y]) as u8;
                self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
            },
            "8xy6" => {
                self.registers[0xf] = self.registers[x] & 0x1;
                self.registers[x] /= 2;
            },
            "8xy7" => {
                self.registers[0xf] = (self.registers[y] > self.registers[x]) as u8;
                self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
            },
            "8xyE" => {
                self.registers[0xf] = self.registers[x] >> 7;
                self.registers[x] *= 2;
            },
            "9xy0" => {
                if self.registers[x] != self.registers[y] {
                    self.program_counter += 2;
                }
            },
            "Annn" => {
                self.address = address;
            },
            "Bnnn" => {
                self.program_counter = address + self.registers[0x0] as u16;
            },
            "Cxkk" => {
                self.registers[x] = self.rng.u8(0..255) & byte as u8;
            },
            "Dxyn" => {
                self.draw_sprite(x, y, nimble, pixels);
            },
            "Ex9E" => {
                if self.keyboard[self.registers[x] as usize] {
                    self.program_counter += 2;
                }
            },
            "ExA1" => {
                if !self.keyboard[self.registers[x] as usize] {
                    self.program_counter += 2;
                }
            },
            // No audio yet
            "Fx07" => {
                self.registers[x] = self.delay_timer;
            },
            "Fx0A" => {
                // TODO: use keydown event and make nicer
                if (self.keyboard.iter().any(|x| *x)) {
                    self.registers[x as usize] = self.keyboard.iter().position(|x| *x).unwrap() as u8;
                } else {
                    self.program_counter -= 2;
                }
            },
            "Fx15" => {
                self.delay_timer = self.registers[x] + 1;
            },
            "Fx18" => {
                self.sound_timer = self.registers[x];
            },
            "Fx1E" => {
                self.address += self.registers[x] as u16;
            },
            "Fx29" => {
                self.address = self.registers[x] as u16 * 5
            },
            "Fx33" => {
                self.memory[self.address as usize] = self.registers[x] / 100;
                self.memory[self.address as usize + 1] = (self.registers[x] / 10) % 10;
                self.memory[self.address as usize + 2] = self.registers[x] % 10;
            },
            "Fx55" => {
                let mem = &mut self.memory[address as usize..address as usize + x as usize];
                mem.copy_from_slice(&self.registers[0..x as usize]);
                self.address += (x + 1) as u16;
            },
            "Fx65" => {
                let mem = &mut self.memory[address as usize..address as usize + x as usize];
                self.registers[0..x as usize].copy_from_slice(&mem);
                self.address += (x + 1) as u16;
            }

        );
        // println!("{opcode:x} {:?}", (self.registers, self.address, self.program_counter, &self.stack, self.vram))
    }
}