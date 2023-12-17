pub mod opcodes;

use memory::Memory;
use opcodes::OpCode;
use std::cell::RefCell;
use std::rc::Rc;

const CARRY_FLAG: u8 = 0b0000_0001;
const ZERO_FLAG: u8 = 0b0000_0010;
const INTERRUPT_DISABLE_FLAG: u8 = 0b0000_0100;
const DECIMAL_MODE_FLAG: u8 = 0b0000_1000;
const BREAK_FLAG: u8 = 0b0001_0000;
const OVERFLOW_FLAG: u8 = 0b0100_0000;
const NEGATIVE_FLAG: u8 = 0b1000_0000;

/// A MOS 6502 CPU.
/// Decimal mode is not yet supported.
pub struct Mos6510 {
    a: u8,
    x: u8,
    y: u8,

    sp: u8,
    ps: u8,
    pc: u16,

    halted: bool,

    mem: Rc<RefCell<Memory>>,
}

impl Mos6510 {
    /// Creates a new `Cpu` instance.
    /// However, this method does not initialize the CPU to its initial state.
    /// To do that, call `reset()` after creating a new `Cpu` instance.
    ///
    /// # Arguments
    ///
    /// * `mem` - A shared pointer to a `Memory` instance. Memory must be initialized first.
    ///          See `memory::Memory::new()`.
    ///
    /// # Returns
    ///
    /// A new `Cpu` instance.
    pub fn new(mem: Rc<RefCell<Memory>>) -> Self {
        Mos6510 {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            sp: 0x00,
            ps: 0x00,
            pc: 0x00,
            halted: false,
            mem: mem,
        }
    }

    /// Resets the CPU to its initial state.
    pub fn reset(&mut self) {
        self.a = 0x00;
        self.x = 0x00;
        self.y = 0x00;

        // however, we're not executing any code, so we'll just set it to 0xff
        // it will be set automatically when we load the c64 kernal rom
        self.sp = 0x00;

        self.ps = 0x00;
        self.pc = self.mem.borrow().get_reset_vector();
    }

    /// Halts/resumes the CPU.
    /// If the CPU is halted, it will not execute any instructions.
    /// halted => !halted
    pub fn halt_resume(&mut self) {
        self.halted = !self.halted;
    }

    pub fn step(&mut self) {
        if !self.halted {
            let op_code: u8 = self.fetch();
            #[cfg(debug_assertions)]
            {
                println!(
                    "== Executing {:#04x} at {:#06x} ==",
                    op_code as u8,
                    self.pc - 1
                );
            }
            self.execute(op_code.into());
            #[cfg(debug_assertions)]
            {
                println!("== Done ==\n");
            }
        }
    }

