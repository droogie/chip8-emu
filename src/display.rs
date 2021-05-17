extern crate sdl2;

// The original implementation of the Chip-8 language used a 64x32-pixel monochrome display with this format:
// ___________________
// |                 |
// | (0,0)	 (63,0)  |
// |                 |
// | (0,31)	 (63,31) |
// |_________________|

// The sound timer is active whenever the sound timer register (ST) is non-zero. 
// This timer also decrements at a rate of 60Hz, however, as long as ST's value is greater than zero, the Chip-8 buzzer will sound. 
// When ST reaches zero, the sound timer deactivates.

// The audio code is essentially the rust sdl2 audio squarewave example
pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

use sdl2::{Sdl, render::Canvas, video::Window, pixels::Color, rect::Rect};
use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioDevice};
use sdl2::pixels;

pub const SCREEN_HEIGHT: u16 = 32;
pub const SCREEN_WIDTH: u16 = 64;
const SCREEN_SCALE: u16 = 16;

/// Chip-8 display memory
pub struct Display {
    /// 64x32 pixel memory region
    pub memory: [u8; 2048],

    /// SDL2 Context
    pub context: Sdl,

    /// SDL2 Canvas
    pub canvas: Canvas<Window>,

    /// SDL2 Audio device
    pub audio_device: AudioDevice<SquareWave>,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

impl Display {
    pub fn new() -> Self {

        let sdl_context = sdl2::init().unwrap();
        let video_subsys = sdl_context.video().unwrap();
        let audio_subsys = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(44_100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };

        let audio_device = audio_subsys.open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.10,
            }
        }).unwrap();

        let window = video_subsys
            .window(
                "CHIP-8 Emulator",
                SCREEN_WIDTH as u32 * SCREEN_SCALE as u32,
                SCREEN_HEIGHT as u32 * SCREEN_SCALE as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string()).unwrap();
    
        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
    
        Display {
            memory: [0; 2048],
            context: sdl_context,
            canvas: canvas,
            audio_device: audio_device,
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.memory.len() {
            self.memory[i] = 0;
        }
    }

    pub fn update(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
    
        for idx in 0..self.memory.len() {
            if self.memory[idx] != 0 {
                self.canvas.fill_rect(Rect::new(
                    (SCREEN_SCALE as u32 * (idx as u32 & 0x3f)) as i32,
                    (SCREEN_SCALE as u32 * (idx as u32 >> 6)) as i32,
                    SCREEN_SCALE as u32,
                    SCREEN_SCALE as u32,
                )).unwrap();
            }
        }
    
        self.canvas.present();

    }    
}