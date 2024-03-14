use memory::Memory;
use mos6502::Mos6502;

use std::io;
use std::rc::Rc;
use std::{cell::RefCell, process::exit};

fn main() {
    // Initialize memory
    let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));

    // Load ROMs
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        let rom_file_path: String = args[1].clone();
        mem.borrow_mut().load_rom(rom_file_path.as_str(), 0x0000);
    } else {
        println!("No ROM or binary file given. Use `path/to/exe <path/to/rom>`");
        exit(0);
    }

    // Initialize CPU and load created memory
    let mut cpu: Mos6502 = Mos6502::new(mem);
    cpu.reset();
    #[cfg(debug_assertions)]
    {
        cpu.print_state();
    }

    // Emulation loop
    loop {
        println!("Select: ");
        println!("'s': Step");
        println!("'r': Reset");
        println!("'q': Quit");

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                // Assuming the user enters only one character
                if let Some(c) = input.chars().next() {
                    match c {
                        's' => {
                            cpu.step();
                            cpu.print_state();
                        }
                        'r' => {
                            cpu.reset();
                            cpu.print_state();
                        }
                        'q' => exit(0),
                        _ => println!("Invalid option."),
                    }
                } else {
                    println!("No character entered.");
                }
            }
            Err(error) => println!("Error: {}", error),
        }
    }
}
