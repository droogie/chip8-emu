use std::path::Path;

// Memory Map:
// +---------------+= 0xFFF (4095) End of Chip-8 RAM
// |               |
// |               |
// |               |
// |               |
// |               |
// | 0x200 to 0xFFF|
// |     Chip-8    |
// | Program / Data|
// |     Space     |
// |               |
// |               |
// |               |
// +- - - - - - - -+= 0x600 (1536) Start of ETI 660 Chip-8 programs
// |               |
// |               |
// |               |
// +---------------+= 0x200 (512) Start of most Chip-8 programs
// | 0x000 to 0x1FF|
// | Reserved for  |
// |  interpreter  |
// +---------------+= 0x000 (0) Start of Chip-8 RAM

/// Reserved Font Data for Interpreters
const FONTS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,   // 0
    0x20, 0x60, 0x20, 0x20, 0x70,   // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0,   // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0,   // 3
    0x90, 0x90, 0xF0, 0x10, 0x10,   // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0,   // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0,   // 6
    0xF0, 0x10, 0x20, 0x40, 0x40,   // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0,   // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0,   // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90,   // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0,   // B
    0xF0, 0x80, 0x80, 0x80, 0xF0,   // C
    0xE0, 0x90, 0x90, 0x90, 0xE0,   // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0,   // E
    0xF0, 0x80, 0xF0, 0x80, 0x80,   // F
];

/// Offset to Game Data Region of Memory Map
const GAME_DATA_OFFSET: usize = 512;

/// Chip-8 memory mapping
#[derive(Debug, Clone, Copy)]
pub struct Memory {
    /// 4K memory region
    pub memory: [u8; 4096],

    /// Stack
    pub stack: [u16;   16],

    /// Stack Pointer
    pub sp: usize,

}

impl Memory {
    pub fn new() -> Self {

        let mut memory = Memory {
            memory: [0; 4096],
            stack: [0; 16],
            sp: usize::MAX,
        };

        // Copy font data into memory region
        memory.memory[..80].copy_from_slice(&FONTS);

        memory
    }

    /// Read from memory
    pub fn read(self, offset: usize) -> u8 {
        self.memory[offset]
    }

    /// Write to memory
    pub fn write(&mut self, offset: usize, data: u8) {
        self.memory[offset] = data
    }

    /// Pop a value off of the stack
    pub fn pop(&mut self) -> u16 {
        assert!(self.sp != usize::MAX);
        let data = self.stack[self.sp];
        self.sp = self.sp.wrapping_sub(1);

        data
    }

    /// Push a value onto the stack
    pub fn push(&mut self, val: u16) {
        assert!(self.sp != 15);
        self.sp = self.sp.wrapping_add(1);
        self.stack[self.sp] = val;
    }

    /// Read the instruction at `offset`
    pub fn read_inst(self, offset: usize) -> u16 {
        if offset >= 4096 {
            panic!("read_inst: out of bounds read!");
        }

        // I originally had this due to a statement in cowgod's chip-8 technical reference stating
        // that instructions need to start on even addresses, but this appears to be an inaccurate statement

        // if offset & 1 != 0 {
        //     panic!("Instruction not on even memory alignment");
        // }

        ((self.read(offset) as u16) << 8) + self.read(offset+1) as u16
    }

    /// Load a game file into the game data memory region
    pub fn load<P: AsRef<Path>>(&mut self, filename: P) -> Option<()> {

        // Read the input file
        let contents = std::fs::read(filename).ok().expect("Failed reading file contents");

        // Write game data into game data memory region
        self.memory[GAME_DATA_OFFSET..GAME_DATA_OFFSET + contents.len()].copy_from_slice(&contents);

        Some(())
    }
}