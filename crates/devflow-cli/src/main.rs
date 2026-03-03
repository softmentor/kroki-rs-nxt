use std::env;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();

    let status = Command::new("/usr/local/cargo/bin/dwf")
        .args(args)
        .status();

    match status {
        Ok(status) => {
            if let Some(code) = status.code() {
                ExitCode::from(code as u8)
            } else {
                ExitCode::FAILURE
            }
        }
        Err(err) => {
            eprintln!("failed to exec /usr/local/cargo/bin/dwf: {err}");
            ExitCode::FAILURE
        }
    }
}