    fn execute(&mut self, op_code: opcodes::OpCode) {
        match op_code {
            OpCode::Nop => {}
            OpCode::Brk => {
                self.set_flag(BREAK_FLAG);
                self.stack_push((self.pc >> 8) as u8);
                self.stack_push(self.pc as u8);
                self.stack_push(self.ps);

                self.pc = self.mem.borrow().get_interrupt_vector();
            }
            OpCode::Rti => {
                self.ps = self.stack_pop();
                self.pc = self.stack_pop() as u16;
                self.pc |= (self.stack_pop() as u16) << 8;
            }
            OpCode::Jmp => {
                let address: u16 = self.fetch_word();
                self.pc = address;
            }
            OpCode::JmpI => {
                let address: u16 = self.fetch_word();
                let address: u16 = self.read_word(address);
                self.pc = address;
            }
            OpCode::Jsr => {
                let address = self.fetch_word();
                self.stack_push((self.pc >> 8) as u8);
                self.stack_push(self.pc as u8);
                self.pc = address;
            }
            OpCode::Rts => {
                self.pc = self.stack_pop() as u16;
                self.pc |= (self.stack_pop() as u16) << 8;
            }
            OpCode::Clc => {
                self.reset_flag(CARRY_FLAG);
            }
            OpCode::Cld => {
                self.reset_flag(DECIMAL_MODE_FLAG);
            }
            OpCode::Cli => {
                self.reset_flag(INTERRUPT_DISABLE_FLAG);
            }
            OpCode::Clv => {
                self.reset_flag(OVERFLOW_FLAG);
            }
            OpCode::Sec => {
                self.set_flag(CARRY_FLAG);
            }
            OpCode::Sed => {
                self.set_flag(DECIMAL_MODE_FLAG);
            }
            OpCode::Sei => {
                self.set_flag(INTERRUPT_DISABLE_FLAG);
            }
            OpCode::LdaI => {
                self.a = self.fetch();
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::LdaZp => {
                let address: u8 = self.fetch();
                self.a = self.mem.borrow().read(address as u16);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::LdaZpX => {
                let address: u8 = self.fetch();
                self.a = self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::LdaA => {
                let address: u16 = self.fetch_word();
                self.a = self.mem.borrow().read(address);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::LdaAX => {
                let address: u16 = self.fetch_word();
                self.a = self.mem.borrow().read(address.wrapping_add(self.x as u16));
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::LdaAY => {
                let address = self.fetch_word();
                self.a = self.mem.borrow().read(address.wrapping_add(self.y as u16));
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::LdaIX => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.x) as u16);
                self.a = self.mem.borrow().read(address);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::LdaIY => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.y) as u16);
                self.a = self.mem.borrow().read(address);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::LdxI => {
                self.x = self.fetch();
                self.update_zero_flag(self.x);
                self.update_negative_flag(self.x);
            }
            OpCode::LdxZp => {
                let address: u8 = self.fetch();
                self.x = self.mem.borrow().read(address as u16);
                self.update_zero_flag(self.x);
                self.update_negative_flag(self.x);
            }
            OpCode::LdxZpY => {
                let address: u8 = self.fetch();
                self.x = self.mem.borrow().read(address.wrapping_add(self.y) as u16);
                self.update_zero_flag(self.x);
                self.update_negative_flag(self.x);
            }
            OpCode::LdxA => {
                let address = self.fetch_word();
                self.x = self.mem.borrow().read(address);
                self.update_zero_flag(self.x);
                self.update_negative_flag(self.x);
            }
            OpCode::LdxAY => {
                let address: u16 = self.fetch_word();
                self.x = self.mem.borrow().read(address.wrapping_add(self.y as u16));
                self.update_zero_flag(self.x);
                self.update_negative_flag(self.x);
            }
            OpCode::LdyI => {
                self.y = self.fetch();
                self.update_zero_flag(self.y);
                self.update_negative_flag(self.y);
            }
            OpCode::LdyZp => {
                let address: u8 = self.fetch();
                self.y = self.mem.borrow().read(address as u16);
                self.update_zero_flag(self.y);
                self.update_negative_flag(self.y);
            }
            OpCode::LdyZpX => {
                let address: u8 = self.fetch();
                self.y = self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                self.update_zero_flag(self.y);
                self.update_negative_flag(self.y);
            }
            OpCode::LdyA => {
                let address = self.fetch_word();
                self.y = self.mem.borrow().read(address);
                self.update_zero_flag(self.y);
                self.update_negative_flag(self.y);
            }
            OpCode::LdyAX => {
                let address: u16 = self.fetch_word();
                self.y = self.mem.borrow().read(address.wrapping_add(self.x as u16));
                self.update_zero_flag(self.y);
                self.update_negative_flag(self.y);
            }
            OpCode::StaZp => {
                let address: u8 = self.fetch();
                self.mem.borrow_mut().write(address as u16, self.a);
            }
            OpCode::StaZpX => {
                let address: u8 = self.fetch();
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.x) as u16, self.a);
            }
            OpCode::StaA => {
                let address: u16 = self.fetch_word();
                self.mem.borrow_mut().write(address, self.a);
            }
            OpCode::StaAX => {
                let address: u16 = self.fetch_word();
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.x as u16), self.a);
            }
            OpCode::StaAY => {
                let address: u16 = self.fetch_word();
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.y as u16), self.a);
            }
            OpCode::StaIX => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.x) as u16);
                self.mem.borrow_mut().write(address, self.a);
            }
            OpCode::StaIY => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.y) as u16);
                self.mem.borrow_mut().write(address, self.a);
            }
            OpCode::StxZp => {
                let address: u8 = self.fetch();
                self.mem.borrow_mut().write(address as u16, self.x);
            }
            OpCode::StxZpY => {
                let address: u8 = self.fetch();
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.y) as u16, self.x);
            }
            OpCode::StxA => {
                let address: u16 = self.fetch_word();
                self.mem.borrow_mut().write(address, self.x);
            }
            OpCode::StyZp => {
                let address: u8 = self.fetch();
                self.mem.borrow_mut().write(address as u16, self.y);
            }
            OpCode::StyZpX => {
                let address: u8 = self.fetch();
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.x) as u16, self.y);
            }
            OpCode::StyA => {
                let address: u16 = self.fetch_word();
                self.mem.borrow_mut().write(address, self.y);
            }
            OpCode::IncZp => {
                let address: u8 = self.fetch();
                let mut value: u8 = self.mem.borrow().read(address as u16);
                value = value.wrapping_add(0x01);
                self.mem.borrow_mut().write(address as u16, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::IncZpX => {
                let address: u8 = self.fetch();
                let mut value: u8 = self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                value = value.wrapping_add(0x01);
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.x) as u16, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::IncA => {
                let address: u16 = self.fetch_word();
                let mut value: u8 = self.mem.borrow().read(address);
                value = value.wrapping_add(0x01);
                self.mem.borrow_mut().write(address, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::IncAX => {
                let address: u16 = self.fetch_word();
                let mut value: u8 = self.mem.borrow().read(address.wrapping_add(self.x as u16));
                value = value.wrapping_add(0x01);
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.x as u16), value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::DecZp => {
                let address: u8 = self.fetch();
                let mut value: u8 = self.mem.borrow().read(address as u16);
                value = value.wrapping_sub(0x01);
                self.mem.borrow_mut().write(address as u16, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::DecZpX => {
                let address: u8 = self.fetch();
                let mut value: u8 = self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                value = value.wrapping_sub(0x01);
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.x) as u16, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::DecA => {
                let address: u16 = self.fetch_word();
                let mut value: u8 = self.mem.borrow().read(address);
                value = value.wrapping_sub(0x01);
                self.mem.borrow_mut().write(address, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::DecAX => {
                let address: u16 = self.fetch_word();
                let mut value: u8 = self.mem.borrow().read(address.wrapping_add(self.x as u16));
                value = value.wrapping_sub(0x01);
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.x as u16), value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::Inx => {
                self.x = self.x.wrapping_add(0x01);
                self.update_zero_flag(self.x);
                self.update_negative_flag(self.x);
            }
            OpCode::Iny => {
                self.y = self.y.wrapping_add(0x01);
                self.update_zero_flag(self.y);
                self.update_negative_flag(self.y);
            }
            OpCode::Dex => {
                self.x = self.x.wrapping_sub(0x01);
                self.update_zero_flag(self.x);
                self.update_negative_flag(self.x);
            }
            OpCode::Dey => {
                self.y = self.y.wrapping_sub(0x01);
                self.update_zero_flag(self.y);
                self.update_negative_flag(self.y);
            }
            OpCode::Pha => {
                self.pc += 0x01;
                self.stack_push(self.a);
            }
            OpCode::Php => {
                self.stack_push(self.ps);
            }
            OpCode::Pla => {
                self.a = self.stack_pop();
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::Plp => {
                self.ps = self.stack_pop();
            }
            OpCode::Tax => {
                self.x = self.a;
                self.update_zero_flag(self.x);
                self.update_negative_flag(self.x);
            }
            OpCode::Tay => {
                self.y = self.a;
                self.update_zero_flag(self.y);
                self.update_negative_flag(self.y);
            }
            OpCode::Tsx => {
                self.x = self.sp;
                self.update_zero_flag(self.x);
                self.update_negative_flag(self.x);
            }
            OpCode::Txa => {
                self.a = self.x;
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::Txs => {
                self.sp = self.x;
            }
            OpCode::Tya => {
                self.a = self.y;
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::Bcc => {
                let offset: u8 = self.fetch();
                if self.get_flag(CARRY_FLAG) == 0 {
                    let offset = if offset & 0x80 != 0 {
                        // If the offset is negative, extend the sign bit to 16 bits
                        (offset as u16) | 0xff00
                    } else {
                        // If the offset is positive, just cast it to u16
                        offset as u16
                    };
                    self.pc = self.pc.wrapping_add(offset);
                }
            }
            OpCode::Bcs => {
                let offset: u8 = self.fetch();
                if self.get_flag(CARRY_FLAG) != 0 {
                    let offset = if offset & 0x80 != 0 {
                        // If the offset is negative, extend the sign bit to 16 bits
                        (offset as u16) | 0xff00
                    } else {
                        // If the offset is positive, just cast it to u16
                        offset as u16
                    };
                    self.pc = self.pc.wrapping_add(offset);
                }
            }
            OpCode::Beq => {
                let offset: u8 = self.fetch();
                if self.get_flag(ZERO_FLAG) != 0 {
                    let offset = if offset & 0x80 != 0 {
                        // If the offset is negative, extend the sign bit to 16 bits
                        (offset as u16) | 0xff00
                    } else {
                        // If the offset is positive, just cast it to u16
                        offset as u16
                    };
                    self.pc = self.pc.wrapping_add(offset);
                }
            }
            OpCode::Bmi => {
                let offset: u8 = self.fetch();
                if self.get_flag(NEGATIVE_FLAG) != 0 {
                    let offset = if offset & 0x80 != 0 {
                        // If the offset is negative, extend the sign bit to 16 bits
                        (offset as u16) | 0xff00
                    } else {
                        // If the offset is positive, just cast it to u16
                        offset as u16
                    };
                    self.pc = self.pc.wrapping_add(offset);
                }
            }
            OpCode::Bne => {
                let offset: u8 = self.fetch();
                if self.get_flag(ZERO_FLAG) == 0 {
                    let offset = if offset & 0x80 != 0 {
                        // If the offset is negative, extend the sign bit to 16 bits
                        (offset as u16) | 0xff00
                    } else {
                        // If the offset is positive, just cast it to u16
                        offset as u16
                    };
                    self.pc = self.pc.wrapping_add(offset);
                }
            }
            OpCode::Bpl => {
                let offset: u8 = self.fetch();
                if self.get_flag(NEGATIVE_FLAG) == 0 {
                    let offset = if offset & 0x80 != 0 {
                        // If the offset is negative, extend the sign bit to 16 bits
                        (offset as u16) | 0xff00
                    } else {
                        // If the offset is positive, just cast it to u16
                        offset as u16
                    };
                    self.pc = self.pc.wrapping_add(offset);
                }
            }
            OpCode::Bvc => {
                let offset: u8 = self.fetch();
                if self.get_flag(OVERFLOW_FLAG) == 0 {
                    let offset = if offset & 0x80 != 0 {
                        // If the offset is negative, extend the sign bit to 16 bits
                        (offset as u16) | 0xff00
                    } else {
                        // If the offset is positive, just cast it to u16
                        offset as u16
                    };
                    self.pc = self.pc.wrapping_add(offset);
                }
            }
            OpCode::Bvs => {
                let offset: u8 = self.fetch();
                if self.get_flag(OVERFLOW_FLAG) != 0 {
                    let offset = if offset & 0x80 != 0 {
                        // If the offset is negative, extend the sign bit to 16 bits
                        (offset as u16) | 0xff00
                    } else {
                        // If the offset is positive, just cast it to u16
                        offset as u16
                    };
                    self.pc = self.pc.wrapping_add(offset);
                }
            }
            OpCode::AdcI => {
                let value: u8 = self.fetch();
                self.adc(value);
            }
            OpCode::AdcZp => {
                let address: u8 = self.fetch();
                let value: u8 = self.mem.borrow().read(address as u16);
                self.adc(value);
            }
            OpCode::AdcZpX => {
                let address: u8 = self.fetch();
                let value: u8 = self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                self.adc(value);
            }
            OpCode::AdcA => {
                let address: u16 = self.fetch_word();
                let value: u8 = self.mem.borrow().read(address);
                self.adc(value);
            }
            OpCode::AdcAX => {
                let address: u16 = self.fetch_word();
                let value: u8 = self.mem.borrow().read(address.wrapping_add(self.x as u16));
                self.adc(value);
            }
            OpCode::AdcAY => {
                let address: u16 = self.fetch_word();
                let value: u8 = self.mem.borrow().read(address.wrapping_add(self.y as u16));
                self.adc(value);
            }
            OpCode::AdcIX => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.x) as u16);
                let value: u8 = self.mem.borrow().read(address);
                self.adc(value);
            }
            OpCode::AdcIY => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.y) as u16);
                let value: u8 = self.mem.borrow().read(address);
                self.adc(value);
            }
            OpCode::SbcI => {
                let value: u8 = self.fetch();
                self.sbc(value);
            }
            OpCode::SbcZp => {
                let address: u8 = self.fetch();
                let value: u8 = self.mem.borrow().read(address as u16);
                self.sbc(value);
            }
            OpCode::SbcZpX => {
                let address: u8 = self.fetch();
                let value: u8 = self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                self.sbc(value);
            }
            OpCode::SbcA => {
                let address: u16 = self.fetch_word();
                let value: u8 = self.mem.borrow().read(address);
                self.sbc(value);
            }
            OpCode::SbcAX => {
                let address: u16 = self.fetch_word();
                let value: u8 = self.mem.borrow().read(address.wrapping_add(self.x as u16));
                self.sbc(value);
            }
            OpCode::SbcAY => {
                let address: u16 = self.fetch_word();
                let value: u8 = self.mem.borrow().read(address.wrapping_add(self.y as u16));
                self.sbc(value);
            }
            OpCode::SbcIX => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.x) as u16);
                let value: u8 = self.mem.borrow().read(address);
                self.sbc(value);
            }
            OpCode::SbcIY => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.y) as u16);
                let value: u8 = self.mem.borrow().read(address);
                self.sbc(value);
            }
            OpCode::AndI => {
                self.a &= self.fetch();
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::AndZp => {
                let address: u8 = self.fetch();
                self.a &= self.mem.borrow().read(address as u16);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::AndZpX => {
                let address: u8 = self.fetch();
                self.a &= self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::AndA => {
                let address: u16 = self.fetch_word();
                self.a &= self.mem.borrow().read(address);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::AndAX => {
                let address: u16 = self.fetch_word();
                self.a &= self.mem.borrow().read(address.wrapping_add(self.x as u16));
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::AndAY => {
                let address: u16 = self.fetch_word();
                self.a &= self.mem.borrow().read(address.wrapping_add(self.y as u16));
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::AndIX => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.x) as u16);
                self.a &= self.mem.borrow().read(address);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::AndIY => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.y) as u16);
                self.a &= self.mem.borrow().read(address);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::BitZp => {
                let address: u8 = self.fetch();
                let value: u8 = self.mem.borrow().read(address as u16);
                self.update_zero_flag(self.a & value);
                self.update_negative_flag(value);
                self.update_overflow_flag(value);
            }
            OpCode::BitA => {
                let address: u16 = self.fetch_word();
                let value: u8 = self.mem.borrow().read(address);
                self.update_zero_flag(self.a & value);
                self.update_negative_flag(value);
                self.update_overflow_flag(value);
            }
            OpCode::EorI => {
                self.a ^= self.fetch();
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::EorZp => {
                let address: u8 = self.fetch();
                self.a ^= self.mem.borrow().read(address as u16);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::EorZpX => {
                let address: u8 = self.fetch();
                self.a ^= self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::EorA => {
                let address: u16 = self.fetch_word();
                self.a ^= self.mem.borrow().read(address);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::EorAX => {
                let address: u16 = self.fetch_word();
                self.a ^= self.mem.borrow().read(address.wrapping_add(self.x as u16));
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::EorAY => {
                let address: u16 = self.fetch_word();
                self.a ^= self.mem.borrow().read(address.wrapping_add(self.y as u16));
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::EorIX => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.x) as u16);
                self.a ^= self.mem.borrow().read(address);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::EorIY => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.y) as u16);
                self.a ^= self.mem.borrow().read(address);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::AslA => {
                self.update_carry_flag(self.a);
                self.a = self.a << 1;
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::AslZp => {
                let address: u8 = self.fetch();
                let mut value: u8 = self.mem.borrow().read(address as u16);
                self.update_carry_flag(value);
                value = value << 1;
                self.mem.borrow_mut().write(address as u16, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::AslZpX => {
                let address: u8 = self.fetch();
                let mut value: u8 = self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                self.update_carry_flag(value);
                value = value << 1;
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.x) as u16, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::AslAbs => {
                let address: u16 = self.fetch_word();
                let mut value: u8 = self.mem.borrow().read(address);
                self.update_carry_flag(value);
                value = value << 1;
                self.mem.borrow_mut().write(address, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::AslAbsX => {
                let address: u16 = self.fetch_word();
                let mut value: u8 = self.mem.borrow().read(address);
                self.update_carry_flag(value);
                value = value << 1;
                self.mem.borrow_mut().write(address, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::LsrA => {
                self.set_flag_to(CARRY_FLAG, self.a & 0b0000_0001);
                self.a = self.a >> 1;
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::LsrZp => {
                let address: u8 = self.fetch();
                let mut value: u8 = self.mem.borrow().read(address as u16);
                self.set_flag_to(CARRY_FLAG, value & 0b0000_0001);
                value = value >> 1;
                self.mem.borrow_mut().write(address as u16, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::LsrZpX => {
                let address: u8 = self.fetch();
                let mut value: u8 = self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                self.set_flag_to(CARRY_FLAG, value & 0b0000_0001);
                value = value >> 1;
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.x) as u16, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::LsrAbs => {
                let address: u16 = self.fetch_word();
                let mut value: u8 = self.mem.borrow().read(address);
                self.set_flag_to(CARRY_FLAG, value & 0b0000_0001);
                value = value >> 1;
                self.mem.borrow_mut().write(address, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::LsrAbsX => {
                let address: u16 = self.fetch_word();
                let mut value: u8 = self.mem.borrow().read(address);
                self.set_flag_to(CARRY_FLAG, value & 0b0000_0001);
                value = value >> 1;
                self.mem.borrow_mut().write(address, value);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
            }
            OpCode::RolA => {
                let mut value: u8 = self.a;
                let bit: u8 = (value & 0b1000_0000) >> 7;
                // Sets bit 0 to the carry flag (works because Carry Flag is 0x1), another value would set another bit
                value = (value << 1) | self.get_flag(CARRY_FLAG);
                self.set_flag_to(CARRY_FLAG, bit);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
                self.a = value;
            }
            OpCode::RolZp => {
                let address: u8 = self.fetch();
                let mut value: u8 = self.mem.borrow().read(address as u16);
                let bit: u8 = (value & 0b1000_0000) >> 7;
                value = (value << 1) | self.get_flag(CARRY_FLAG);
                self.set_flag_to(CARRY_FLAG, bit);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
                self.mem.borrow_mut().write(address as u16, value);
            }
            OpCode::RolZpX => {
                let address: u8 = self.fetch();
                let mut value: u8 = self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                let bit: u8 = (value & 0b1000_0000) >> 7;
                value = (value << 1) | self.get_flag(CARRY_FLAG);
                self.set_flag_to(CARRY_FLAG, bit);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.x) as u16, value);
            }
            OpCode::RolAbs => {
                let address: u16 = self.fetch_word();
                let mut value: u8 = self.mem.borrow().read(address);
                let bit: u8 = (value & 0b1000_0000) >> 7;
                value = (value << 1) | self.get_flag(CARRY_FLAG);
                self.set_flag_to(CARRY_FLAG, bit);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
                self.mem.borrow_mut().write(address, value);
            }
            OpCode::RolAbsX => {
                let address: u16 = self.fetch_word();
                let mut value: u8 = self.mem.borrow().read(address);
                let bit: u8 = (value & 0b1000_0000) >> 7;
                value = (value << 1) | self.get_flag(CARRY_FLAG);
                self.set_flag_to(CARRY_FLAG, bit);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
                self.mem.borrow_mut().write(address, value);
            }
            OpCode::RorA => {
                let mut value: u8 = self.a;
                let bit: u8 = value & 0b0000_0001;
                value = (value >> 1) | (self.get_flag(CARRY_FLAG) << 7);
                self.set_flag_to(CARRY_FLAG, bit);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
                self.a = value;
            }
            OpCode::RorZp => {
                let address: u8 = self.fetch();
                let mut value: u8 = self.mem.borrow().read(address as u16);
                let bit: u8 = value & 0b0000_0001;
                value = (value >> 1) | (self.get_flag(CARRY_FLAG) << 7);
                self.set_flag_to(CARRY_FLAG, bit);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
                self.mem.borrow_mut().write(address as u16, value);
            }
            OpCode::RorZpX => {
                let address: u8 = self.fetch();
                let mut value: u8 = self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                let bit: u8 = value & 0b0000_0001;
                value = (value >> 1) | (self.get_flag(CARRY_FLAG) << 7);
                self.set_flag_to(CARRY_FLAG, bit);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
                self.mem
                    .borrow_mut()
                    .write(address.wrapping_add(self.x) as u16, value);
            }
            OpCode::RorAbs => {
                let address: u16 = self.fetch_word();
                let mut value: u8 = self.mem.borrow().read(address);
                let bit: u8 = value & 0b0000_0001;
                value = (value >> 1) | (self.get_flag(CARRY_FLAG) << 7);
                self.set_flag_to(CARRY_FLAG, bit);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
                self.mem.borrow_mut().write(address, value);
            }
            OpCode::RorAbsX => {
                let address: u16 = self.fetch_word();
                let mut value: u8 = self.mem.borrow().read(address);
                let bit: u8 = value & 0b0000_0001;
                value = (value >> 1) | (self.get_flag(CARRY_FLAG) << 7);
                self.set_flag_to(CARRY_FLAG, bit);
                self.update_zero_flag(value);
                self.update_negative_flag(value);
                self.mem.borrow_mut().write(address, value);
            }
            OpCode::OraI => {
                self.a |= self.fetch();
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::OraZp => {
                let address: u8 = self.fetch();
                self.a |= self.mem.borrow().read(address as u16);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::OraZpX => {
                let address: u8 = self.fetch();
                self.a |= self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::OraA => {
                let address: u16 = self.fetch_word();
                self.a |= self.mem.borrow().read(address);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::OraAX => {
                let address: u16 = self.fetch_word();
                self.a |= self.mem.borrow().read(address.wrapping_add(self.x as u16));
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::OraAY => {
                let address: u16 = self.fetch_word();
                self.a |= self.mem.borrow().read(address.wrapping_add(self.y as u16));
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::OraIX => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.x) as u16);
                self.a |= self.mem.borrow().read(address);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::OraIY => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.y) as u16);
                self.a |= self.mem.borrow().read(address);
                self.update_zero_flag(self.a);
                self.update_negative_flag(self.a);
            }
            OpCode::CmpI => {
                let value: u8 = self.fetch();
                self.update_carry_flag(self.a.wrapping_sub(value));
                self.update_zero_flag(self.a.wrapping_sub(value));
                self.update_negative_flag(self.a.wrapping_sub(value));
            }
            OpCode::CmpZp => {
                let address: u8 = self.fetch();
                let value: u8 = self.mem.borrow().read(address as u16);
                self.update_carry_flag(self.a.wrapping_sub(value));
                self.update_zero_flag(self.a.wrapping_sub(value));
                self.update_negative_flag(self.a.wrapping_sub(value));
            }
            OpCode::CmpZpX => {
                let address: u8 = self.fetch();
                let value: u8 = self.mem.borrow().read(address.wrapping_add(self.x) as u16);
                self.update_carry_flag(self.a.wrapping_sub(value));
                self.update_zero_flag(self.a.wrapping_sub(value));
                self.update_negative_flag(self.a.wrapping_sub(value));
            }
            OpCode::CmpA => {
                let address: u16 = self.fetch_word();
                let value: u8 = self.mem.borrow().read(address);
                self.update_carry_flag(self.a.wrapping_sub(value));
                self.update_zero_flag(self.a.wrapping_sub(value));
                self.update_negative_flag(self.a.wrapping_sub(value));
            }
            OpCode::CmpAX => {
                let address: u16 = self.fetch_word();
                let value: u8 = self.mem.borrow().read(address.wrapping_add(self.x as u16));
                self.update_carry_flag(self.a.wrapping_sub(value));
                self.update_zero_flag(self.a.wrapping_sub(value));
                self.update_negative_flag(self.a.wrapping_sub(value));
            }
            OpCode::CmpAY => {
                let address: u16 = self.fetch_word();
                let value: u8 = self.mem.borrow().read(address.wrapping_add(self.y as u16));
                self.update_carry_flag(self.a.wrapping_sub(value));
                self.update_zero_flag(self.a.wrapping_sub(value));
                self.update_negative_flag(self.a.wrapping_sub(value));
            }
            OpCode::CmpIX => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.x) as u16);
                let value: u8 = self.mem.borrow().read(address);
                self.update_carry_flag(self.a.wrapping_sub(value));
                self.update_zero_flag(self.a.wrapping_sub(value));
                self.update_negative_flag(self.a.wrapping_sub(value));
            }
            OpCode::CmpIY => {
                let address: u8 = self.fetch();
                let address: u16 = self.read_word(address.wrapping_add(self.y) as u16);
                let value: u8 = self.mem.borrow().read(address);
                self.update_carry_flag(self.a.wrapping_sub(value));
                self.update_zero_flag(self.a.wrapping_sub(value));
                self.update_negative_flag(self.a.wrapping_sub(value));
            }
            OpCode::CpxI => {
                let value: u8 = self.fetch();
                self.update_carry_flag(self.x.wrapping_sub(value));
                self.update_zero_flag(self.x.wrapping_sub(value));
                self.update_negative_flag(self.x.wrapping_sub(value));
            }
            OpCode::CpxZp => {
                let address: u8 = self.fetch();
                let value: u8 = self.mem.borrow().read(address as u16);
                self.update_carry_flag(self.x.wrapping_sub(value));
                self.update_zero_flag(self.x.wrapping_sub(value));
                self.update_negative_flag(self.x.wrapping_sub(value));
            }
            OpCode::CpxA => {
                let address: u16 = self.fetch_word();
                let value: u8 = self.mem.borrow().read(address);
                self.update_carry_flag(self.x.wrapping_sub(value));
                self.update_zero_flag(self.x.wrapping_sub(value));
                self.update_negative_flag(self.x.wrapping_sub(value));
            }
            OpCode::CpyI => {
                let value: u8 = self.fetch();
                self.update_carry_flag(self.y.wrapping_sub(value));
                self.update_zero_flag(self.y.wrapping_sub(value));
                self.update_negative_flag(self.y.wrapping_sub(value));
            }
            OpCode::CpyZp => {
                let address: u8 = self.fetch();
                let value: u8 = self.mem.borrow().read(address as u16);
                self.update_carry_flag(self.y.wrapping_sub(value));
                self.update_zero_flag(self.y.wrapping_sub(value));
                self.update_negative_flag(self.y.wrapping_sub(value));
            }
            OpCode::CpyA => {
                let address: u16 = self.fetch_word();
                let value: u8 = self.mem.borrow().read(address);
                self.update_carry_flag(self.y.wrapping_sub(value));
                self.update_zero_flag(self.y.wrapping_sub(value));
                self.update_negative_flag(self.y.wrapping_sub(value));
            }
        }
    }

    /// # Returns
    /// The instruction located at the current address stored in the PC register.
    /// PC is incremented by 1.
    fn fetch(&mut self) -> u8 {
        let value: u8 = self.mem.borrow().read(self.pc);
        self.pc += 0x01;
        value
    }

    fn fetch_word(&mut self) -> u16 {
        let low_byte: u8 = self.mem.borrow().read(self.pc);
        let high_byte: u8 = self.mem.borrow().read(self.pc.wrapping_add(0x01));
        let address: u16 = (high_byte as u16) << 8 | (low_byte as u16);
        self.pc += 0x02;
        address
    }

    fn read_word(&self, address: u16) -> u16 {
        let low_byte: u8 = self.mem.borrow().read(address);
        let high_byte: u8 = self.mem.borrow().read(address.wrapping_add(0x01));
        (high_byte as u16) << 8 | (low_byte as u16)
    }

    fn stack_push(&mut self, value: u8) {
        self.mem.borrow_mut().write(0x0100 + self.sp as u16, value);
        self.sp -= 1;
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp += 1;
        self.mem.borrow().read(0x0100 + self.sp as u16)
    }

    fn update_zero_flag(&mut self, value: u8) {
        if value == 0x00 {
            self.set_flag(ZERO_FLAG);
        } else {
            self.reset_flag(ZERO_FLAG);
        }
    }

    fn update_negative_flag(&mut self, value: u8) {
        if value & 0x80 == 0x80 {
            self.set_flag(NEGATIVE_FLAG);
        } else {
            self.reset_flag(NEGATIVE_FLAG);
        }
    }

    fn update_carry_flag(&mut self, value: u8) {
        if value & 0x80 == 0x80 {
            self.set_flag(CARRY_FLAG);
        } else {
            self.reset_flag(CARRY_FLAG);
        }
    }

    fn update_overflow_flag(&mut self, value: u8) {
        if value & 0x40 == 0x40 {
            self.set_flag(OVERFLOW_FLAG);
        } else {
            self.reset_flag(OVERFLOW_FLAG);
        }
    }

    fn adc(&mut self, value: u8) {
        let result: u8 = self
            .a
            .wrapping_add(value)
            .wrapping_add(self.get_flag(CARRY_FLAG));
        // Set OVERFLOW_FLAG if the sign of the result is different from the sign of both operands
        if (self.a & 0x80) == 0 && (value & 0x80) == 0 && (result & 0x80) != 0
            || (self.a & 0x80) != 0 && (value & 0x80) != 0 && (result & 0x80) == 0
        {
            self.set_flag(OVERFLOW_FLAG);
        } else {
            self.reset_flag(OVERFLOW_FLAG);
        }
        self.update_zero_flag(result as u8);
        self.update_negative_flag(result as u8);
        self.a = result as u8;
    }

    fn sbc(&mut self, value: u8) {
        let result: u8;
        if self.get_flag(CARRY_FLAG) == 0 {
            result = self.a.wrapping_sub(value).wrapping_sub(1);
        } else {
            result = self.a.wrapping_sub(value);
        }

        // Check for overflow
        if (self.a ^ value) & (self.a ^ result) & 0x80 != 0 {
            self.set_flag(OVERFLOW_FLAG);
        } else {
            self.reset_flag(OVERFLOW_FLAG);
        }

        self.update_zero_flag(result as u8);
        self.update_negative_flag(result as u8);
        self.a = result;
    }

    fn set_flag(&mut self, flag: u8) {
        self.ps |= flag;
    }

    fn set_flag_to(&mut self, flag: u8, value: u8) {
        if value == 0 {
            self.reset_flag(flag);
        } else {
            self.set_flag(flag);
        }
    }

    fn reset_flag(&mut self, flag: u8) {
        self.ps &= !flag;
    }

    /// # Returns
    /// 0 if the flag is not set.
    /// Non 0 if set.
    fn get_flag(&self, flag: u8) -> u8 {
        self.ps & flag
    }

    /// Prints the current state of the CPU to stdout.
    /// This method is only available when the `debug_assertions` feature is enabled.
    #[cfg(debug_assertions)]
    pub fn print_state(&self) {
        println!("== Registers:");
        println!("  A:  {:#04x}", self.a);
        println!("  X:  {:#04x}", self.x);
        println!("  Y:  {:#04x}", self.y);
        println!("  SP: {:#04x}", self.sp);
        println!("  PS: {:#04x}", self.ps);
        println!("  PC: {:#06x}", self.pc);
        println!("== Memory:");
        println!(
            "  {:#06x}: {:#04x}\n",
            0x0000,
            self.mem.borrow().read(0x0000)
        );
    }
}

#[cfg(test)]
mod tests_6510 {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn execute_dex() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu: Mos6510 = Mos6510::new(mem);
        cpu.reset();

        cpu.x = 0x01;
        cpu.execute(OpCode::Dex);

        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.get_flag(ZERO_FLAG), ZERO_FLAG);
        assert_eq!(cpu.get_flag(NEGATIVE_FLAG), 0);
    }

    #[test]
    fn execute_dey() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu: Mos6510 = Mos6510::new(mem);
        cpu.reset();

        cpu.y = 0x01;
        cpu.execute(OpCode::Dey);

        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.get_flag(ZERO_FLAG), ZERO_FLAG);
        assert_eq!(cpu.get_flag(NEGATIVE_FLAG), 0);
    }

    #[test]
    fn execute_bcc() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu: Mos6510 = Mos6510::new(mem);
        cpu.reset();

        cpu.pc = 0x0000;
        cpu.ps = 0x00;
        cpu.mem.borrow_mut().write(0x0000, OpCode::Bcc.into());
        cpu.mem.borrow_mut().write(0x0001, 0x02);
        cpu.step();

        assert_eq!(cpu.pc, 0x0004);
        assert_eq!(cpu.get_flag(CARRY_FLAG), 0);

        cpu.pc = 0x0000;
        cpu.ps = 0x01;
        cpu.mem.borrow_mut().write(0x0000, OpCode::Bcc.into());
        cpu.mem.borrow_mut().write(0x0001, 0x02);
        cpu.step();

        assert_eq!(cpu.pc, 0x0002);
        assert_eq!(cpu.get_flag(CARRY_FLAG), CARRY_FLAG);
    }

    #[test]
    fn execute_beq() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu: Mos6510 = Mos6510::new(mem);
        cpu.reset();

        cpu.ps = ZERO_FLAG;

        cpu.mem.borrow_mut().write(0x0000, OpCode::Beq.into());
        cpu.mem.borrow_mut().write(0x0001, 0x01);

        cpu.step();

        assert_eq!(cpu.pc, 0x0003);
        assert_eq!(cpu.ps, ZERO_FLAG);
    }

    #[test]
    fn execute_beq_negative_offset() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu: Mos6510 = Mos6510::new(mem);
        cpu.reset();

        cpu.ps = ZERO_FLAG;
        cpu.pc = 0x0000;

        cpu.mem.borrow_mut().write(0x0000, OpCode::Nop.into());
        cpu.mem.borrow_mut().write(0x0001, OpCode::Nop.into());
        cpu.mem.borrow_mut().write(0x0002, OpCode::LdaI.into());
        cpu.mem.borrow_mut().write(0x0003, 0x00);
        cpu.mem.borrow_mut().write(0x0004, OpCode::Beq.into());
        cpu.mem.borrow_mut().write(0x0005, 0xFC);

        cpu.step();
        cpu.step();
        cpu.step();
        cpu.step();

        assert_eq!(cpu.pc, 0x0002);
        assert_eq!(cpu.ps, ZERO_FLAG);
    }

    #[test]
    fn execute_bmi() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu: Mos6510 = Mos6510::new(mem);
        cpu.reset();

        cpu.pc = 0x0000;
        cpu.ps = 0x00;
        cpu.mem.borrow_mut().write(0x0000, OpCode::Bmi.into());
        cpu.mem.borrow_mut().write(0x0001, 0x02);
        cpu.step();

        assert_eq!(cpu.pc, 0x0002);
        assert_eq!(cpu.get_flag(NEGATIVE_FLAG), 0);

        cpu.pc = 0x0000;
        cpu.ps = 0x80;
        cpu.mem.borrow_mut().write(0x0000, OpCode::Bmi.into());
        cpu.mem.borrow_mut().write(0x0001, 0x02);
        cpu.step();

        assert_eq!(cpu.pc, 0x0004);
        assert_eq!(cpu.get_flag(NEGATIVE_FLAG), NEGATIVE_FLAG);
    }

    #[test]
    fn execute_ldai() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu: Mos6510 = Mos6510::new(mem);
        cpu.reset();

        cpu.mem.borrow_mut().write(0x0000, OpCode::LdaI.into());
        cpu.mem.borrow_mut().write(0x0001, 0xFA);
        cpu.step();

        assert_eq!(cpu.a, 0xFA);
        assert_eq!(cpu.get_flag(ZERO_FLAG), 0);
        assert_eq!(cpu.get_flag(NEGATIVE_FLAG), NEGATIVE_FLAG);
    }

    #[test]
    fn execute_ldxi() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu: Mos6510 = Mos6510::new(mem);
        cpu.reset();

        cpu.mem.borrow_mut().write(0x0000, OpCode::LdxI.into());
        cpu.mem.borrow_mut().write(0x0001, 0xFA);
        cpu.step();

        assert_eq!(cpu.x, 0xFA);
        assert_eq!(cpu.get_flag(ZERO_FLAG), 0);
        assert_eq!(cpu.get_flag(NEGATIVE_FLAG), NEGATIVE_FLAG);
    }

    #[test]
    fn execute_ldyi() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu: Mos6510 = Mos6510::new(mem);
        cpu.reset();

        cpu.mem.borrow_mut().write(0x0000, OpCode::LdyI.into());
        cpu.mem.borrow_mut().write(0x0001, 0xFA);
        cpu.step();

        assert_eq!(cpu.y, 0xFA);
        assert_eq!(cpu.get_flag(ZERO_FLAG), 0);
    }

    #[test]
    fn execute_rola() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu = Mos6510::new(mem);
        cpu.reset();

        cpu.a = 0;
        cpu.ps = CARRY_FLAG | ZERO_FLAG | NEGATIVE_FLAG;
        cpu.mem.borrow_mut().write(0x0000, OpCode::RolA.into());
        cpu.step();

        assert_eq!(cpu.a, 1);
        assert_eq!(cpu.get_flag(CARRY_FLAG), 0);
        assert_eq!(cpu.get_flag(ZERO_FLAG), 0);
        assert_eq!(cpu.get_flag(NEGATIVE_FLAG), 0);
    }

    #[test]
    fn execute_rolzp() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu = Mos6510::new(mem);
        cpu.reset();

        cpu.ps = CARRY_FLAG | ZERO_FLAG | NEGATIVE_FLAG;
        cpu.mem.borrow_mut().write(0x0000, OpCode::RolZp.into());
        cpu.mem.borrow_mut().write(0x0001, 0x42);
        cpu.mem.borrow_mut().write(0x0042, 0);

        cpu.step();

        assert_eq!(cpu.mem.borrow().read(0x0042), 1);
        assert_eq!(cpu.get_flag(CARRY_FLAG), 0);
        assert_eq!(cpu.get_flag(ZERO_FLAG), 0);
        assert_eq!(cpu.get_flag(NEGATIVE_FLAG), 0);
    }

    #[test]
    fn execute_rolzp2() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu = Mos6510::new(mem);
        cpu.reset();

        cpu.ps = NEGATIVE_FLAG;
        cpu.mem.borrow_mut().write(0x0000, OpCode::RolZp.into());
        cpu.mem.borrow_mut().write(0x0001, 0x42);
        cpu.mem.borrow_mut().write(0x0042, 0x80);

        cpu.step();

        assert_eq!(cpu.mem.borrow().read(0x0042), 0);
        assert_eq!(cpu.get_flag(CARRY_FLAG), CARRY_FLAG);
        assert_eq!(cpu.get_flag(ZERO_FLAG), ZERO_FLAG);
        assert_eq!(cpu.get_flag(NEGATIVE_FLAG), 0);
    }

    #[test]
    fn execute_adc() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu = Mos6510::new(mem);
        cpu.reset();

        cpu.a = 0x01;
        cpu.mem.borrow_mut().write(0x0000, OpCode::AdcI.into());
        cpu.mem.borrow_mut().write(0x0001, 0x01);
        cpu.step();

        assert_eq!(cpu.a, 0x02);
        assert_eq!(cpu.get_flag(CARRY_FLAG), 0);
        assert_eq!(cpu.get_flag(ZERO_FLAG), 0);
        assert_eq!(cpu.get_flag(NEGATIVE_FLAG), 0);
        assert_eq!(cpu.get_flag(OVERFLOW_FLAG), 0);
    }

    #[test]
    fn execute_adc_overflow() {
        let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
        let mut cpu = Mos6510::new(mem);
        cpu.reset();

        cpu.a = 0x7F;
        cpu.mem.borrow_mut().write(0x0000, OpCode::AdcI.into());
        cpu.mem.borrow_mut().write(0x0001, 0x01);
        cpu.step();

        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.get_flag(CARRY_FLAG), 0);
        assert_eq!(cpu.get_flag(ZERO_FLAG), 0);
        assert_eq!(cpu.get_flag(NEGATIVE_FLAG), NEGATIVE_FLAG);
        assert_eq!(cpu.get_flag(OVERFLOW_FLAG), OVERFLOW_FLAG);
    }
}
