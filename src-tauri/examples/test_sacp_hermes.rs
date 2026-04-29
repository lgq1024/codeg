use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() {
    println!("Starting sacp-tokio hermes test...");

    let resolved = which::which("hermes").unwrap_or_else(|_| std::path::PathBuf::from("hermes"));
    println!("Resolved hermes -> {}", resolved.display());

    let agent = sacp_tokio::AcpAgent::from_args(&[
        resolved.to_string_lossy().as_ref(),
        "acp",
    ])
    .unwrap()
    .with_debug(|line, dir| {
        let tag = match dir {
            sacp_tokio::LineDirection::Stdin => "[stdin]",
            sacp_tokio::LineDirection::Stdout => "[stdout]",
            sacp_tokio::LineDirection::Stderr => "[stderr]",
        };
        eprintln!("{tag} {line}");
    });

    let result = timeout(Duration::from_secs(30), sacp::Client
        .builder()
        .name("test-sacp")
        .connect_with(agent, async move |cx| {
            println!("Connected, sending Initialize...");
            let init_req = sacp::schema::InitializeRequest::new(sacp::schema::ProtocolVersion::LATEST)
                .client_capabilities(sacp::schema::ClientCapabilities::new());

            match cx.send_request_to(sacp::Agent, init_req).block_task().await {
                Ok(resp) => {
                    println!("Initialize OK: agent_capabilities.load_session={}", resp.agent_capabilities.load_session);
                }
                Err(e) => {
                    eprintln!("Initialize failed: {e}");
                }
            }
            Ok(())
        })
    ).await;

    match result {
        Ok(Ok(())) => println!("Test completed successfully"),
        Ok(Err(e)) => eprintln!("sacp error: {e}"),
        Err(_) => eprintln!("Test timed out after 30s"),
    }
}
