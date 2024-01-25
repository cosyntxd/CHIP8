use fastrand::Rng;
use std::{
    fs::File,
    io::{Error, Read},
    path::PathBuf,
};
use winit::event::{ElementState, Event, VirtualKeyCode as VKC, WindowEvent};

use crate::font::FONT_SET;

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

const PIXEL_ON: &[u8] = &[255, 255, 255, 255];
const PIXEL_OFF: &[u8] = &[0, 0, 0, 255];

const CONTROLS: [VKC; 16] = [
    VKC::Key1,
    VKC::Key2,
    VKC::Key3,
    VKC::Key4,
    VKC::Q,
    VKC::W,
    VKC::E,
    VKC::R,
    VKC::A,
    VKC::S,
    VKC::D,
    VKC::F,
    VKC::Z,
    VKC::X,
    VKC::C,
    VKC::V,
];

pub struct Chip8Interpreter {
    memory: [u8; 4096],
    registers: [u8; 16],
    address: u16,
    program_counter: u16,
    stack: [u16; 16],
    stack_ptr: usize,
    rng: Rng,
    vram: [u8; WIDTH * HEIGHT],
    delay_timer: u8,
    sound_timer: u8,
    should_execute: bool,
    keyboard: [bool; 16],
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
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            vram: [0; WIDTH * HEIGHT],
            rng: Rng::new(),
            should_execute: false,
            keyboard: [true; 16],
            beep: false,
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
    pub fn error(&self, opcode: u16) {
        panic!("Unsupported opcode: {opcode:#x}");
    }
    pub fn execute_cycle(&mut self, pixels: &mut [u8]) {
        if !self.should_execute {
            return;
        }
        self.delay_timer = self.delay_timer.wrapping_sub(1);
        // Fetch
        let opcode = {
            let location = self.program_counter as usize;
            let mem = [self.memory[location], self.memory[location + 1]];
            u16::from_be_bytes(mem)
        };

        // Decode
        let address = opcode & 0x07FF;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let byte = opcode & 0x00FF;
        let nimble = opcode & 0x000F;

        // Execute
        match opcode >> 12 {
            0x0 => {
                match nimble {
                    // Clear the display.
                    0x0 => self.clear_display(pixels),
                    // Return from a subroutine.
                    0xE => {
                        self.program_counter = self.stack[self.stack_ptr];
                        self.stack_ptr -= 1;
                    }
                    _ => self.error(opcode),
                }
            }
            // Jump to location
            0x1 => {
                self.program_counter = address - 2;
            }
            // Call subroutine
            0x2 => {
                self.stack_ptr += 1;
                self.stack[self.stack_ptr] = self.program_counter;
            }
            // Skip instruction on condition
            0x3 => {
                if self.registers[x] as u16 == byte {
                    self.program_counter += 2;
                }
            }
            // Skip instruction on condition if not equal
            0x4 => {
                if self.registers[x] as u16 != byte {
                    self.program_counter += 2;
                }
            }
            // Skip instruction on condition if equal
            0x5 => {
                if self.registers[x] == self.registers[y] {
                    self.program_counter += 2;
                }
            }
            // Set register
            0x6 => {
                self.registers[x] = byte as u8;
            }
            // Increment register
            0x7 => {
                self.registers[x] += byte as u8;
            }
            // Math operations
            0x8 => {
                match nimble {
                    // Copies register
                    0x0 => {
                        self.registers[x] = self.registers[y];
                    }
                    // Bitwise or
                    0x1 => {
                        self.registers[x] |= self.registers[y];
                    }
                    // Bitwise and
                    0x2 => {
                        self.registers[x] &= self.registers[y];
                    }
                    // Bitwise xor
                    0x3 => {
                        self.registers[x] ^= self.registers[y];
                    }
                    // Add overflow
                    0x4 => {
                        let intermediate = self.registers[x] as u16 + self.registers[y] as u16;
                        self.registers[0xf] = (intermediate > 0xff) as u8;
                        self.registers[x] = (intermediate & 0xff) as u8;
                    }
                    // Subtract
                    0x5 => {
                        self.registers[0xf] = (self.registers[x] > self.registers[y]) as u8;
                        self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
                    }
                    // Divide
                    0x6 => {
                        self.registers[0xf] = self.registers[x] & 0x1;
                        self.registers[x] /= 2;
                    }
                    // Subtract
                    0x7 => {
                        self.registers[0xf] = (self.registers[y] > self.registers[x]) as u8;
                        self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
                    }
                    // Multiply
                    0xe => {
                        self.registers[0xf] = self.registers[x] >> 7;
                        self.registers[x] *= 2;
                    }
                    _ => self.error(opcode),
                }
            }
            // Skip instruction if not equal
            0x9 => {
                if self.registers[x] != self.registers[y] {
                    self.program_counter += 2;
                }
            }
            // Set address
            0xa => {
                self.address = address;
            }
            // Jump to address
            0xb => {
                self.program_counter = address + self.registers[0x0] as u16 - 2;
            }
            // Random byte
            0xc => {
                self.registers[x] = self.rng.u8(0..255) & byte as u8;
            }
            // Draw sprite
            0xd => {
                self.draw_sprite(x, y, nimble, pixels);
            }
            // Keyboard input
            0xe => match byte {
                0x9e => {
                    if (self.keyboard[self.registers[x] as usize]) {
                        self.program_counter += 2;
                    }
                }
                0xa1 => {
                    if (!self.keyboard[self.registers[x] as usize]) {
                        self.program_counter += 2;
                    }
                }
                _ => self.error(opcode),
            },
            0xf => {
                match byte {
                    // TODO: Sound
                    0xf7 => {}
                    0x0a => {}
                    0x15 => {}
                    0x18 => {}
                    // Increment address
                    0x1e => {
                        self.address += self.registers[x] as u16;
                    }
                    // Load sprite address
                    0x29 => self.address = self.registers[x] as u16 * 5,
                    // Copy registers into memory
                    0x55 => {
                        let mem = &mut self.memory[address as usize..address as usize + x as usize];
                        mem.copy_from_slice(&self.registers[0..x as usize]);
                    }
                    // Copy registers from  memory
                    0x65 => {
                        let mem = &mut self.memory[address as usize..address as usize + x as usize];
                        self.registers[0..x as usize].copy_from_slice(&mem);
                    }

                    _ => self.error(opcode),
                }
            }

            _ => self.error(opcode),
        }
        // Go to next instruction
        self.program_counter += 2;
    }
}
