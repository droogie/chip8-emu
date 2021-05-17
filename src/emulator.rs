extern crate sdl2;

use crate::memory::{Memory};
use crate::display::{Display, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::input::{Input};
use crate::cpu::{Register};

use std::io;
use std::io::prelude::*; 

use std::fmt;
use rand::Rng;
use std::time::{Duration};

use sdl2::{event::Event, keyboard::Keycode, EventPump, audio::AudioStatus};

/// Enable Debug printing of disassembly during execution
const DEBUG_PRINT: bool = true;

/// Enable single step debugging
const DEBUG_STEP: bool = false;
/// State of the emulated system
pub struct Emulator {
    /// Memory mapping for the emulator
    pub memory: Memory,

    /// All chip-8 registers
    pub registers: Register,

    /// Display memory
    pub display: Display,

    /// Input
    pub input: Input,
    
    /// Clock tick count
    pub tick_cnt: u8,
}

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            memory: Memory::new(),
            registers: Register::new(),
            display: Display::new(),
            input: Input::new(),
            tick_cnt: 0,
        }
    }

    pub fn run(&mut self) {
        loop {
            let emu_exit = self.enter_emu().expect("Failed to execute emulator <enter_emu>!");

            match emu_exit {
                _ => { break }
            }
        }
    }

    fn enter_emu(&mut self) -> Option<()> {
        let mut event_pump = self.display.context.event_pump().unwrap();

        while self.process_events(&mut event_pump) {
            // Fetch the current instruction
            let pc = self.registers.pc;
            let inst: u16 = self.memory.read_inst(pc as usize);

            self.execute_instruction(inst);

            self.update_timers();

            self.display.update();

            // maybe move this to an audio update function
            if self.registers.st > 0 {
                if self.display.audio_device.status() == AudioStatus::Paused {
                    self.display.audio_device.resume();
                }
            } else {
                if self.display.audio_device.status() == AudioStatus::Playing {
                    self.display.audio_device.pause();
                }
            }

            // CPU is executing at 1/600th of a second, we update the timers every 10th cycle
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 600));
        }

        Some(())
    }
    
    fn update_timers(&mut self) {

        if self.tick_cnt == 10 {
            if self.registers.dt > 0 {
                self.registers.dt -= 1;
            }
    
            if self.registers.st > 0 {
                self.registers.st -= 1;
            }

            self.tick_cnt = 0;
        } else {
            self.tick_cnt += 1;
        }

    }

    fn process_events(&mut self, event_pump: &mut EventPump) -> bool {
        for event in event_pump.poll_iter() {

            match event {
                Event::Quit { .. } => { return false },
    
                Event::KeyDown {
                    keycode: Some(kc), ..
                } => self.input.set(
                match kc {
                        //  1	2	3	C
                        //  4	5	6	D
                        //  7	8	9	E
                        //  A	0	B	F

                        Keycode::Num1   => 0x01,
                        Keycode::Num2   => 0x02,
                        Keycode::Num3   => 0x03,
                        Keycode::Num4   => 0x0C,
                        Keycode::Q      => 0x04,
                        Keycode::W      => 0x05,
                        Keycode::E      => 0x06,
                        Keycode::R      => 0x0D,
                        Keycode::A      => 0x07,
                        Keycode::S      => 0x08,
                        Keycode::D      => 0x09,
                        Keycode::F      => 0x0E,
                        Keycode::Z      => 0x0A,
                        Keycode::X      => 0x00,
                        Keycode::C      => 0x0B,
                        Keycode::V      => 0x0F,
                        _ => { break },
                    }, true),
    
                Event::KeyUp {
                    keycode: Some(kc), ..
                } => self.input.set(
                match kc {
                        Keycode::Num1   => 0x01,
                        Keycode::Num2   => 0x02,
                        Keycode::Num3   => 0x03,
                        Keycode::Num4   => 0x0C,
                        Keycode::Q      => 0x04,
                        Keycode::W      => 0x05,
                        Keycode::E      => 0x06,
                        Keycode::R      => 0x0D,
                        Keycode::A      => 0x07,
                        Keycode::S      => 0x08,
                        Keycode::D      => 0x09,
                        Keycode::F      => 0x0E,
                        Keycode::Z      => 0x0A,
                        Keycode::X      => 0x00,
                        Keycode::C      => 0x0B,
                        Keycode::V      => 0x0F,
                        _ => { break },
                    }, false),
            _ => (),
            }
        }

        true
    }
    
    fn execute_instruction(&mut self, inst: u16) -> Option<()> {
        match (inst >> 12) & 0xff {
            0x0 => {
                match inst & 0xff
                {
                    0x0e0 => {
                        // 00E0 - CLS
                        // Clear the display.

                        if DEBUG_PRINT {
                            println!("cls");
                        }

                        self.display.clear();

                    },
                    0xee => {
                        // 00EE - RET
                        // Return from a subroutine.
                        // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    
                        if DEBUG_PRINT {
                            println!("ret");
                        }

                        self.registers.pc = self.memory.pop();

                    },
                    _ => {
                        unimplemented!("Unknown instruction: {:#04x}", inst);
                    }
                }
            },
            0x1 => {
                // 1nnn - JP addr
                // Jump to location nnn.                
                // The interpreter sets the program counter to nnn.

                let addr = inst & 0xfff;

                if DEBUG_PRINT {
                    println!("jp {:#04x}", addr);
                }

                self.registers.pc = addr;
                self.registers.pc -= 2; // adjusting here due to the auto pc increase at bottom
            },
            0x2 => {
                // 2nnn - CALL addr
                // Call subroutine at nnn.
                // The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
                
                let addr = inst & 0xfff;

                if DEBUG_PRINT {
                    println!("call {:#04x}", addr);
                }

                self.memory.push(self.registers.pc);
                self.registers.sp = self.memory.sp as u16;
                self.registers.pc = addr;
                self.registers.pc -= 2; // adjusting here due to the auto pc increase at bottom
            },
            0x3 => {
                // 3xkk - SE Vx, byte
                // Skip next instruction if Vx = kk.
                // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.

                let reg = ((inst >> 8) & 0xf) as u8;
                let val = (inst & 0xff) as u8;

                if DEBUG_PRINT {
                    println!("se v{:x}, {:#02x}", reg, val);
                }

                if self.registers.reg_read(reg) == val {
                    self.registers.pc += 2;
                }
            },
            0x4 => {
                // 4xkk - SNE Vx, byte
                // Skip next instruction if Vx != kk.
                // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
                let reg = ((inst >> 8) & 0xf) as u8;
                let val = (inst & 0xff) as u8;

                if DEBUG_PRINT {
                    println!("sne v{:x}, {:#02x}", reg, val);
                }

                if self.registers.reg_read(reg) != val {
                    self.registers.pc += 2;
                }
            },
            0x5 => {
                // 5xy0 - SE Vx, Vy
                // Skip next instruction if Vx = Vy.
                // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
                let regx = ((inst >> 8) & 0xf) as u8;
                let regy = ((inst >> 4) & 0xf) as u8;

                if DEBUG_PRINT {
                    println!("se v{:x}, v{:x}", regx, regy);
                }

                if self.registers.reg_read(regx) == self.registers.reg_read(regy) {
                    self.registers.pc += 2;
                }
            },
            0x6 => {
                // 6xkk - LD Vx, byte
                // Set Vx = kk.
                // The interpreter puts the value kk into register Vx.

                let reg = ((inst >> 8) & 0xf) as u8;
                let val = (inst & 0xff) as u8;

                if DEBUG_PRINT {
                    println!("ld v{:x}, {:#02x}", reg, val);
                }

                self.registers.reg_write(reg, val);
            },
            0x7 => {
                // 7xkk - ADD Vx, byte
                // Set Vx = Vx + kk.
                // Adds the value kk to the value of register Vx, then stores the result in Vx.

                let reg = ((inst >> 8) & 0xf) as u8;
                let val = (inst & 0xff) as u8;

                if DEBUG_PRINT {
                    println!("add v{:x}, {:#02x}", reg, val);
                }

                let x = self.registers.reg_read(reg);
                
                // assuming wraps but not certain
                self.registers.reg_write(reg, x.wrapping_add(val));
            },
            0x8 => {
                // 8xy0 - LD Vx, Vy
                // 8xy1 - OR Vx, Vy
                // 8xy2 - AND Vx, Vy
                // 8xy3 - XOR Vx, Vy
                // 8xy4 - ADD Vx, Vy
                // 8xy5 - SUB Vx, Vy
                // 8xy6 - SHR Vx {, Vy}
                // 8xy7 - SUBN Vx, Vy
                // 8xyE - SHL Vx {, Vy}
                match inst & 0xf {
                    0x0 => {
                        // 8xy0 - LD Vx, Vy
                        // Set Vx = Vy.
                        // Stores the value of register Vy in register Vx.

                        let regx = ((inst >> 8) & 0xf) as u8;
                        let regy = ((inst >> 4) & 0xf) as u8;
                        
                        if DEBUG_PRINT {
                            println!("ld v{:x}, v{:x}", regx, regy);
                        }        

                        let y = self.registers.reg_read(regy);
                        self.registers.reg_write(regx, y);
                    },
                    0x1 => {
                        // 8xy1 - OR Vx, Vy
                        // Set Vx = Vx OR Vy.
                        // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. 
                        // A bitwise OR compares the corrseponding bits from two values, and if either bit is 1, then the same bit in the result is also 1. Otherwise, it is 0.
                        
                        let regx = ((inst >> 8) & 0xf) as u8;
                        let regy = ((inst >> 4) & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("or v{:x}, v{:x}", regx, regy);
                        }        

                        let x = self.registers.reg_read(regx);
                        let y = self.registers.reg_read(regy);

                        self.registers.reg_write(regx, x | y);
                        
                    },
                    0x2 => {
                        // 8xy2 - AND Vx, Vy
                        // Set Vx = Vx AND Vy.                        
                        // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. 
                        // A bitwise AND compares the corrseponding bits from two values, and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.

                        let regx = ((inst >> 8) & 0xf) as u8;
                        let regy = ((inst >> 4) & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("and v{:x}, v{:x}", regx, regy);
                        }        

                        let x = self.registers.reg_read(regx);
                        let y = self.registers.reg_read(regy);

                        self.registers.reg_write(regx, x & y);

                    },
                    0x3 => {
                        // 8xy3 - XOR Vx, Vy
                        // Set Vx = Vx XOR Vy.
                        // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx. 
                        //An exclusive OR compares the corrseponding bits from two values, and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
                        
                        let regx = ((inst >> 8) & 0xf) as u8;
                        let regy = ((inst >> 4) & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("xor v{:x}, v{:x}", regx, regy);
                        }        

                        let x = self.registers.reg_read(regx);
                        let y = self.registers.reg_read(regy);

                        self.registers.reg_write(regx, x ^ y);

                    },
                    0x4 => {
                        // 8xy4 - ADD Vx, Vy
                        // Set Vx = Vx + Vy, set VF = carry.
                        // The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
                        
                        let regx = ((inst >> 8) & 0xf) as u8;
                        let regy = ((inst >> 4) & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("add v{:x}, v{:x}", regx, regy);
                        } 

                        let x = self.registers.reg_read(regx);
                        let y = self.registers.reg_read(regy);

                        let res: u16 = x as u16 + y as u16;
                        if res > 255 {
                            self.registers.vf = 1;
                        } else {
                            self.registers.vf = 0;
                        }

                        self.registers.reg_write(regx, res as u8);
                    },
                    0x5 => {
                        // 8xy5 - SUB Vx, Vy
                        // Set Vx = Vx - Vy, set VF = NOT borrow.                        
                        // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.

                        let regx = ((inst >> 8) & 0xf) as u8;
                        let regy = ((inst >> 4) & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("sub v{:x}, v{:x}", regx, regy);
                        } 

                        let x = self.registers.reg_read(regx);
                        let y = self.registers.reg_read(regy);

                        if x > y {
                            self.registers.vf = 1;
                        } else {
                            self.registers.vf = 0;
                        }

                        // assuming this is supposed to wrap, but not certain
                        self.registers.reg_write(regx, x.wrapping_sub(y));
                    },
                    0x6 => {
                        // 8xy6 - SHR Vx {, Vy}
                        // Set Vx = Vx SHR 1.
                        // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.

                        // Not sure what is supposed to happen with Vy here???
                        let regx = ((inst >> 8) & 0xf) as u8;
                        let regy = ((inst >> 4) & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("shr v{:x} {{, v{:x}}}", regx, regy);
                        } 

                        let x = self.registers.reg_read(regx);
                        let _y = self.registers.reg_read(regy);

                        self.registers.vf = x & 0x1;
                        self.registers.reg_write(regx, x >> 1);

                    },
                    0x7 => {
                        // 8xy7 - SUBN Vx, Vy
                        // Set Vx = Vy - Vx, set VF = NOT borrow.
                        // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
                        
                        let regx = ((inst >> 8) & 0xf) as u8;
                        let regy = ((inst >> 4) & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("subn v{:x}, v{:x}", regx, regy);
                        } 

                        let x = self.registers.reg_read(regx);
                        let y = self.registers.reg_read(regy);

                        if y > x {
                            self.registers.vf = 1;
                        } else {
                            self.registers.vf = 0;
                        }

                        // assuming this is supposed to wrap, but not certain
                        self.registers.reg_write(regx, y.wrapping_sub(x));

                    },
                    0xe => {
                        // 8xyE - SHL Vx {, Vy}
                        // Set Vx = Vx SHL 1.
                        // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.

                        // Not sure what is supposed to happen with Vy here???
                        let regx = ((inst >> 8) & 0xf) as u8;
                        let regy = ((inst >> 4) & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("shl v{:x} {{, v{:x}}}", regx, regy);
                        } 

                        let x = self.registers.reg_read(regx);
                        let _y = self.registers.reg_read(regy);

                        if x & 0x7 == 1 {
                            self.registers.vf = 1;
                        } else {
                            self.registers.vf = 0;
                        }
                        self.registers.reg_write(regx, x << 1);
                    },
                    _ => {
                        println!("{}", self.registers);
                        unimplemented!("Unknown 8 instruction {:#04x}", inst);
                    }
                }
            },
            0x9 => {
                // 9xy0 - SNE Vx, Vy
                // Skip next instruction if Vx != Vy.
                // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.

                let regx = ((inst >> 8) & 0xf) as u8;
                let regy = ((inst >> 4) & 0xf) as u8;

                if DEBUG_PRINT {
                    println!("sne v{:x} {{, v{:x}}}", regx, regy);
                } 

                let x = self.registers.reg_read(regx);
                let y = self.registers.reg_read(regy);

                if x != y {
                    self.registers.pc += 2;
                }
            },
            0xa => { 
                // Annn - LD I, addr
                // Set I = nnn.
                // The value of register I is set to nnn.
                let addr = inst & 0xfff;

                if DEBUG_PRINT {
                    println!("ld i, {:#02x}", addr);
                }

                self.registers.i = addr;
            },
            0xb => {
                // Bnnn - JP V0, addr
                // Jump to location nnn + V0.                
                // The program counter is set to nnn plus the value of V0.
                
                let addr = inst & 0xfff;

                if DEBUG_PRINT {
                    println!("jp v0, {:#02x}", addr);
                }

                self.registers.pc = self.registers.v0 as u16 + addr;
                self.registers.pc -= 2; // adjusting here due to the auto pc increase at bottom
            },
            0xc => {
                // Cxkk - RND Vx, byte
                // Set Vx = random byte AND kk.
                // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx.

                let reg = ((inst >> 8) & 0xf) as u8;
                let val = (inst & 0xff) as u8;
                let rnum = rand::thread_rng().gen_range(0..=255);

                if DEBUG_PRINT {
                    println!("rnd v{:x}, {:#02x}", reg, val);
                }


                self.registers.reg_write(reg, val & rnum);
            },
            0xd => {
                // Dxyn - DRW Vx, Vy, nibble
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                // The interpreter reads n bytes from memory, starting at the address stored in I. 
                // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). 
                // Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. 
                // If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen. 
                
                let x = ((inst >> 8) & 0xf) as u8;
                let y = ((inst >> 4) & 0xf) as u8;
                let n = (inst & 0xf) as u8;

                if DEBUG_PRINT {
                    println!("drw v{:x}, v{:x}, {}", x, y, n);
                }        

                let begin_addr = self.registers.i as usize;
                let end_addr = begin_addr + n as usize;
                if begin_addr >= self.memory.memory.len() || end_addr > self.memory.memory.len() {
                    println!("begin addr issue!");
                }

                let x_begin = std::cmp::min(self.registers.reg_read(x) as usize & 0x3f, (SCREEN_WIDTH - 1) as usize);
                let y_begin = std::cmp::min(self.registers.reg_read(y) as usize & 0x1f, (SCREEN_HEIGHT - 1) as usize);
                let mut collision = 0;
        
                for dy in 0..n as usize {
                    let yc = y_begin + dy;
                    if yc >= SCREEN_HEIGHT as usize {
                        break;
                    }
        
                    for dx in 0..8 {
                        let xc = x_begin + dx;
                        if xc >= SCREEN_WIDTH as usize {
                            break;
                        }
        
                        if self.memory.memory[begin_addr + dy] & (0x80 >> dx) != 0 {
                            let offset = xc + SCREEN_WIDTH as usize * yc;
                            collision |= self.display.memory[offset];
                            self.display.memory[offset] = !self.display.memory[offset];
                        }
                    }
                }
        
                self.registers.vf = if collision != 0 { 1 } else { 0 };        

            },
            0xe => {
                match inst & 0xff {
                    0x9e => {
                        // Ex9E - SKP Vx
                        // Skip next instruction if key with the value of Vx is pressed.
                        // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.

                        let reg = ((inst >> 8) & 0xf) as u8;
                        let val = self.registers.reg_read(reg);

                        if DEBUG_PRINT {
                            println!("skp v{:x}", reg);
                        }

                        if self.input.poll(val as usize) == 1 {
                            self.registers.pc += 2;
                        }
                    },
                    0xa1 => {
                        // ExA1 - SKNP Vx
                        // Skip next instruction if key with the value of Vx is not pressed.
                        // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
                        
                        let reg = ((inst >> 8) & 0xf) as u8;
                        let val = self.registers.reg_read(reg);

                        if DEBUG_PRINT {
                            println!("sknp v{:x}", reg);
                        }        

                        if self.input.poll(val as usize) == 0 {
                            self.registers.pc += 2;
                        }

                    },
                    _ => {
                        println!("{}", self.registers);
                        unimplemented!("Unknown E instruction {:#04x}", inst);        
                    }
                }
            },
            0xf => {
                match inst & 0xff {
                    // Fx07 - LD Vx, DT
                    // Fx0A - LD Vx, K
                    // Fx15 - LD DT, Vx
                    // Fx18 - LD ST, Vx
                    // Fx1E - ADD I, Vx
                    // Fx29 - LD F, Vx
                    // Fx33 - LD B, Vx
                    // Fx55 - LD [I], Vx
                    // Fx65 - LD Vx, [I]
                    0x07 => {
                        // Fx07 - LD Vx, DT
                        // Set Vx = delay timer value.
                        // The value of DT is placed into Vx.
                        
                        let reg = (inst >> 8 & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("ld v{:x}, dt", reg);
                        }

                        self.registers.reg_write(reg, self.registers.dt);
                    },
                    0xa => {
                        // Fx0A - LD Vx, K
                        // Wait for a key press, store the value of the key in Vx.
                        // All execution stops until a key is pressed, then the value of that key is stored in Vx.
                        let mut keypress: bool = false;
                        let reg = ((inst >> 8) & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("ld v{:x}, k", reg);
                        }

                        for i in 0..self.input.input.len() {
                            if self.input.poll(i) == 1 {
                                self.registers.reg_write(reg, i as u8);
                                keypress = true;
                            }
                        }

                        // this will essentially loop while polling input for change, so PC does not update
                        if !keypress {
                            self.registers.pc -= 2;
                        }
                    },
                    0x15 => {
                        // Fx15 - LD DT, Vx
                        // Set delay timer = Vx.
                        // DT is set equal to the value of Vx.
                        
                        let reg = (inst >> 8 & 0xf) as u8;
                        
                        if DEBUG_PRINT {
                            println!("ld dt, v{:x}", reg);
                        }

                        self.registers.dt = self.registers.reg_read(reg);
                    },
                    0x18 => {
                        // Fx18 - LD ST, Vx
                        // Set sound timer = Vx.
                        // ST is set equal to the value of Vx.
                        
                        let reg = ((inst >>8) & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("ld st, v{:x}", reg);
                        }

                        self.registers.st = self.registers.reg_read(reg);
                    },
                    0x1e => {
                        // Fx1E - ADD I, Vx
                        // Set I = I + Vx.
                        // The values of I and Vx are added, and the results are stored in I.
                        
                        let reg = ((inst >>8) & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("add i, v{:x}", reg);
                        }

                        self.registers.i = self.registers.i + self.registers.reg_read(reg) as u16;
                    },
                    0x29 => {
                        // Fx29 - LD F, Vx
                        // Set I = location of sprite for digit Vx.
                        // The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx. 

                        // I believe this is essentially using reg# as an index into the FONTS region which starts at 0x00.
                        // So for each Register value 0..F we index into FONTS by that value * 5.
                        let reg = (inst >> 8 & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("ld f, v{:x}", reg);
                        }

                        self.registers.i = self.registers.reg_read(reg) as u16 * 5;
                    },
                    0x33 => {
                        // Fx33 - LD B, Vx
                        // Store BCD representation of Vx in memory locations I, I+1, and I+2.
                        // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, 
                        // the tens digit at location I+1, and the ones digit at location I+2.
                        let reg = (inst >> 8 & 0xf) as u8;
                        let val = self.registers.reg_read(reg);
                        let h = (val / 100) % 10;
                        let t = (val / 10) % 10;
                        let o = val % 10;
                        if DEBUG_PRINT {
                            println!("ld b, v{:x}", reg);
                        }

                        self.memory.write(self.registers.i as usize, h);
                        self.memory.write((self.registers.i+1) as usize, t);
                        self.memory.write((self.registers.i+2) as usize, o);

                    },
                    0x55 => {
                        // Fx55 - LD [I], Vx
                        // Store registers V0 through Vx in memory starting at location I.                        
                        // The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.

                        let reg = ((inst >> 8) & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("ld [i], v{:x}", reg);
                        }

                        for i in 0..=reg {
                            self.memory.memory[self.registers.i as usize + i as usize] = self.registers.reg_read(i);
                        }

                    },
                    0x65 => {
                        // Fx65 - LD Vx, [I]
                        // Read registers V0 through Vx from memory starting at location I.
                        // The interpreter reads values from memory starting at location I into registers V0 through Vx.

                        let reg = (inst >> 8 & 0xf) as u8;

                        if DEBUG_PRINT {
                            println!("ld  v{:x}, [I]", reg);
                        }

                        for i in 0..=reg {
                            self.registers.reg_write(i, self.memory.read((self.registers.i + i as u16) as usize));
                        }
                    },
                    _ => {
                        println!("{}", self.registers);
                        unimplemented!("Unknown F instruction {:#04x}", inst);
                    }
        
                }
            }
            _ => {
                println!("{}", self.registers);
                unimplemented!("Unknown instruction {:#04x}", inst);
            }
        }

        self.registers.pc += 2;

        if DEBUG_STEP {
            println!("{}", self.registers);
            println!("{:?}", &self.memory.memory[self.registers.i as usize..self.registers.i as usize + 10]);
            pause();
        }

        Some(())
    }
}

// used for single step debugging
fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
r#"
v0[{:02x}]   v1[{:02x}]   v2[{:02x}]   v3[{:02x}]
v4[{:02x}]   v5[{:02x}]   v6[{:02x}]   v7[{:02x}]
v8[{:02x}]   v9[{:02x}]   va[{:02x}]   vb[{:02x}]
vc[{:02x}]   vd[{:02x}]   ve[{:02x}]   vf[{:02x}]
dt[{:02x}]   st[{:02x}]

 i[{:04x}]
sp[{:04x}]
pc[{:04x}]
"#,    
    self.v0,
    self.v1,
    self.v2,
    self.v3,
    self.v4,
    self.v5,
    self.v6,
    self.v7,
    self.v8,
    self.v9,
    self.va,
    self.vb,
    self.vc,
    self.vd,
    self.ve,
    self.vf,
    self.dt,
    self.st,
    self.i,
    self.sp,
    self.pc,
        )
    }
}

