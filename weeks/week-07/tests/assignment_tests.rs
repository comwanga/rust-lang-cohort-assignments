use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use week_07_async_node::*;

fn genesis() -> Block {
    Block {
        hash: "genesis".to_string(),
        previous_hash: "0".to_string(),
        height: 0,
        payload: "genesis".to_string(),
    }
}

fn block(hash: &str, previous_hash: &str, height: u64) -> Block {
    Block {
        hash: hash.to_string(),
        previous_hash: previous_hash.to_string(),
        height,
        payload: format!("payload-{height}"),
    }
}

#[test]
fn block_new_copies_fields() {
    let block = Block::new("h1", "h0", 1, "payload");
    assert_eq!(block.hash, "h1");
    assert_eq!(block.previous_hash, "h0");
    assert_eq!(block.height, 1);
    assert_eq!(block.payload, "payload");
}

#[test]
fn block_genesis_uses_expected_values() {
    assert_eq!(Block::genesis(), genesis());
}

#[test]
fn block_wire_format_uses_pipe_separator() {
    assert_eq!(block("h1", "h0", 1).wire_format(), "h1|h0|1|payload-1");
}

#[test]
fn validate_genesis_requires_height_zero_and_previous_zero() {
    assert_eq!(genesis().validate_against_tip(None), Ok(()));
    assert_eq!(
        block("bad", "not-zero", 0).validate_against_tip(None),
        Err(NodeError::InvalidBlock)
    );
}

#[test]
fn validate_non_genesis_requires_tip_hash_and_next_height() {
    let tip = genesis();
    assert_eq!(
        block("h1", "genesis", 1).validate_against_tip(Some(&tip)),
        Ok(())
    );
    assert_eq!(
        block("h1", "wrong", 1).validate_against_tip(Some(&tip)),
        Err(NodeError::InvalidBlock)
    );
    assert_eq!(
        block("h1", "genesis", 2).validate_against_tip(Some(&tip)),
        Err(NodeError::InvalidBlock)
    );
}

#[test]
fn parse_block_accepts_wire_format() {
    assert_eq!(
        parse_block("h1|h0|1|payload").unwrap(),
        Block::new("h1", "h0", 1, "payload")
    );
}

#[test]
fn parse_block_rejects_wrong_field_count() {
    assert_eq!(parse_block("h1|h0|1"), Err(NodeError::MalformedMessage));
}

#[test]
fn parse_block_rejects_non_numeric_height() {
    assert_eq!(
        parse_block("h1|h0|not-height|payload"),
        Err(NodeError::MalformedMessage)
    );
}

#[test]
fn parse_request_accepts_simple_commands() {
    assert_eq!(parse_request("ping\n"), Ok(NodeRequest::Ping));
    assert_eq!(parse_request("height"), Ok(NodeRequest::Height));
    assert_eq!(parse_request("get_tip"), Ok(NodeRequest::GetTip));
    assert_eq!(parse_request("get_peers"), Ok(NodeRequest::GetPeers));
}

#[test]
fn parse_request_accepts_get_block() {
    assert_eq!(
        parse_request("get_block abc123"),
        Ok(NodeRequest::GetBlock("abc123".to_string()))
    );
}

#[test]
fn parse_request_accepts_add_peer() {
    assert_eq!(
        parse_request("add_peer 127.0.0.1:18444"),
        Ok(NodeRequest::AddPeer("127.0.0.1:18444".to_string()))
    );
}

#[test]
fn parse_request_accepts_submit_block() {
    assert_eq!(
        parse_request("submit_block h1|genesis|1|payload-1"),
        Ok(NodeRequest::SubmitBlock(block("h1", "genesis", 1)))
    );
}

#[test]
fn parse_request_rejects_missing_argument() {
    assert_eq!(parse_request("get_block"), Err(NodeError::MalformedMessage));
    assert_eq!(parse_request("add_peer"), Err(NodeError::MalformedMessage));
}

#[test]
fn parse_request_rejects_unknown_command() {
    assert_eq!(parse_request("mempool"), Err(NodeError::UnknownCommand));
}

