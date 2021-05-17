// The computers which originally used the Chip-8 Language had a 16-key hexadecimal keypad with the following layout:
//
//  1	2	3	C
//  4	5	6	D
//  7	8	9	E
//  A	0	B	F
//
// This layout must be mapped into various other configurations to fit the keyboards of today's platforms.

/// Chip-8 input
#[derive(Debug, Clone, Copy)]
pub struct Input {
    /// 16 input memory region
    pub input: [u8; 16],
}

impl Input {
    pub fn new() -> Self {
        Input {
            input: [0; 16],
        }
    }

    /// Poll specific input `key`
    pub fn poll(self, key:usize) -> u8 {
        self.input[key]
    }

    /// Set the `state` for a specific input `key`
    pub fn set(&mut self, key:usize, state: bool) {
        if state {
            self.input[key] = 1; // pressed
        } else {
            self.input[key] = 0; // not pressed
        }
    }

}