use std::{env, path::{Path, PathBuf}};

use chip8::{chip8::Chip8Interpreter, window::Chip8Window};
use winit::event::{ElementState, Event, VirtualKeyCode as VKC, WindowEvent};

// Hexademical keyboard remapped to qwerty
const CONTROLS: [VKC; 16] = [
    VKC::Key1, VKC::Key2, VKC::Key3, VKC::Key4,
    VKC::Q,    VKC::W,    VKC::E,    VKC::R,
    VKC::A,    VKC::S,    VKC::D,    VKC::F,
    VKC::Z,    VKC::X,    VKC::C,    VKC::V,
];

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

    let mut iterations = 1;
    window.run(move |event, pixels| {
        // Drag and drop file
        if let Event::WindowEvent { event: WindowEvent::DroppedFile(path), .. } = event {
            if let Err(e) = interpreter.load_rom(path.to_path_buf()) {
                println!("Could not load ROM: {e}");
            }
        }
        // Keyboard input
        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::KeyboardInput { input, .. } = event {
                if let Some(key) = input.virtual_keycode {
                    if let Some(position) = CONTROLS.iter().position(|k| k == &key) {
                        let state = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };
                        interpreter.update_key(position, state);
                    } else {
                        // Speed up or slow down
                        if (key == VKC::Minus) && iterations > 0 {
                            iterations -= 1;
                        }
                        if (key == VKC::Equals) && iterations < 1000 {
                            iterations += 1;
                        }
                    }
                }
            }
        }
        
        if let Event::RedrawRequested(_) = event {
            // Can run multiple iterations/cycles per frame
            for _ in 0..iterations {
                interpreter.execute_cycle();
            }
            interpreter.draw_pixels(pixels);
            return interpreter.should_beep();
        }
        return false;
    });
}
