use super::value::Value;
use std::process::Command;

pub fn execute(cmd: String, args: Vec<Value>, capture: bool) -> (i64, String, String) {
    let mut command = Command::new(cmd);

    for arg in args {
        command.arg(arg.to_native_string());
    }

    if capture {
        match command.output() {
            Ok(out) => (
                out.status.code().unwrap_or(0) as i64,
                String::from_utf8_lossy(&out.stdout).to_string(),
                String::from_utf8_lossy(&out.stderr).to_string(),
            ),
            Err(_) => (-1, String::new(), String::new()),
        }
    } else {
        match command.status() {
            Ok(s) => (s.code().unwrap_or(0) as i64, String::new(), String::new()),
            Err(_) => (-1, String::new(), String::new()),
        }
    }
}