#[test]
fn encode_response_formats_simple_responses() {
    assert_eq!(encode_response(&NodeResponse::Pong), "pong\n");
    assert_eq!(encode_response(&NodeResponse::Height(3)), "height 3\n");
    assert_eq!(
        encode_response(&NodeResponse::Tip("abc".to_string())),
        "tip abc\n"
    );
}

#[test]
fn encode_response_formats_block_and_peer_responses() {
    assert_eq!(
        encode_response(&NodeResponse::Block(block("h1", "h0", 1))),
        "block h1|h0|1|payload-1\n"
    );
    assert_eq!(
        encode_response(&NodeResponse::Peers(vec!["a".to_string(), "b".to_string()])),
        "peers a,b\n"
    );
}

#[test]
fn encode_response_formats_errors_and_acceptance() {
    assert_eq!(
        encode_response(&NodeResponse::Accepted("h1".to_string())),
        "accepted h1\n"
    );
    assert_eq!(
        encode_response(&NodeResponse::Rejected("bad prev".to_string())),
        "rejected bad prev\n"
    );
    assert_eq!(encode_response(&NodeResponse::NotFound), "not_found\n");
    assert_eq!(
        encode_response(&NodeResponse::Error("bad request".to_string())),
        "error bad request\n"
    );
}

#[test]
fn parse_response_accepts_encoded_simple_responses() {
    assert_eq!(parse_response("pong\n"), Ok(NodeResponse::Pong));
    assert_eq!(parse_response("height 3"), Ok(NodeResponse::Height(3)));
    assert_eq!(
        parse_response("tip abc123"),
        Ok(NodeResponse::Tip("abc123".to_string()))
    );
}

#[test]
fn parse_response_accepts_blocks_peers_and_errors() {
    assert_eq!(
        parse_response("block h1|h0|1|payload-1"),
        Ok(NodeResponse::Block(block("h1", "h0", 1)))
    );
    assert_eq!(
        parse_response("peers a,b"),
        Ok(NodeResponse::Peers(vec!["a".to_string(), "b".to_string()]))
    );
    assert_eq!(
        parse_response("rejected bad prev"),
        Ok(NodeResponse::Rejected("bad prev".to_string()))
    );
    assert_eq!(
        parse_response("error malformed"),
        Ok(NodeResponse::Error("malformed".to_string()))
    );
}

#[test]
fn parse_response_rejects_malformed_known_response() {
    assert_eq!(
        parse_response("height not-a-number"),
        Err(NodeError::MalformedMessage)
    );
    assert_eq!(
        parse_response("block broken"),
        Err(NodeError::MalformedMessage)
    );
}

#[test]
fn parse_response_rejects_unknown_prefix() {
    assert_eq!(
        parse_response("mystery value"),
        Err(NodeError::UnknownCommand)
    );
}

#[test]
fn node_state_new_starts_from_genesis() {
    let state = NodeState::new(genesis()).unwrap();
    assert_eq!(state.height(), 0);
    assert_eq!(state.tip_hash(), Some("genesis"));
    assert!(state.peers.is_empty());
}

#[test]
fn node_state_add_peer_records_height_and_returns_count() {
    let mut state = NodeState::new(genesis()).unwrap();
    assert_eq!(state.add_peer("b"), 1);
    assert_eq!(state.add_peer("a"), 2);
    assert_eq!(
        state.peer_addresses(),
        vec!["a".to_string(), "b".to_string()]
    );
    assert_eq!(state.peers["a"].last_seen_height, 0);
}

#[test]
fn node_state_append_block_updates_tip() {
    let mut state = NodeState::new(genesis()).unwrap();
    state.append_block(block("h1", "genesis", 1)).unwrap();
    assert_eq!(state.height(), 1);
    assert_eq!(state.tip_hash(), Some("h1"));
}

#[test]
fn node_state_append_block_rejects_bad_previous_hash() {
    let mut state = NodeState::new(genesis()).unwrap();
    assert_eq!(
        state.append_block(block("h1", "wrong", 1)),
        Err(NodeError::InvalidBlock)
    );
}

#[test]
fn node_state_get_block_finds_by_hash() {
    let mut state = NodeState::new(genesis()).unwrap();
    state.append_block(block("h1", "genesis", 1)).unwrap();
    assert_eq!(state.get_block("h1").unwrap().height, 1);
    assert!(state.get_block("missing").is_none());
}

