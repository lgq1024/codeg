use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() {
    println!("Starting sacp-tokio hermes FULL test...");

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

    let result = timeout(Duration::from_secs(60), sacp::Client
        .builder()
        .name("test-sacp-full")
        .connect_with(agent, async move |cx| {
            println!("Connected, sending Initialize...");
            let init_req = sacp::schema::InitializeRequest::new(sacp::schema::ProtocolVersion::LATEST)
                .client_capabilities(sacp::schema::ClientCapabilities::new());

            let init_resp = cx.send_request_to(sacp::Agent, init_req).block_task().await?;
            println!("Initialize OK: load_session={}, fork={}",
                init_resp.agent_capabilities.load_session,
                init_resp.agent_capabilities.session_capabilities.fork.is_some()
            );

            println!("Creating session...");
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let new_req = sacp::schema::NewSessionRequest::new(cwd);
            let new_resp = cx.send_request_to(sacp::Agent, new_req).block_task().await?;
            let sid = new_resp.session_id.clone();
            println!("Session created: id={}", sid.0);

            println!("Attaching session...");
            let mut session = cx.attach_session(new_resp, Default::default())?;

            println!("Sending prompt 'hello'...");
            let prompt = sacp::schema::PromptRequest::new(
                sid.clone(),
                vec![sacp::schema::ContentBlock::Text(sacp::schema::TextContent::new("hello"))],
            );
            let mut prompt_fut = Box::pin(cx.clone().send_request_to(sacp::Agent, prompt).block_task());

            let mut update_count = 0;
            loop {
                tokio::select! {
                    update = session.read_update() => {
                        match update {
                            Ok(sacp::SessionMessage::SessionMessage(dispatch)) => {
                                update_count += 1;
                                println!("Update #{update_count}: {:?}", dispatch.method());
                            }
                            Ok(sacp::SessionMessage::StopReason(reason)) => {
                                println!("StopReason: {:?}", reason);
                                break;
                            }
                            Ok(_other) => {
                                println!("Other message received");
                            }
                            Err(e) => {
                                eprintln!("Update error: {e}");
                                break;
                            }
                        }
                    }
                    prompt_result = &mut prompt_fut => {
                        match prompt_result {
                            Ok(resp) => println!("Prompt completed: stop_reason={:?}", resp.stop_reason),
                            Err(e) => eprintln!("Prompt failed: {e}"),
                        }
                        break;
                    }
                }
            }

            println!("Total updates received: {update_count}");
            Ok(())
        })
    ).await;

    match result {
        Ok(Ok(())) => println!("Test completed successfully"),
        Ok(Err(e)) => eprintln!("sacp error: {e}"),
        Err(_) => eprintln!("Test timed out after 60s"),
    }
}
