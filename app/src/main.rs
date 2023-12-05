use std::rc::Rc;
use std::cell::RefCell;
use cpu::Cpu;
use cpu::opcodes::OpCode;
use memory::Memory;

use minifb::{ Window, WindowOptions, Menu };

// Pixel dimensions upscaled to double the original size
// So each C64 pixel is 2x2 pixels on the screen
const WIDTH: usize = 320 * 2;
const HEIGHT: usize = 200 * 2;

const MENU_STEP_ID: usize = 1;

fn main() {

    // Window setup
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Commodore 64 Emulator",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    // Create menu
    let mut menu = Menu::new("Emulation").unwrap();
    menu.add_item("Step", MENU_STEP_ID).build();

    window.add_menu(&menu);

    // Initialize memory
    let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));
    
    // Write some code to memory
    // Will be loading ROMs in the future
    mem.borrow_mut().write(0x0000, OpCode::Sec.into());
    mem.borrow_mut().write(0x0001, OpCode::Sed.into());
    mem.borrow_mut().write(0x0002, OpCode::Sei.into());
    mem.borrow_mut().write(0x0003, OpCode::Clc.into());
    mem.borrow_mut().write(0x0004, OpCode::Cld.into());
    mem.borrow_mut().write(0x0005, OpCode::Cli.into());
    mem.borrow_mut().write(0x0006, OpCode::Clv.into());
    mem.borrow_mut().write(0x0007, OpCode::Dex.into());
    mem.borrow_mut().write(0x0008, OpCode::Dex.into());
    mem.borrow_mut().write(0x0009, OpCode::Dey.into());
    mem.borrow_mut().write(0x000A, OpCode::Nop.into());
    mem.borrow_mut().write(0x000B, OpCode::Nop.into());
    mem.borrow_mut().write(0x000C, OpCode::Brk.into());
    
    // Initialize CPU
    let mut cpu: Cpu = Cpu::new(mem);
    cpu.reset();
    cpu.print_state();

    // Emulation loop
    while window.is_open() {
        // Step CPU here, for now, step in menu
        if let Some(menu_id) = window.is_menu_pressed() {
            match menu_id {
                MENU_STEP_ID => {
                    cpu.step();
                    cpu.print_state();
                },
                _ => (),
            }
        }

        for i in buffer.iter_mut() {
            *i = 0x00ff00ff; // write something more funny here!
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