#[test]
fn handle_request_ping_height_tip_and_peers() {
    let mut state = NodeState::new(genesis()).unwrap();
    assert_eq!(
        handle_request(&mut state, NodeRequest::Ping),
        NodeResponse::Pong
    );
    assert_eq!(
        handle_request(&mut state, NodeRequest::Height),
        NodeResponse::Height(0)
    );
    assert_eq!(
        handle_request(&mut state, NodeRequest::GetTip),
        NodeResponse::Tip("genesis".to_string())
    );
    assert_eq!(
        handle_request(&mut state, NodeRequest::AddPeer("peer1".to_string())),
        NodeResponse::PeerAdded(1)
    );
    assert_eq!(
        handle_request(&mut state, NodeRequest::GetPeers),
        NodeResponse::Peers(vec!["peer1".to_string()])
    );
}

#[test]
fn handle_request_get_block_returns_block_or_not_found() {
    let mut state = NodeState::new(genesis()).unwrap();
    assert_eq!(
        handle_request(&mut state, NodeRequest::GetBlock("genesis".to_string())),
        NodeResponse::Block(genesis())
    );
    assert_eq!(
        handle_request(&mut state, NodeRequest::GetBlock("missing".to_string())),
        NodeResponse::NotFound
    );
}

#[test]
fn handle_request_submit_block_accepts_or_rejects() {
    let mut state = NodeState::new(genesis()).unwrap();
    assert_eq!(
        handle_request(
            &mut state,
            NodeRequest::SubmitBlock(block("h1", "genesis", 1))
        ),
        NodeResponse::Accepted("h1".to_string())
    );
    assert!(matches!(
        handle_request(
            &mut state,
            NodeRequest::SubmitBlock(block("bad", "wrong", 2))
        ),
        NodeResponse::Rejected(_)
    ));
}

#[tokio::test]
async fn send_request_round_trips_through_state_manager() {
    let sender = spawn_state_manager(NodeState::new(genesis()).unwrap(), 4);
    assert_eq!(
        send_request(&sender, NodeRequest::Ping).await.unwrap(),
        NodeResponse::Pong
    );
    assert_eq!(
        send_request(&sender, NodeRequest::Height).await.unwrap(),
        NodeResponse::Height(0)
    );
}

#[tokio::test]
async fn handle_line_parses_sends_and_encodes() {
    let sender = spawn_state_manager(NodeState::new(genesis()).unwrap(), 4);
    assert_eq!(handle_line(&sender, "ping").await.unwrap(), "pong\n");
    assert_eq!(
        handle_line(&sender, "submit_block h1|genesis|1|payload-1")
            .await
            .unwrap(),
        "accepted h1\n"
    );
    assert_eq!(handle_line(&sender, "height").await.unwrap(), "height 1\n");
}

#[tokio::test]
async fn send_request_reports_closed_channel() {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    drop(receiver);
    assert_eq!(
        send_request(&sender, NodeRequest::Ping).await,
        Err(NodeError::ChannelClosed)
    );
}

#[tokio::test]
async fn handle_connection_replies_to_multiple_lines() {
    let sender = spawn_state_manager(NodeState::new(genesis()).unwrap(), 8);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_sender = sender.clone();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        handle_connection(stream, server_sender).await.unwrap();
    });

    let stream = TcpStream::connect(addr).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    writer.write_all(b"ping\n").await.unwrap();
    reader.read_line(&mut line).await.unwrap();
    assert_eq!(line, "pong\n");

    line.clear();
    writer.write_all(b"height\n").await.unwrap();
    reader.read_line(&mut line).await.unwrap();
    assert_eq!(line, "height 0\n");

    drop(writer);
    server.await.unwrap();
}

#[tokio::test]
async fn run_tcp_server_stops_on_shutdown() {
    let sender = spawn_state_manager(NodeState::new(genesis()).unwrap(), 8);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server = tokio::spawn(run_tcp_server(listener, sender, shutdown_rx));
    shutdown_tx.send(()).unwrap();
    assert_eq!(server.await.unwrap(), Ok(()));
}
