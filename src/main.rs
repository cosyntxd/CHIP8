use chip8::{chip8::Chip8Interpreter, window::Chip8Window};
use winit::event::{Event, WindowEvent};

fn main() {
    let window = Chip8Window::new();
    let mut interpreter = Chip8Interpreter::new();

    window.run(move |event, pixels| {
        interpreter.process_keys(event);
        if let Event::WindowEvent {event: WindowEvent::DroppedFile(path) ,..} = event {
            if let Err(e) = interpreter.load_rom(path.to_path_buf()) {
                println!("Could not load ROM: {e}");
            }
        }

        if let Event::RedrawRequested(_) = event {
            interpreter.execute_cycle(pixels);
            interpreter.draw_pixels(pixels);
            return interpreter.should_beep()
        }
        return false;
    });
}
