use crate::font::FONT_SET;
use fastrand::Rng;
use opcode_macros::opcode_handler;
use std::{
    fs::File,
    io::{Error, Read},
    path::PathBuf,
};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

const PIXEL_ON: &[u8] = &[255, 255, 255, 255];
const PIXEL_OFF: &[u8] = &[0, 0, 0, 255];

pub struct Chip8Interpreter {
    memory: [u8; 4096],
    registers: [u8; 16],
    address: u16,
    program_counter: u16,
    stack: [u16; 16],
    stack_ptr: usize,
    rng: Rng,
    vram: [bool; WIDTH * HEIGHT],
    delay_timer: u8,
    sound_timer: u8,
    should_execute: bool,
    keyboard: [bool; 16],
    total_dt: u16,
    total_dt2: u16,
}
impl Chip8Interpreter {
    pub fn new() -> Self {
        let mut mem = [0; 4096];
        mem[..FONT_SET.len()].copy_from_slice(&FONT_SET);
        Self {
            memory: mem,
            registers: [0; 16],
            address: 0,
            program_counter: 0x200,
            stack: [0; 16],
            stack_ptr: 0,
            delay_timer: 0,
            sound_timer: 0,
            vram: [false; WIDTH * HEIGHT],
            rng: Rng::new(),
            should_execute: false,
            keyboard: [false; 16],
            total_dt: 0,
            total_dt2: 0,
        }
    }
    pub fn update_key(&mut self, position: usize, state: bool) {
        self.keyboard[position] = state;
    }
    pub fn should_beep(&self) -> bool {
        self.sound_timer > 0
    }
    // Given a path to a file, load it into memory and execute it
    pub fn load_rom(&mut self, f: PathBuf) -> Result<(), Error> {
        *self = Self::new();
        let mut file = File::open(f)?;
        file.read(&mut self.memory[0x200..])?;
        self.should_execute = true;
        Ok(())
    }

    pub fn clear_display(&mut self) {
        self.vram = [false; WIDTH * HEIGHT];
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, n: usize) {
        self.registers[0xf] = 0;
        for byte in 0..n {
            let y = (self.registers[y] as usize + byte) % HEIGHT;
            for bit in 0..8 {
                let x = (self.registers[x] + bit) as usize % WIDTH;
                let color = (self.memory[self.address as usize + byte] >> (7 - bit)) & 1;
                self.registers[0x0f] |= color & self.vram[y * WIDTH + x] as u8;
                self.vram[y * WIDTH + x] ^= color != 0;
            }
        }
    }
    pub fn draw_pixels(&mut self, pixels: &mut [u8]) {
        debug_assert!(pixels.len() == HEIGHT * WIDTH * 4);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let state = match self.vram[y * WIDTH + x] {
                    true => PIXEL_ON,
                    false => PIXEL_OFF,
                };
                let index = (y * WIDTH + x) * 4;
                pixels[index..index + 4].copy_from_slice(&state);
            }
        }
    }
    pub fn update_timer(&mut self) {
        const TIMER_PERIOD: u16 = 1;
        if self.delay_timer > 0 {
            self.total_dt += TIMER_PERIOD;
            while self.total_dt > TIMER_PERIOD {
                self.total_dt -= TIMER_PERIOD;
                self.delay_timer = self.delay_timer.wrapping_sub(1);
            }
        }
        if self.sound_timer > 0 {
            self.total_dt2 += TIMER_PERIOD;
            while self.total_dt2 > TIMER_PERIOD {
                self.total_dt2 -= TIMER_PERIOD;
                self.sound_timer = self.sound_timer.wrapping_sub(1);
            }
        }
    }
    pub fn execute_cycle(&mut self) {
        if !self.should_execute {
            return;
        }
        self.update_timer();

        let opcode = {
            let location = self.program_counter as usize;
            let mem = [self.memory[location], self.memory[location + 1]];
            u16::from_be_bytes(mem)
        };
        self.program_counter += 2;

        self.handle_opcode(opcode);
    }
}

impl Chip8Interpreter {
    pub fn handle_opcode(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let byte = opcode & 0x00FF;
        let address = opcode & 0x07FF;
        let nimble = opcode & 0x000F;

        opcode_handler!(opcode
            "00e0" => {
                self.clear_display()
            },
            "00ee" => {
                self.stack_ptr -= 1;
                self.program_counter = self.stack[self.stack_ptr];
            },
            "1nnn" => {
                self.program_counter = address;
            },
            "2nnn" => {
                self.stack[self.stack_ptr] = self.program_counter;
                self.stack_ptr += 1;
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
                self.registers[x] = self.registers[x].wrapping_add(byte as u8);
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
                self.draw_sprite(x, y, nimble as usize);
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
            "Fx07" => {
                self.registers[x] = self.delay_timer;
            },
            "Fx0A" => {
                // TODO: use keydown event and make nicer
                if self.keyboard.iter().any(|x| *x) {
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
    }
}
