use std::{
    env,
    fs::{self, OpenOptions},
};

use rustyline::{error::ReadlineError, DefaultEditor, Result};

use crate::{
    compile::compiler::Compiler,
    scan::scanner::Scanner,
    vm::{chunk::bytecode_chunk::ByteCodeChunk, evaluate::EvaluateContext, vm::Vm},
};

pub fn interactive() -> Result<()> {
    let mut editor = DefaultEditor::new()?;

    #[cfg(feature = "with-file-history")]
    if let Some(dir) = dirs::preference_dir() {
        let path = dir.as_path().join("insh/history");
        if !path.exists() {
            if let Some(p) = path.parent() {
                fs::create_dir_all(p)?
            }
            let _ = OpenOptions::new()
                .create(true)
                .write(true)
                .open(path.as_path());
        }
        let _ = editor.load_history(path.as_path())?;
    }

    let mut vm = Vm::new();

    loop {
        let cwd = env::current_dir()?;
        let line = editor.readline(format!("{} >> ", cwd.display()).as_str());
        match line {
            Ok(line) => {
                let _ = editor.add_history_entry(line.as_str());

                let scanner = Scanner::new(line);
                let chunk = ByteCodeChunk::new();
                let mut compiler = Compiler::new(scanner, chunk);

                match compiler.compile() {
                    Err(e) => {
                        println!("error: {:?}", e)
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
                    Ok(value) => {
                        println!("{}", value);
                    }
                };
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => {}
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }

    #[cfg(feature = "with-file-history")]
    if let Some(dir) = dirs::preference_dir() {
        editor.save_history(dir.as_path().join("insh/history").as_path())?;
    }

    Ok(())
}
