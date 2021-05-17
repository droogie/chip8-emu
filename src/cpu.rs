// 16 x 8-bit GPIO Registers
// 1  x 16-bit I Register

#[derive(Debug, Clone, Copy)]
pub struct Register {
    pub v0:  u8,
    pub v1:  u8,
    pub v2:  u8,
    pub v3:  u8,
    pub v4:  u8,
    pub v5:  u8,
    pub v6:  u8,
    pub v7:  u8,
    pub v8:  u8,
    pub v9:  u8,
    pub va:  u8,
    pub vb:  u8,
    pub vc:  u8,
    pub vd:  u8,
    pub ve:  u8,
    pub vf:  u8,
    pub dt:  u8,
    pub st:  u8,
    pub  i:  u16,
    pub pc:  u16,
    pub sp:  u16,
}

impl Register {
    pub fn new() -> Self {
        Register {
            v0: 0,
            v1: 0,
            v2: 0,
            v3: 0,
            v4: 0,
            v5: 0,
            v6: 0,
            v7: 0,
            v8: 0,
            v9: 0,
            va: 0,
            vb: 0,
            vc: 0,
            vd: 0,
            ve: 0,
            vf: 0,
            dt: 0,
            st: 0,        
             i: 0,
            pc: 0x200, // Game Data
            sp: 0,
        }
    }

    /// Write to specific V register
    pub fn reg_write(&mut self, reg:u8, val: u8) {
        match reg {
            0 => self.v0 = val,
            1 => self.v1 = val,
            2 => self.v2 = val,
            3 => self.v3 = val,
            4 => self.v4 = val,
            5 => self.v5 = val,
            6 => self.v6 = val,
            7 => self.v7 = val,
            8 => self.v8 = val,
            9 => self.v9 = val,
            10 => self.va = val,
            11 => self.vb = val,
            12 => self.vc = val,
            13 => self.vd = val,
            14 => self.ve = val,
            15 => self.vf = val,            
            _ => {
                panic!("Invalid GPIO Register!");
            }
        }
    }

    /// Read from specific V register
    pub fn reg_read(&mut self, reg:u8) -> u8 {
        match reg {
            0 =>  return self.v0,
            1 =>  return self.v1,
            2 =>  return self.v2,
            3 =>  return self.v3,
            4 =>  return self.v4,
            5 =>  return self.v5,
            6 =>  return self.v6,
            7 =>  return self.v7,
            8 =>  return self.v8,
            9 =>  return self.v9,
            10 => return self.va,
            11 => return self.vb,
            12 => return self.vc,
            13 => return self.vd,
            14 => return self.ve,
            15 => return self.vf,            
            _ => {
                panic!("Invalid GPIO Register!");
            }
        }
    }

}