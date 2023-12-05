/// Interrupt codes from 6510
pub enum OpCode {
    // Misc
    Nop = 0xEA,
    
    // Interrupts
    Brk = 0x00,
    Rti = 0x40,

    // Subroutines
    Jsr = 0x20,
    Rts = 0x60,

    // Clear flags
    Clc = 0x18,
    Cld = 0xD8,
    Cli = 0x58,
    Clv = 0xB8,

    Sec = 0x38,
    Sed = 0xF8,
    Sei = 0x78,

    // Register operations
    Dex = 0xCA,
    Dey = 0x88,
}

impl From<OpCode> for u8 {
    fn from(op_code: OpCode) -> u8 {
        op_code as u8
    }
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0xEA => OpCode::Nop,
            0x00 => OpCode::Brk,
            0x40 => OpCode::Rti,
            0x20 => OpCode::Jsr,
            0x60 => OpCode::Rts,
            0x18 => OpCode::Clc,
            0xD8 => OpCode::Cld,
            0x58 => OpCode::Cli,
            0xB8 => OpCode::Clv,
            0x38 => OpCode::Sec,
            0xF8 => OpCode::Sed,
            0x78 => OpCode::Sei,
            0xCA => OpCode::Dex,
            0x88 => OpCode::Dey,
            _ => panic!("Unknown OpCode: {:#04x}", value)
        }
    }
}
