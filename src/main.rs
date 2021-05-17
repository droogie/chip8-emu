pub mod cpu;
pub mod memory;
pub mod display;
pub mod input;
pub mod emulator;

use emulator::{Emulator};

fn main() -> Result<(), String> {

    let mut emu = Emulator::new();

    emu.memory.load("games/PONG");

    emu.run();

    Ok(())
}