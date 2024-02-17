use pixels::{wgpu::Color, Pixels, PixelsBuilder, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::chip8::{HEIGHT, WIDTH};

pub struct Chip8Window {
    surface: Pixels,
    window: Window,
    event_loop: EventLoop<()>,
}
impl Chip8Window {
    pub fn new() -> Self {
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

        let surface = {
            let surface_texture = SurfaceTexture::new(width, height, &window);
            const DARKNESS: f64 = 0.05;
            PixelsBuilder::new(width, height, surface_texture)
                .clear_color(Color {
                    r: DARKNESS,
                    g: DARKNESS,
                    b: DARKNESS,
                    a: 1.0,
                })
                .build()
                .expect("Could not create surface")
        };
        Self {
            surface,
            window,
            event_loop,
        }
    }
    pub fn run<F>(mut self, mut func: F)
    where
        F: 'static + FnMut(&Event<'_, ()>, &mut [u8]),
    {
        self.event_loop.run(move |event, _, control_flow| {
            if let Event::WindowEvent {event: WindowEvent::CloseRequested, ..} = event {
                control_flow.set_exit();
            }
            if let Event::WindowEvent {event: WindowEvent::Resized(size), ..} = event {
                self.surface
                    .resize_surface(size.width, size.height)
                    .expect("Could not resize surface");
            }

            if let Event::RedrawRequested(_) = event {
                func(&event, &mut self.surface.frame_mut());
                if let Err(e) = self.surface.render() {
                    println!("{e}");
                    control_flow.set_exit();
                }
                self.window.request_redraw();
            }
        });
    }
}
