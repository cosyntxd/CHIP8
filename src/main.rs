use winit::{
    window::WindowBuilder,
    event::{Event, WindowEvent, MouseButton},
    event_loop::EventLoop,
    dpi::LogicalSize,
};
use pixels::{
    PixelsBuilder,
    SurfaceTexture,
    wgpu::Color,
};

mod chip8;

use chip8::{Chip8Interpreter, WIDTH, HEIGHT};

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
                a: 1.0
            })
            .build()
            .expect("Could not create surface")
    };

    let mut interpreter = Chip8Interpreter::new();

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                control_flow.set_exit();
            },
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                surface.resize_surface(size.width, size.height)
                    .expect("Could not resize surface");
            },
            Event::WindowEvent { event: WindowEvent::DroppedFile(path), .. } => {
                if let Err(e) = interpreter.load_rom(path) {
                    println!("Could not load ROM: {e}");
                }
            }
            Event::RedrawRequested(_) => {
                interpreter.execute_cycle(surface.get_frame_mut());
                if let Err(e) = surface.render() {
                    println!("{e}");
                    control_flow.set_exit();
                }
                window.request_redraw();
            },
            _ => {},
        }
    });
}
