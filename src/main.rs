use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::process::{Command, Stdio};
use pty::fork::*;

fn main() {
    let log_file = "bash_session.log";

    // Open log file for appending
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .expect("Failed to open log file");

    // Create a new PTY fork
    let fork = Fork::from_ptmx().expect("Failed to create PTY");
    
    if let Some(mut master) = fork.is_parent().ok() {
        let mut input = String::new();
        let mut buffer = [0u8; 1024];

        loop {
            // Print prompt manually
            print!("$ ");
            io::stdout().flush().unwrap();

            // Read user input
            input.clear();
            io::stdin().read_line(&mut input).expect("Failed to read input");

            let trimmed_input = input.trim();

            if trimmed_input == "exit" || trimmed_input == "quit" {
                break;
            }

            // Write input to PTY (simulating terminal input)
            master.write_all(input.as_bytes()).expect("Failed to write to PTY");

            // Save input to log
            writeln!(log, "$ {}", trimmed_input).expect("Failed to write to log");

            // Read output interactively
            loop {
                let bytes_read = master.read(&mut buffer).unwrap_or(0);
                if bytes_read == 0 {
                    break;
                }

                let output = String::from_utf8_lossy(&buffer[..bytes_read]);
                print!("{}", output);
                log.write_all(output.as_bytes()).expect("Failed to write output to log");
                
                // Break the loop if we see a shell prompt again
                if output.ends_with("$ ") {
                    break;
                }
            }
        }
    } else {
        // Child process: Replace with Bash
        Command::new("bash")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to start bash")
            .wait()
            .expect("Bash process failed");
    }
}
