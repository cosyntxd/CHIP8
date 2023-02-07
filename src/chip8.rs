use std::{
    fs::File,
    io::{Error, ErrorKind, Read},
    path::PathBuf,
};
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub struct Chip8Interpreter {
    memory: [u8; 4096],
}
impl Chip8Interpreter {
    pub fn new() -> Self {
        Self {
            memory: [0; 4096]
        }
    }
    pub fn load_rom(&mut self, f: PathBuf) -> Result<(), Error> {
        let mut file = File::open(&f)?;
        if file.metadata()?.len() > 4096 - 0x200 {
            panic!("File larger than maximum size of 3584")
        }
        file.read(&mut self.memory[0x200..])?;
        println!("Successfully loaded {}", f.display());
        Ok(())
    }
    pub fn execute_cycle(&mut self, pixels: &mut [u8]) {}
}
