use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::timeout;

#[tokio::main]
async fn main() {
    println!("Starting hermes.bat acp...");

    let resolved = which::which("hermes").unwrap_or_else(|_| std::path::PathBuf::from("hermes"));
    println!("Resolved hermes -> {}", resolved.display());

    let mut cmd = Command::new(&resolved);
    cmd.arg("acp")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(windows)]
    {
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to spawn: {}", e);
            return;
        }
    };

    let pid = child.id().unwrap_or(0);
    println!("Spawned child pid={}", pid);

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Read stdout
    tokio::spawn(async move {
        let reader = tokio::io::BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut count = 0;
        loop {
            match timeout(Duration::from_secs(30), lines.next_line()).await {
                Ok(Ok(Some(line))) => {
                    println!("[STDOUT] {}", line);
                    count += 1;
                    if count >= 5 {
                        break;
                    }
                }
                Ok(Ok(None)) => {
                    println!("[STDOUT] EOF");
                    break;
                }
                Ok(Err(e)) => {
                    eprintln!("[STDOUT] error: {}", e);
                    break;
                }
                Err(_) => {
                    println!("[STDOUT] timeout");
                    break;
                }
            }
        }
        println!("[STDOUT] total lines read: {}", count);
    });

    // Read stderr
    tokio::spawn(async move {
        let reader = tokio::io::BufReader::new(stderr);
        let mut lines = reader.lines();
        loop {
            match timeout(Duration::from_secs(30), lines.next_line()).await {
                Ok(Ok(Some(line))) => {
                    eprintln!("[STDERR] {}", line);
                }
                Ok(Ok(None)) => {
                    eprintln!("[STDERR] EOF");
                    break;
                }
                Ok(Err(e)) => {
                    eprintln!("[STDERR] error: {}", e);
                    break;
                }
                Err(_) => {
                    eprintln!("[STDERR] timeout");
                    break;
                }
            }
        }
    });

    // Send initialize request
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{"experimental":{}},"clientInfo":{"name":"test","version":"0.1.0"}}}"#;
    println!("Sending initialize...");
    if let Err(e) = stdin.write_all(init.as_bytes()).await {
        eprintln!("Failed to write stdin: {}", e);
        return;
    }
    if let Err(e) = stdin.write_all(b"\n").await {
        eprintln!("Failed to write newline: {}", e);
        return;
    }
    if let Err(e) = stdin.flush().await {
        eprintln!("Failed to flush stdin: {}", e);
        return;
    }
    println!("Initialize sent, waiting for response...");

    tokio::time::sleep(Duration::from_secs(10)).await;
    println!("Done. Killing child...");
    let _ = child.kill().await;
}
