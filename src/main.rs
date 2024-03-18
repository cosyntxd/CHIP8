use std::{env, path::{Path, PathBuf}};

use chip8::{chip8::Chip8Interpreter, window::Chip8Window};
use winit::event::{ElementState, Event, VirtualKeyCode as VKC, WindowEvent};

const CONTROLS: [VKC; 16] = [
    VKC::Key1, VKC::Key2, VKC::Key3, VKC::Key4,
    VKC::Q,    VKC::W,    VKC::E,    VKC::R,
    VKC::A,    VKC::S,    VKC::D,    VKC::F,
    VKC::Z,    VKC::X,    VKC::C,    VKC::V,
];

fn main() {
    let window = Chip8Window::new();
    let mut interpreter = Chip8Interpreter::new();

    // Process both optional cli args of: chip8 <PATH> -d <LEVEL>
    let args: Vec<String> = env::args().collect();
    if let Some(file) = args.get(0) {
        if Path::new(file).is_file() {
            if let Err(e) = interpreter.load_rom(PathBuf::from(file)) {
                println!("Could not load ROM: {e}");
            }
        }
    }
    if let Some(debug) = args.iter().position(|arg| arg == "-d") {
        let level = args.get(debug+1)
            .and_then(|val| val.parse().ok())
            .unwrap_or(0);
        interpreter.set_debug(level)
    }

    let mut iterations = 1;
    window.run(move |event, pixels| {
        if let Event::WindowEvent { event: WindowEvent::DroppedFile(path), .. } = event {
            if let Err(e) = interpreter.load_rom(path.to_path_buf()) {
                println!("Could not load ROM: {e}");
            }
        }

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
            for _ in 0..iterations {
                interpreter.execute_cycle();
            }
            interpreter.draw_pixels(pixels);
            return interpreter.should_beep();
        }
        return false;
    });
}
