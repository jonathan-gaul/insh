use std::{
    fs::File,
    io::{BufReader, Read},
};

use rustyline::Result;

use crate::{
    compile::compiler::Compiler,
    scan::scanner::Scanner,
    vm::{chunk::bytecode_chunk::ByteCodeChunk, evaluate::EvaluateContext, vm::Vm},
};

pub fn run_script(vm: &mut Vm, name: String) -> Result<()> {
    let file = File::open(name)?;
    let mut reader = BufReader::new(file);

    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    let scanner = Scanner::new(buffer);
    let mut compiler = Compiler::new(scanner, ByteCodeChunk::new());

    match compiler.compile() {
        Err(e) => {
            println!("compile error: {:?}", e)
        }
        Ok(_) => {
            println!("compiled")
        }
    }

    let chunk = compiler.into_chunk();
    match vm.run(chunk, EvaluateContext::None) {
        Err(e) => {
            println!("runtime error: {:?}", e)
        }
        Ok(_) => {
            println!("executed")
        }
    };

    Ok(())
}
