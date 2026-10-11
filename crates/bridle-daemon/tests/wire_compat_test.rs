//! Wire compatibility for the task -> ticket rename (22ab step 9): an old client's request paths
//! and JSON bodies still work against the new daemon, and the new names work too.

mod support;

use bridle_api::types::{NewTicketRequest, TicketKind, TokenCreateRequest};
use support::start_daemon;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// One raw HTTP/1.1 request, as an old client would send it; returns (status, body).
async fn raw(
    addr: &str,
    token: &str,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> (u16, String) {
    let mut conn = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let body = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    conn.write_all(req.as_bytes()).await.expect("write");
    let mut out = String::new();
    conn.read_to_string(&mut out).await.expect("read");
    let status = out[9..12].parse().expect("status");
    let body = out.split_once("\r\n\r\n").map_or("", |(_, b)| b);
    (status, body.to_string())
}

#[tokio::test]
async fn old_task_paths_and_bodies_work_next_to_the_ticket_ones() {
    let (daemon, _tmp) = start_daemon(None).await;
    let addr = daemon.running.url.trim_start_matches("http://").to_string();
    let token = daemon
        .client
        .create_token(&TokenCreateRequest {
            name: "old-client".to_string(),
            machine: None,
            home: None,
        })
        .await
        .expect("token")
        .token;

    // An old client creates a ticket on the old path with an old-shape body.
    let (status, body) = raw(
        &addr,
        &token,
        "POST",
        "/v1/tasks",
        Some(r#"{"title":"old shape","kind":"chore","body":"b"}"#),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let id = serde_json::from_str::<serde_json::Value>(&body).expect("json")["id"]
        .as_str()
        .expect("id")
        .to_string();

    // Both paths read the same record, and the new client method finds it.
    for prefix in ["tasks", "tickets"] {
        let (status, body) = raw(&addr, &token, "GET", &format!("/v1/{prefix}/{id}"), None).await;
        assert_eq!(status, 200, "{prefix}: {body}");
        assert!(body.contains("old shape"), "{prefix}: {body}");
    }
    assert_eq!(
        daemon.client.get_task(&id).await.expect("get").title,
        "old shape"
    );

    // A new-path write is visible on the old path.
    let (status, body) = raw(
        &addr,
        &token,
        "POST",
        &format!("/v1/tickets/{id}/note"),
        Some(r#"{"body":"hello"}"#),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let (_, listed) = raw(&addr, &token, "GET", "/v1/tasks", None).await;
    assert!(listed.contains(&id), "{listed}");

    // New client creates through /v1/tickets.
    let made = daemon
        .client
        .new_task(&NewTicketRequest {
            title: "new shape".to_string(),
            kind: TicketKind::Chore,
            body: String::new(),
            size: None,
            components: Vec::new(),
            for_human: false,
            priority: None,
            ticket: None,
            parent: None,
        })
        .await
        .expect("new");
    assert_eq!(made.title, "new shape");

    // Request fields take the new names as well as the old: `ticket` for `task`.
    let (status, body) = raw(
        &addr,
        &token,
        "POST",
        "/v1/messages",
        Some(&format!(
            r#"{{"to":"human","body":"x","ticket":"{id}","kind":"note"}}"#
        )),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert!(body.contains(&id), "{body}");
}
