mod lc3;
use lc3::vm::VM;
use std::env;

fn main() -> Result<(), lc3::vm::VMError> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage ./lc3-vm <object-file>");
        std::process::exit(1);
    }
    let mut vm = VM::new();
    let Some(file_path) = args.get(1) else {
        std::process::exit(1);
    };
    vm.read_image(file_path)?;
    if let Err(e) = vm.run() {
        println!("{e:?}");
        std::process::exit(1);
    };
    Ok(())
}
