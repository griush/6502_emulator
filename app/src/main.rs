use memory::Memory;
use mos6510::Mos6510;
use std::cell::RefCell;
use std::rc::Rc;

use minifb::{Key, Menu, Window, WindowOptions, MENU_KEY_SHIFT};

// Pixel dimensions upscaled to double the original size
// So each C64 pixel is 2x2 pixels on the screen
const WIDTH: usize = 320 * 2;
const HEIGHT: usize = 200 * 2;

const MENU_STEP_ID: usize = 1;
const MENU_EXIT_ID: usize = 2;
const MENU_RESET_ID: usize = 3;
const MENU_HALT_ID: usize = 4;

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

    // Limit to max ~50 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(20_000)));

    // Create menus
    let mut file_menu = Menu::new("File").unwrap();
    file_menu.add_item("Exit", MENU_EXIT_ID).build();

    // Temporary menu for 6510 emulation
    let mut emulation_menu = Menu::new("6510").unwrap();
    emulation_menu
        .add_item("Step", MENU_STEP_ID)
        .shortcut(Key::S, MENU_KEY_SHIFT)
        .build();
    emulation_menu
        .add_item("Reset", MENU_RESET_ID)
        .shortcut(Key::R, MENU_KEY_SHIFT)
        .build();
    emulation_menu
        .add_item("Halt", MENU_HALT_ID)
        .shortcut(Key::H, MENU_KEY_SHIFT)
        .build();

    window.add_menu(&file_menu);
    window.add_menu(&emulation_menu);

    // Initialize memory
    let mem: Rc<RefCell<Memory>> = Rc::new(RefCell::new(Memory::new()));

    // Load ROMs
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        let rom_file_path: String = args[1].clone();
        mem.borrow_mut().load_rom(rom_file_path.as_str(), 0x0000);
    } else {
        // Load C64 ROMs
        // kernal rom
        mem.borrow_mut().load_rom("roms/C64_KERNALS/dolphindos.bin", 0xe000);
    }

    // Initialize CPU
    let mut cpu: Mos6510 = Mos6510::new(mem);
    cpu.reset();
    #[cfg(debug_assertions)]
    {
        cpu.print_state();
    }

    // Emulation loop
    while window.is_open() {
        // Step CPU here, for now, step in menu
        if let Some(menu_id) = window.is_menu_pressed() {
            match menu_id {
                MENU_STEP_ID => {
                    cpu.step();
                    #[cfg(debug_assertions)]
                    {
                        cpu.print_state();
                    }
                }
                MENU_RESET_ID => {
                    cpu.reset();
                    #[cfg(debug_assertions)]
                    {
                        cpu.print_state();
                    }
                }
                MENU_HALT_ID => {
                    cpu.halt_resume();
                    #[cfg(debug_assertions)]
                    {
                        cpu.print_state();
                    }
                }
                MENU_EXIT_ID => break,
                _ => (),
            }
        }

        for i in buffer.iter_mut() {
            *i = 0x007c71da; // write something more funny here!
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
