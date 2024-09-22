mod compile;
mod interactive;
mod scan;
mod script;
mod vm;

use std::env;

use interactive::interactive;
use rustyline::Result;
use script::run_script;
use vm::vm::Vm;

fn main() -> Result<()> {
    println!("insh v0.5.0");

    let args: Vec<String> = env::args().skip(1).collect();
    let mut scripts = Vec::new();

    for arg in args {
        scripts.push(arg);
    }

    if scripts.is_empty() {
        interactive()?;
    } else {
        let mut vm = Vm::new();
        for script in scripts {
            run_script(&mut vm, script)?;
        }
    }

    Ok(())
}
