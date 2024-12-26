mod lc3;
use lc3::vm::VM;

fn main() -> Result<(), lc3::vm::VMError> {
    let mut vm = VM::new();
    vm.read_image("images/test-image-load-big-endian")?;
    Ok(())
}
