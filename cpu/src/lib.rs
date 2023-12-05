pub mod opcodes;

use std::rc::Rc;
use std::cell::RefCell;
use memory::Memory;
use opcodes::OpCode;

const CARRY_FLAG: u8 = 0b0000_0001;
const ZERO_FLAG: u8 = 0b0000_0010;
const INTERRUPT_DISABLE_FLAG: u8 = 0b0000_0100;
const DECIMAL_MODE_FLAG: u8 = 0b0000_1000;
const BREAK_FLAG: u8 = 0b0001_0000;
const OVERFLOW_FLAG: u8 = 0b0100_0000;
const NEGATIVE_FLAG: u8 = 0b1000_0000;

pub struct Cpu {
    a: u8,
    x: u8,
    y: u8,

    sp: u8,
    ps: u8,
    pc: u16,

    mem: Rc<RefCell<Memory>>
}

impl Cpu {
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
        Cpu { 
            a: 0x00,
            x: 0x00,
            y: 0x00,
            sp: 0x00,
            ps: 0x00,
            pc: 0x00,
            mem: mem
        }
    }

    /// Reset the CPU to its initial state. Must be called at least once before calling step().
    pub fn reset(&mut self) {
        self.a = 0x00;
        self.x = 0x00;
        self.y = 0x00;
        self.sp = 0xff;
        self.ps = 0x00;
        self.pc = self.mem.borrow().get_reset_vector();
    }

    pub fn step(&mut self) {
        let op_code: OpCode = self.fetch();
        self.execute(op_code);
    }

    fn fetch(&self) -> OpCode {
        let op_code: u8 = self.mem.borrow().read(self.pc);
        OpCode::from(op_code)
    }

    fn execute(&mut self, op_code: opcodes::OpCode) {
        match op_code {
            OpCode::Nop => {
                self.pc += 0x01;
            },
            OpCode::Brk => {
                self.pc += 0x01;
                self.set_flag(BREAK_FLAG);
                self.stack_push((self.pc >> 8) as u8);
                self.stack_push(self.pc as u8);
                self.stack_push(self.ps);

                self.pc = self.mem.borrow().get_interrupt_vector();
            },
            OpCode::Rti => {
                self.ps = self.stack_pop();
                self.pc = self.stack_pop() as u16;
                self.pc |= (self.stack_pop() as u16) << 8;
            },
            OpCode::Jsr => {
                let low_byte: u8 = self.mem.borrow().read(self.pc + 0x01);
                let high_byte: u8 = self.mem.borrow().read(self.pc + 0x02);
                let address: u16 = (high_byte as u16) << 8 | (low_byte as u16);

                self.pc += 0x02;
                self.stack_push((self.pc >> 8) as u8);
                self.stack_push(self.pc as u8);
                self.pc = address;
            },
            OpCode::Rts => {
                self.pc = self.stack_pop() as u16;
                self.pc |= (self.stack_pop() as u16) << 8;
                self.pc += 0x01;
            },
            OpCode::Clc => {
                self.pc += 0x01;
                self.reset_flag(CARRY_FLAG);
            },
            OpCode::Cld => {
                self.pc += 0x01;
                self.reset_flag(DECIMAL_MODE_FLAG);
            },
            OpCode::Cli => {
                self.pc += 0x01;
                self.reset_flag(INTERRUPT_DISABLE_FLAG);
            },
            OpCode::Clv => {
                self.pc += 0x01;
                self.reset_flag(OVERFLOW_FLAG);
            },
            OpCode::Sec => {
                self.pc += 0x01;
                self.set_flag(CARRY_FLAG);
            },
            OpCode::Sed => {
                self.pc += 0x01;
                self.set_flag(DECIMAL_MODE_FLAG);
            },
            OpCode::Sei => {
                self.pc += 0x01;
                self.set_flag(INTERRUPT_DISABLE_FLAG);
            },
            OpCode::Dex => {
                self.pc += 0x01;
                self.x = self.x.wrapping_sub(0x01);
                self.update_zero_flag(self.x);
                self.update_negative_flag(self.x);
            },
            OpCode::Dey => {
                self.pc += 0x01;
                self.y = self.y.wrapping_sub(0x01);
                self.update_zero_flag(self.y);
                self.update_negative_flag(self.y);
            },
        }
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

    fn set_flag(&mut self, flag: u8) {
        self.ps |= flag;
    }

    fn reset_flag(&mut self, flag: u8) {
        self.ps &= !flag;
    }

    /// Prints the current state of the CPU to stdout.
    /// This method is only available when the `debug_assertions` feature is enabled.
    #[cfg(debug_assertions)]
    pub fn print_state(&self) {
        println!("== Executing instruction at {:#06x} ==", self.pc);
        println!("== Done ==");
        println!("== Registers:");
        println!("\tA: {:#04x}", self.a);
        println!("\tX: {:#04x}", self.x);
        println!("\tY: {:#04x}", self.y);
        println!("\tSP: {:#04x}", self.sp);
        println!("\tPS: {:#04x}", self.ps);
        println!("\tPC: {:#06x}", self.pc);
        println!("");
    }
}
