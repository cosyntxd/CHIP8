use pixels::{wgpu::Color, PixelsBuilder, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use chip8::chip8::{Chip8Interpreter, HEIGHT, WIDTH};

fn main() {
    let (width, height) = (WIDTH as u32, HEIGHT as u32);

    let event_loop = EventLoop::new();

    let window = {
        let size = LogicalSize::new(width * 4, height * 4);
        WindowBuilder::new()
            .with_title("CHIP-8 Interpreter")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .expect("Could not create window")
    };

    let mut surface = {
        let surface_texture = SurfaceTexture::new(width, height, &window);
        let darkness = 0.05;
        PixelsBuilder::new(width, height, surface_texture)
            .clear_color(Color {
                r: darkness,
                g: darkness,
                b: darkness,
                a: 1.0,
            })
            .build()
            .expect("Could not create surface")
    };

    let mut interpreter = Chip8Interpreter::new();
    use std::path::PathBuf;

    if let Err(e) = interpreter.load_rom(PathBuf::from("INVADERS")) {
        println!("{e:?}")
    }

    interpreter.clear_display(surface.frame_mut());

    event_loop.run(move |event, _, control_flow| {
        interpreter.process_keys(&event);
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.set_exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                surface
                    .resize_surface(size.width, size.height)
                    .expect("Could not resize surface");
            }
            Event::WindowEvent {
                event: WindowEvent::DroppedFile(path),
                ..
            } => {
                if let Err(e) = interpreter.load_rom(path) {
                    println!("Could not load ROM: {e}");
                }
            }
            Event::RedrawRequested(_) => {
                interpreter.execute_cycle(surface.frame_mut());
                interpreter.draw_pixels(surface.frame_mut());

                if let Err(e) = surface.render() {
                    println!("{e}");
                    control_flow.set_exit();
                }
                window.request_redraw();
            }
            _ => {}
        }
    });
}
