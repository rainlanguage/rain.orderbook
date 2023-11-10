use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread,
};

/// Execute the command with the given arguments.
pub fn run(main_cmd: &str, args: &[&str]) -> anyhow::Result<()> {
    let mut cmd = Command::new(main_cmd);

    cmd.args(args);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let full_cmd = format!("{} {}", main_cmd, args.join(" "));
    println!("Running: {}\n", full_cmd);

    // Execute the command
    let mut child = cmd.spawn()?;

    // Read and print stdout in a separate thread
    let stdout_child = child.stdout.take().expect("Should take stdout from child");
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
    let stderr_reader = BufReader::new(child.stderr.take().expect("Should take stderr from child"));
    for line in stderr_reader.lines() {
        if let Ok(line) = line {
            eprintln!("{}", line);
        }
    }

    // Wait for the command to finish and get the exit status
    let status = child.wait().expect("should wait for the child to exit");

    // Wait for the stdout thread to finish
    stdout_handle.join().expect("should wait for stdout thread");

    if status.success() {
        println!("{} ✅\n", full_cmd);
        Ok(())
    } else {
        eprintln!(
            "❌ {}",
            format!("failed with exit code: {}\n", status.code().unwrap_or(-1)),
        );

        return Err(anyhow::anyhow!("command execution failed"));
    }
}