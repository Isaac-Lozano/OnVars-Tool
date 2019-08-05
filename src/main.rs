use std::io::{self, Write};
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use onvars_tool::SaveStateUnit;
use onvars_tool::sa2_units::{CharacterUnit, CameraUnit, TimeUnit, GravityUnit};
use onvars_tool::process_reader::ProcessHandle;

fn main() {
    println!("OnVar's Tool (version {})", env!("CARGO_PKG_VERSION"));
    let mut process_string = "sonic2app.exe".to_string();
    let handle;
    'process_hook_loop: loop {
        match ProcessHandle::from_name_filter(|n| n == process_string).unwrap() {
            Some(h) => {
                handle = h;
                break 'process_hook_loop;
            }
            None => {
                println!();
                println!("Could not find process \"{}\".", process_string);
                println!("Please enter the name of the SA2 process.");
                print!("Process name: ");
                io::stdout().flush().unwrap();
                let stdin = io::stdin();
                process_string.clear();
                stdin.read_line(&mut process_string).unwrap();
                process_string = process_string.trim().to_string();
            }
        }
    }

    println!();
    println!("Successfully hooked into \"{}\".", process_string);
    println!();
    println!("Press D-pad Left to save a state.");
    println!("Press D-pad Right to load a state.");

    let mut units: Vec<Rc<dyn SaveStateUnit>> = vec![
        Rc::new(CharacterUnit::new()),
        Rc::new(CameraUnit::new()),
        Rc::new(TimeUnit::new()),
        Rc::new(GravityUnit::new()),
    ];

    let mut prev_buttons = 0;
    let mut save_level = 0;
    let mut frame_opt = None;
    let mut save_valid = false;
    let mut prev_game_state = 0;
    loop {
        handle.write_data(0x0174B050, b"\x01\x00\x00\x00").unwrap();
        let buttons = handle.read_u32(0x01A52C4C).unwrap();
        let buttons_pressed = !prev_buttons & buttons;
        prev_buttons = buttons;

        let level = handle.read_u32(0x1934B70).unwrap();

        let game_state = handle.read_u32(0x1934BE0).unwrap();
        if prev_game_state != 0 && game_state == 0 {
            save_valid = false;
            println!("Exited level. Invalidating savestate.")
        }
        prev_game_state = game_state;

        if buttons_pressed & 0x1 != 0 {
            if game_state != 0 {
                save_level = level;
                save_valid = true;
                for unit in units.iter_mut() {
                    match Rc::get_mut(unit).unwrap().save(&handle) {
                        Ok(()) => {}
                        Err(string) => println!("Error: {}", string),
                    }
                }
                println!("Saving state");
            } else {
                println!("Not in level. Cannot save state.")
            }
        }

        if buttons_pressed & 0x2 != 0 {
            if !save_valid {
                println!("Error: savestate not valid")
            } else if level != save_level {
                println!("Error: not the same stage as savestate");
            } else {
                println!("Loading state");
                frame_opt = Some(handle.read_u32(0x0174b03c).unwrap());
                for unit in units.iter() {
                    match unit.load(&handle) {
                        Ok(()) => {}
                        Err(string) => println!("Error: {}", string),
                    }
                }
            }
        }

        // second-frame savestate load for collision stuff
        if let Some(frame) = frame_opt {
            if frame != handle.read_u32(0x0174b03c).unwrap() {
                for unit in units.iter() {
                    match unit.load(&handle) {
                        Ok(()) => {}
                        Err(string) => println!("Error: {}", string),
                    }
                }
                frame_opt = None;
            }
        }

        thread::sleep(Duration::from_millis(10))
    }
}
