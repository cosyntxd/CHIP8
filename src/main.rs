use std::{env, path::{Path, PathBuf}};

use chip8::{chip8::Chip8Interpreter, window::{Chip8Window, GameEvents, MINUS_POSITION, PLUS_POSITION}};

fn main() {
    let window = Chip8Window::new();
    let mut interpreter = Chip8Interpreter::new();

    // CLI args of: chip8 <PATH> -d <LEVEL>
    let args: Vec<String> = env::args().collect();
    // Process debug first to immediately begin logging
    if let Some(debug) = args.iter().position(|arg| arg == "-d") {
        let level = args.get(debug+1)
            .and_then(|val| val.parse().ok())
            .unwrap_or(0);
        interpreter.set_debug(level)
    }
    // Now when loading rom from args, there will be logs 
    if let Some(file) = args.get(1) {
        if Path::new(file).is_file() {
            if let Err(e) = interpreter.load_rom(PathBuf::from(file)) {
                println!("Could not load ROM: {e}");
            }
        }
    }

    
    let mut iteration_speed = 1;
    window.run(move |event, pixels| {
        match event {
            GameEvents::DroppedFile(path_buf) => {
                if let Err(e) = interpreter.load_rom(path_buf) {
                    println!("Could not load ROM: {e}");
                }
            },
            GameEvents::KeyInput(position, state) => {
                if position < 16 {
                    interpreter.update_key(position, state);
                } else {
                    if (position == MINUS_POSITION) && iteration_speed > 0 {
                        iteration_speed -= 1;
                    }

                    if (position == PLUS_POSITION) && iteration_speed < 250 {
                        iteration_speed += 1;
                    }
                    assert!(iteration_speed > 0 && iteration_speed < 250);
                }
                
            },
            GameEvents::Redraw => {
                for _ in 0..iteration_speed {
                    interpreter.execute_cycle();
                }
                interpreter.draw_pixels(pixels);
                return interpreter.should_beep();
            },
        }
        false
    });
}
