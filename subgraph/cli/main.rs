use clap::{Args, Parser, Subcommand};
use colored::*;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub subgraph: Subgraph,
}

#[derive(Subcommand)]
pub enum Subgraph {
    #[command(about = "Install dependecies for the rain subgraph")]
    Install,
    #[command(about = "Build the rain subgraph")]
    Build,
    #[command(about = "Test the rain subgraph")]
    Test,
    #[command(about = "Deploy the rain subgraph")]
    Deploy(DeployCommand),
}

#[derive(Args, Debug)]
pub struct DeployCommand {
    /// Endpoint URL where the subgraph will be deployed
    url: String,
    /// Subgraph token to deploy the subgraph
    key: String,
    /// Network that the subgraph will index
    network: String,
    /// Block number where the subgraph will start indexing
    block_number: String,
    /// Contract address that the subgraph will be indexing (Assuming one address)
    address: String,
}

fn main() {
    let args = Cli::parse();
    match args.subgraph {
        Subgraph::Install => {
            // Get the current working directory
            let current_dir = env::current_dir().expect("Failed to get current directory");

            // Create a new Command to run the npm install command
            let mut npm_install = Command::new("npm");
            npm_install.arg("install");
            npm_install.current_dir(&current_dir);
            npm_install.stdout(Stdio::piped());
            npm_install.stderr(Stdio::piped());

            println!("{}", "Running npm install...".green());

            // Execute the npm install command
            let mut child = npm_install.spawn().expect("Failed to start npm install");

            // Read and print stdout in a separate thread
            let stdout_child = child.stdout.take().expect("Failed to get stdout");
            let stdout_reader = BufReader::new(stdout_child);

            let stdout_handle = thread::spawn(move || {
                for line in stdout_reader.lines() {
                    if let Ok(line) = line {
                        println!("npm: {}", line);
                    }
                }
            });

            // Read and print stderr in the main thread
            let stderr_reader = BufReader::new(child.stderr.take().expect("Failed to get stderr"));
            for line in stderr_reader.lines() {
                if let Ok(line) = line {
                    eprintln!("npm error: {}", line);
                }
            }

            // Wait for the command to finish and get the exit status
            let status = child.wait().expect("Failed to wait for npm install");

            // Wait for the stdout thread to finish
            stdout_handle.join().expect("Failed to join stdout thread");

            if status.success() {
                println!("{}", "npm install successful".green());
            } else {
                eprintln!(
                    "{}",
                    format!(
                        "npm install failed with exit code: {}",
                        status.code().unwrap_or(-1)
                    )
                    .red()
                );
            }
        }
        Subgraph::Build => {
            println!("Hello build");
        }
        Subgraph::Test => {
            println!("Hello tests");
        }
        Subgraph::Deploy(args) => {
            println!("Deploy with: {:?}", args);
        }
    }
}
