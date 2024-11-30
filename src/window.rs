use std::{path::PathBuf, time::{Duration, Instant}};

use pixels::{wgpu::Color, Pixels, PixelsBuilder, SurfaceTexture};
use winit::event::{ElementState, VirtualKeyCode as VKC};
use rodio::{Device, Source};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
use crate::chip8::{HEIGHT, WIDTH};

/// An easy way to interact with window, pixel buffer and audio
pub struct Chip8Window {
    surface: Pixels,
    window: Window,
    event_loop: EventLoop<()>,
    audio: Device,
}
impl Default for Chip8Window {
    fn default() -> Self {
        Self::new()
    }
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
        let audio = rodio::default_output_device().unwrap();
        Self {
            surface,
            window,
            event_loop,
            audio,
        }
    }
    pub fn run<F>(mut self, mut func: F)
    where
        F: 'static + FnMut(GameEvents, &mut [u8]) -> bool,
    {
        let target_ms = 1_000_000f32 /
            self.window.primary_monitor()
                .and_then(|monitor| monitor.refresh_rate_millihertz())
                .unwrap_or(60_000) as f32;
        
        let mut last_draw = Instant::now();

        self.event_loop.run(move |event, _, control_flow| {
            if let Event::WindowEvent { event: WindowEvent::CloseRequested, .. } = event {
                control_flow.set_exit();
            }
            
            if let Event::WindowEvent { event: WindowEvent::Resized(size), .. } = event {
                self.surface
                    .resize_surface(size.width, size.height)
                    .expect("Could not resize surface");
            }

            if let Event::WindowEvent { event: WindowEvent::DroppedFile(path), .. } = &event {
                func(GameEvents::DroppedFile(path.clone()), self.surface.frame_mut());
            }
            if let Event::WindowEvent { ref event, .. } = event {
                if let WindowEvent::KeyboardInput { input, .. } = event {
                    if let Some(key) = input.virtual_keycode {
                        println!("{key:?}");
                        if let Some(position) = CONTROLS.iter().position(|k| k == &key) {
                            let state = match input.state {
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            };
                            println!("{position}");
                            func(GameEvents::KeyInput(position, state), self.surface.frame_mut());
                        } else {
                            // TODO: user feedback
                        }
                    }
                }
            }

            if let &Event::RedrawRequested(_) = &event {
                let beep = func(GameEvents::Redraw, self.surface.frame_mut());

                let time_now = Instant::now();

                let frame_time = time_now.duration_since(last_draw);

                let sleep = Duration::from_secs_f32(target_ms / 1000.0).saturating_sub(frame_time);
                last_draw = time_now;

                std::thread::sleep(sleep);

                if beep {
                    let source = rodio::source::SineWave::new(400);
                    rodio::play_raw(
                        &self.audio,
                        source.take_duration(Duration::from_secs_f32(target_ms / 1000.0)),
                    );
                }

                if let Err(e) = self.surface.render() {
                    println!("{e}");
                    control_flow.set_exit();
                }
                self.window.request_redraw();
            }

        });
    }
}
pub const MINUS_POSITION: usize = 16;
pub const PLUS_POSITION: usize = 17;

// Hexademical keyboard remapped to qwerty
const CONTROLS: [VKC; 18] = [
    VKC::Key1, VKC::Key2, VKC::Key3, VKC::Key4,
    VKC::Q,    VKC::W,    VKC::E,    VKC::R,
    VKC::A,    VKC::S,    VKC::D,    VKC::F,
    VKC::Z,    VKC::X,    VKC::C,    VKC::V,
    VKC::Minus, VKC::Equals, // special
];

pub enum GameEvents {
    DroppedFile(PathBuf),
    KeyInput(usize, bool),
    Redraw,
}