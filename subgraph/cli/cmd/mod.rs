use anyhow::anyhow;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread,
};
use tracing::{debug, error, info};

/// Execute the command with the given arguments.
pub fn run(main_cmd: &str, args: &[&str]) -> anyhow::Result<()> {
    let mut cmd = Command::new(main_cmd);

    cmd.args(args);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let full_cmd = format!("{} {}", main_cmd, args.join(" "));
    info!("Running: {}", full_cmd);

    // Execute the command
    let mut child = cmd.spawn()?;

    // Read and print stdout in a separate thread
    let stdout_child = child.stdout.take().expect("Should take stdout from child");
    let stdout_reader = BufReader::new(stdout_child);

    let stdout_handle = thread::spawn({
        move || {
            for line in stdout_reader.lines() {
                if let Ok(line) = line {
                    debug!("{}", line);
                }
            }
        }
    });

    // Read and print stderr in the main thread
    let stderr_reader = BufReader::new(child.stderr.take().expect("Should take stderr from child"));
    for line in stderr_reader.lines() {
        match line {
            Ok(data) => {
                debug!("{}", data);
            }
            Err(err) => {
                error!("{}", err.to_string());
            }
        }
    }

    // Wait for the command to finish and get the exit status
    let status = child.wait()?;

    // Wait for the stdout thread to finish
    match stdout_handle.join() {
        Ok(_) => (),
        Err(_) => {
            return Err(anyhow!("failed to wait for stdout thread"));
        }
    }

    if status.success() {
        Ok(())
    } else {
        return Err(anyhow::anyhow!(
            "command execution failed with exit code: {}\n",
            status.code().unwrap_or(-1)
        ));
    }
}
