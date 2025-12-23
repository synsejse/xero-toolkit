//! Xero Authentication Client
//!
//! Command-line client for testing the authentication daemon.

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use xero_auth::protocol::{ClientMessage, DaemonMessage};
use xero_auth::shared::{get_socket_path, is_daemon_running};

struct Client {
    stream: UnixStream,
}

impl Client {
    async fn new() -> Result<Self> {
        let socket_path = get_socket_path(None)?;

        use tokio::time::{timeout, Duration};
        let stream = timeout(Duration::from_secs(5), UnixStream::connect(&socket_path))
            .await
            .context("Connection timeout")?
            .context("Failed to connect to daemon")?;

        Ok(Self { stream })
    }

    async fn execute<F, G>(
        &mut self,
        program: &str,
        args: &[&str],
        working_dir: Option<&str>,
        on_output: F,
        on_error: G,
    ) -> Result<i32>
    where
        F: Fn(&str),
        G: Fn(&str),
    {
        let (mut reader, mut writer) = self.stream.split();
        let mut buf_reader = BufReader::new(&mut reader);

        let message = ClientMessage::Execute {
            program: program.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            working_dir: working_dir.map(|s| s.to_string()),
        };

        let request = serde_json::to_string(&message)? + "\n";
        writer.write_all(request.as_bytes()).await?;

        let mut line = String::new();
        let mut exit_code = None;

        loop {
            line.clear();
            let bytes_read = buf_reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                break;
            }

            let response: DaemonMessage = match serde_json::from_str(line.trim()) {
                Ok(msg) => msg,
                Err(e) => {
                    log::warn!("Failed to parse daemon response: {}", e);
                    continue;
                }
            };

            match response {
                DaemonMessage::Output(text) => {
                    on_output(&text);
                }
                DaemonMessage::Error(text) => {
                    on_error(&text);
                }
                DaemonMessage::Completed { exit_code: code } => {
                    exit_code = Some(code);
                    break;
                }
                DaemonMessage::ErrorMessage(msg) => {
                    anyhow::bail!("Daemon error: {}", msg);
                }
                _ => {}
            }
        }

        Ok(exit_code.unwrap_or(-1))
    }
}

#[tokio::main]
async fn main() {
    if !is_daemon_running() {
        eprintln!("Error: xero-auth daemon is not running");
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: xero-auth <program> [args...]");
        std::process::exit(1);
    }

    let program = &args[0];
    let cmd_args: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();

    let mut client = match Client::new().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            std::process::exit(1);
        }
    };

    let exit_code = match client
        .execute(
            program,
            &cmd_args,
            None,
            |line| print!("{}", line),
            |line| eprint!("{}", line),
        )
        .await
    {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Failed to execute command: {}", e);
            std::process::exit(1);
        }
    };

    std::process::exit(exit_code);
}
