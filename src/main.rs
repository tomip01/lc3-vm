mod lc3;
use lc3::vm::VM;
use std::{env, io, os::fd::AsRawFd};
use termios::{Termios, ECHO, ICANON, TCSANOW};

fn set_termios() -> Termios {
    // Get the current terminal settings
    let stdin = io::stdin();
    let mut termios = if let Ok(termios) = Termios::from_fd(stdin.lock().as_raw_fd()) {
        termios
    } else {
        println!("Unable to get termios configuration");
        std::process::exit(1);
    };
    // Save the current terminal settings so we can restore them later
    let original_termios = termios;

    termios.c_lflag &= !(ICANON | ECHO);

    // Apply the new terminal settings immediately
    termios::tcsetattr(stdin.lock().as_raw_fd(), TCSANOW, &termios).unwrap_or_else(|e| {
        println!("Unable to restore configuration, error {e}");
        std::process::exit(1);
    });

    original_termios
}

fn restore_termios(original_termios: Termios) {
    let stdin = io::stdin();
    termios::tcsetattr(stdin.lock().as_raw_fd(), TCSANOW, &original_termios).unwrap_or_else(|e| {
        println!("Unable to restore configuration, error {e}");
        std::process::exit(1);
    });
}

fn main() -> Result<(), lc3::vm::VMError> {
    // config terminal
    let original_termios = set_termios();

    // collect file to execute
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("make run FILEPATH=<path/to/file>");
        std::process::exit(1);
    }
    let Some(file_path) = args.get(1) else {
        std::process::exit(1);
    };

    // create VM
    let mut vm = VM::new();
    // Load program
    vm.read_image(file_path)?;
    // run program
    if let Err(e) = vm.run() {
        println!("{e:?}");
        std::process::exit(1);
    };

    // restore terminal
    restore_termios(original_termios);
    Ok(())
}
