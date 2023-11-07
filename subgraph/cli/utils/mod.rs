use colored::*;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;

// This function will work onthe working directory
pub fn run_cmd(main_cmd: &str, args: &[&str]) -> bool {
    // Get the current working directory
    let current_dir = env::current_dir().expect("Failed to get current directory");

    // Create a new Command to run
    let mut cmd = Command::new(main_cmd);

    // Add the arguments
    cmd.args(args);

    // Set the directory from where the command wil run
    cmd.current_dir(&current_dir);

    // Tell what to do when try to print the process
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let full_cmd = format!("{} {}", main_cmd, args.join(" "));

    println!("{} {}\n", "Running:".green(), full_cmd.blue());

    // Execute the command
    let mut child = cmd
        .spawn()
        .expect(format!("Failed to run: {}", full_cmd).as_str());

    // Read and print stdout in a separate thread
    let stdout_child = child.stdout.take().expect("Failed to get stdout");
    let stdout_reader = BufReader::new(stdout_child);

    let stdout_handle = thread::spawn({
        move || {
            for line in stdout_reader.lines() {
                if let Ok(line) = line {
                    println!("{}", line);
                }
            }
        }
    });

    // Read and print stderr in the main thread
    let stderr_reader = BufReader::new(child.stderr.take().expect("Failed to get stderr"));
    for line in stderr_reader.lines() {
        if let Ok(line) = line {
            eprintln!("{}", line);
        }
    }

    // Wait for the command to finish and get the exit status
    let status = child
        .wait()
        .expect(format!("Failed to wait: {}", full_cmd).as_str());

    // Wait for the stdout thread to finish
    stdout_handle.join().expect("Failed to join stdout thread");

    if status.success() {
        println!("✅ {} {}\n", full_cmd.blue(), "completed".green());
        return true;
    } else {
        eprintln!(
            "❌ {} {}",
            full_cmd.blue(),
            format!("failed with exit code: {}\n", status.code().unwrap_or(-1)).red()
        );

        return false;
    }
}
