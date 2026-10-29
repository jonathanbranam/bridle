//! Mail between daemons (3haz slice 1): two in-process daemons, A sending to B through its own
//! outbox with a peer token.

mod support;

use bridle_api::Client;
use bridle_api::discovery::store_credential;
use bridle_api::types::{
    ForwardRequest, HelloRequest, MessageKind, MessageQuery, NewTicketRequest, OutboxSendRequest,
    PeerTokenCreateRequest, PeerWatchRequest, RemoteWatchRequest, TicketKind, TokenCreateRequest,
    When,
};
use support::{ClientExt, TestDaemon, machine_home_dir, start_daemon_named, wait_for};

const NO_MANAGER: &str = "[roles.manager]\nautostart = false\n";

struct Pair {
    a: TestDaemon,
    b: TestDaemon,
    /// B's project, the key of A's `[peer]` entry.
    b_project: String,
    /// The peer token B minted for machine `m1`.
    peer_token: String,
    _tmp_a: tempfile::TempDir,
    _tmp_b: tempfile::TempDir,
}

impl Pair {
    fn peer_client(&self) -> Client {
        Client::new(self.b.running.url.clone(), Some(self.peer_token.clone()))
    }

    /// Writes A's machine config and credentials: `b` lives on machine `m2`.
    fn point_a_at_b(&self, token: &str) {
        let home = machine_home_dir(self._tmp_a.path());
        std::fs::create_dir_all(&home).expect("home");
        let port = self.b.running.url.rsplit(':').next().expect("port");
        std::fs::write(
            home.join("config.toml"),
            format!(
                "[machine]\nname = \"m1\"\n[machines]\nm1 = \"127.0.0.1\"\nm2 = \"127.0.0.1\"\n\
                 [projects]\n{} = {{ machine = \"m2\", port = {port} }}\n",
                self.b_project
            ),
        )
        .expect("config");
        let creds = home.join("credentials.toml");
        let _ = std::fs::remove_file(&creds);
        store_credential(&creds, "peer", &self.b_project, token).expect("credentials");
    }
}

async fn pair() -> Pair {
    let (a, tmp_a) = start_daemon_named(Some("alpha"), None, Some(NO_MANAGER)).await;
    let (b, tmp_b) = start_daemon_named(Some("beta"), None, Some(NO_MANAGER)).await;
    let b_project = b.running.info.project.clone();
    let peer_token = b
        .client
        .create_peer_token(&PeerTokenCreateRequest {
            machine: "m1".to_string(),
            replace: false,
        })
        .await
        .expect("peer token")
        .token;
    let pair = Pair {
        a,
        b,
        b_project,
        peer_token: peer_token.clone(),
        _tmp_a: tmp_a,
        _tmp_b: tmp_b,
    };
    pair.point_a_at_b(&peer_token);
    pair
}

fn out(project: &str, to: &str, body: &str) -> OutboxSendRequest {
    OutboxSendRequest {
        project: project.to_string(),
        to: to.to_string(),
        body: body.to_string(),
        kind: MessageKind::Note,
        when: When::Now,
        reply_to: None,
    }
}

fn fwd(origin_id: &str, from: &str, to: &str, body: &str) -> ForwardRequest {
    ForwardRequest {
        origin_machine: "m1".to_string(),
        origin_daemon: "a".to_string(),
        origin_id: origin_id.to_string(),
        from: from.to_string(),
        to: to.to_string(),
        body: body.to_string(),
        kind: MessageKind::Note,
        when: When::Now,
        reply_to: None,
    }
}

async fn inbox_bodies(b: &TestDaemon, to: &str) -> Vec<(String, String)> {
    let mut msgs = b
        .client
        .list_messages(&MessageQuery {
            to: Some(to.to_string()),
            ..Default::default()
        })
        .await
        .expect("messages");
    msgs.sort_by(|x, y| x.id.cmp(&y.id));
    msgs.into_iter().map(|m| (m.from, m.body)).collect()
}

async fn wait_for_bodies(b: &TestDaemon, to: &str, n: usize) -> Vec<(String, String)> {
    wait_for("forwarded mail", || async {
        let got = inbox_bodies(b, to).await;
        (got.len() >= n).then_some(got)
    })
    .await
}

#[tokio::test]
async fn a_send_is_queued_on_the_senders_daemon_then_delivered_with_a_machine_label() {
    let p = pair().await;
    p.b.external_client("advisor").await;
    let sender = p.a.external_client("orchestrator").await;

    let queued = sender
        .send_outbox(&out(&p.b_project, "external:advisor", "hello"))
        .await
        .expect("accepted at once");
    assert!(queued.id.starts_with("o-"), "{queued:?}");

    let got = wait_for_bodies(&p.b, "external:advisor", 1).await;
    assert_eq!(
        got,
        vec![("external:orchestrator@m1".to_string(), "hello".to_string())]
    );
}

#[tokio::test]
async fn a_send_to_a_down_daemon_is_still_accepted_and_the_queue_keeps_its_order() {
    let p = pair().await;
    p.b.external_client("advisor").await;
    let sender = p.a.external_client("orchestrator").await;
    // A wrong token: every try is refused and stays queued (a transient failure).
    p.point_a_at_b(&"0".repeat(64));

    for body in ["one", "two"] {
        sender
            .send_outbox(&out(&p.b_project, "external:advisor", body))
            .await
            .expect("accepted although it can't be delivered");
    }
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    assert!(inbox_bodies(&p.b, "external:advisor").await.is_empty());

    // The token is fixed; the next send flushes the queue oldest first. (Without a send it
    // would wait out the backoff, or a greeting from the peer: see the hello test.)
    p.point_a_at_b(&p.peer_token);
    sender
        .send_outbox(&out(&p.b_project, "external:advisor", "three"))
        .await
        .expect("queued");
    let got = wait_for_bodies(&p.b, "external:advisor", 3).await;
    let bodies: Vec<_> = got.iter().map(|(_, b)| b.as_str()).collect();
    assert_eq!(bodies, ["one", "two", "three"]);
}

#[tokio::test]
async fn a_message_refused_for_good_does_not_block_the_ones_behind_it() {
    let p = pair().await;
    p.b.external_client("advisor").await;
    let sender = p.a.external_client("orchestrator").await;
    sender
        .send_outbox(&out(&p.b_project, "external:nobody", "lost"))
        .await
        .expect("accepted");
    sender
        .send_outbox(&out(&p.b_project, "external:advisor", "next"))
        .await
        .expect("accepted");
    let got = wait_for_bodies(&p.b, "external:advisor", 1).await;
    assert_eq!(got[0].1, "next");
}

#[tokio::test]
async fn a_repeated_forward_is_acknowledged_again_and_delivered_once() {
    let p = pair().await;
    p.b.external_client("advisor").await;
    let peer = p.peer_client();
    let req = fwd("o-0001", "agent:w1@m1", "external:advisor", "once");

    let first = peer.forward(&req).await.expect("first");
    // The acknowledgement was lost; the sender tries again.
    let again = peer.forward(&req).await.expect("repeat");
    assert_eq!(first, again);
    assert_eq!(first.message_ids.len(), 1);

    let got = inbox_bodies(&p.b, "external:advisor").await;
    assert_eq!(got, vec![("agent:w1@m1".to_string(), "once".to_string())]);
}

#[tokio::test]
async fn only_a_peer_token_is_believed_about_the_sender() {
    let p = pair().await;
    p.b.external_client("advisor").await;
    let req = fwd("o-0001", "external:advisor@m9", "external:advisor", "x");

    // A visitor's token and a plain external token can't forward.
    let visitor =
        p.b.client
            .create_token(&TokenCreateRequest {
                name: "orchestrator".to_string(),
                machine: Some("m1".to_string()),
                home: None,
            })
            .await
            .expect("visitor");
    let visitor = Client::new(p.b.running.url.clone(), Some(visitor.token));
    let err = visitor.forward(&req).await.expect_err("visitor refused");
    assert!(err.to_string().contains("peer token"), "{err}");
    let external = p.b.external_client("someone").await;
    assert!(external.forward(&req).await.is_err());

    // A peer token can't do anything else.
    let peer = p.peer_client();
    let err = peer
        .send(&bridle_api::types::SendRequest {
            to: Some("external:advisor".to_string()),
            body: "hi".to_string(),
            ..Default::default()
        })
        .await
        .expect_err("peer token only forwards");
    assert!(err.to_string().contains("only forward"), "{err}");

    // And what it forwards carries the label it states.
    peer.forward(&req).await.expect("peer forwards");
    let got = inbox_bodies(&p.b, "external:advisor").await;
    assert_eq!(got[0].0, "external:advisor@m9");
}

#[tokio::test]
async fn a_destination_with_no_peer_token_is_refused_up_front() {
    let p = pair().await;
    let sender = p.a.external_client("orchestrator").await;
    std::fs::remove_file(machine_home_dir(p._tmp_a.path()).join("credentials.toml")).expect("rm");
    let err = sender
        .send_outbox(&out(&p.b_project, "human", "hi"))
        .await
        .expect_err("no token");
    assert!(err.to_string().contains("peer token"), "{err}");
}

#[tokio::test]
async fn the_send_reports_how_the_first_try_went() {
    let p = pair().await;
    p.b.external_client("advisor").await;
    let sender = p.a.external_client("orchestrator").await;

    let ok = sender
        .send_outbox(&out(&p.b_project, "external:advisor", "hello"))
        .await
        .expect("send");
    assert_eq!(ok.state, "delivered");

    let refused = sender
        .send_outbox(&out(&p.b_project, "external:nobody", "lost"))
        .await
        .expect("accepted");
    assert_eq!(refused.state, "failed");
    assert!(
        refused
            .last_error
            .as_deref()
            .is_some_and(|e| e.contains("no such recipient")),
        "{refused:?}"
    );
    // ... and the sender is told in its inbox too, by the daemon, once.
    let notes = wait_for("the refusal notice", || async {
        let got = inbox_bodies(&p.a, "external:orchestrator").await;
        (!got.is_empty()).then_some(got)
    })
    .await;
    assert_eq!(notes.len(), 1, "{notes:?}");
    assert_eq!(notes[0].0, "system");
    assert!(notes[0].1.contains("external:nobody"), "{notes:?}");

    p.point_a_at_b(&"0".repeat(64));
    let down = sender
        .send_outbox(&out(&p.b_project, "external:advisor", "later"))
        .await
        .expect("accepted");
    assert_eq!(down.state, "queued");
    assert!(down.last_error.is_some(), "{down:?}");
}

#[tokio::test]
async fn a_peer_saying_hello_gets_its_queue_flushed_at_once() {
    let p = pair().await;
    p.b.external_client("advisor").await;
    let sender = p.a.external_client("orchestrator").await;
    p.point_a_at_b(&"0".repeat(64));
    sender
        .send_outbox(&out(&p.b_project, "external:advisor", "waiting"))
        .await
        .expect("queued");
    // B comes back (its token is good again), and greets A with a peer token A minted for it.
    p.point_a_at_b(&p.peer_token);
    let a_peer =
        p.a.client
            .create_peer_token(&PeerTokenCreateRequest {
                machine: "m2".to_string(),
                replace: false,
            })
            .await
            .expect("peer token")
            .token;
    let greeting = Client::new(p.a.running.url.clone(), Some(a_peer));
    greeting
        .hello(&HelloRequest {
            daemon: p.b_project.clone(),
        })
        .await
        .expect("hello");
    // Well inside the 30 s backoff.
    let got = wait_for_bodies(&p.b, "external:advisor", 1).await;
    assert_eq!(got[0].1, "waiting");

    // Only a peer token may greet.
    let err = sender
        .hello(&HelloRequest {
            daemon: p.b_project.clone(),
        })
        .await
        .expect_err("not a peer");
    assert!(err.to_string().contains("peer token"), "{err}");
}

#[tokio::test]
async fn mail_to_agent_colon_name_reaches_the_agent_across_daemons() {
    let p = pair().await;
    p.b.client
        .spawn(&bridle_api::types::SpawnRequest {
            ticket: None,
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(bridle_api::types::Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    let sender = p.a.external_client("orchestrator").await;
    let sent = sender
        .send_outbox(&out(&p.b_project, "agent:w1", "for you"))
        .await
        .expect("send");
    assert_eq!(sent.state, "delivered", "{sent:?}");
}

/// Slice 2: B (the "remote" daemon) forwards mail for a visitor to the visitor's home daemon A,
/// where its ordinary inbox (and waiter) is; a visitor minted without a home keeps its inbox.
#[tokio::test]
async fn mail_for_a_visitor_is_forwarded_to_its_home_daemon() {
    let p = pair().await;
    let a_project = p.a.running.info.project.clone();

    // B reaches A as machine `m2`: A mints the peer token, B's machine config and credentials
    // point at A.
    let a_peer =
        p.a.client
            .create_peer_token(&PeerTokenCreateRequest {
                machine: "m2".to_string(),
                replace: false,
            })
            .await
            .expect("peer token")
            .token;
    let b_home = machine_home_dir(p._tmp_b.path());
    std::fs::create_dir_all(&b_home).expect("home");
    let a_port = p.a.running.url.rsplit(':').next().expect("port");
    std::fs::write(
        b_home.join("config.toml"),
        format!(
            "[machine]\nname = \"m2\"\n[machines]\nm1 = \"127.0.0.1\"\nm2 = \"127.0.0.1\"\n\
             [projects]\n{a_project} = {{ machine = \"m1\", port = {a_port} }}\n"
        ),
    )
    .expect("config");
    store_credential(
        &b_home.join("credentials.toml"),
        "peer",
        &a_project,
        &a_peer,
    )
    .expect("credentials");

    // The visitor's home must be reachable when its token is minted.
    let err =
        p.b.client
            .create_token(&TokenCreateRequest {
                name: "aide".to_string(),
                machine: Some("m1".to_string()),
                home: Some("nowhere".to_string()),
            })
            .await
            .expect_err("unknown home");
    assert!(err.to_string().contains("nowhere"), "{err}");

    for (name, home) in [("aide", Some(a_project.clone())), ("old", None)] {
        p.b.client
            .create_token(&TokenCreateRequest {
                name: name.to_string(),
                machine: Some("m1".to_string()),
                home,
            })
            .await
            .expect("visitor");
    }
    p.a.external_client("aide").await;
    let advisor = p.b.external_client("advisor").await;
    for to in ["external:aide@m1", "external:old@m1"] {
        advisor
            .send(&bridle_api::types::SendRequest {
                to: Some(to.to_string()),
                body: format!("reply for {to}"),
                ..Default::default()
            })
            .await
            .expect("send");
    }

    // The reply lands in A's inbox, from the sender qualified with B's machine ...
    let got = wait_for_bodies(&p.a, "external:aide", 1).await;
    assert_eq!(
        got,
        vec![(
            "external:advisor@m2".to_string(),
            "reply for external:aide@m1".to_string()
        )]
    );
    // ... and not in B's; the token with no home keeps its mail on B, as before.
    assert!(inbox_bodies(&p.b, "external:aide@m1").await.is_empty());
    assert_eq!(inbox_bodies(&p.b, "external:old@m1").await.len(), 1);
}

/// x8c8: a visitor whose recorded home is this daemon's own project is refused, not delivered to
/// the local principal of the same name.
#[tokio::test]
async fn mail_for_a_visitor_whose_home_is_this_project_is_refused() {
    let p = pair().await;
    let b_project = p.b.running.info.project.clone();
    let b_peer =
        p.b.client
            .create_peer_token(&PeerTokenCreateRequest {
                machine: "m2".to_string(),
                replace: false,
            })
            .await
            .expect("peer token")
            .token;
    let b_home = machine_home_dir(p._tmp_b.path());
    std::fs::create_dir_all(&b_home).expect("home");
    let b_port = p.b.running.url.rsplit(':').next().expect("port");
    std::fs::write(
        b_home.join("config.toml"),
        format!(
            "[machine]\nname = \"m2\"\n[machines]\nm1 = \"127.0.0.1\"\nm2 = \"127.0.0.1\"\n\
             [projects]\n{b_project} = {{ machine = \"m1\", port = {b_port} }}\n"
        ),
    )
    .expect("config");
    store_credential(
        &b_home.join("credentials.toml"),
        "peer",
        &b_project,
        &b_peer,
    )
    .expect("credentials");
    p.b.client
        .create_token(&TokenCreateRequest {
            name: "orchestrator".to_string(),
            machine: Some("m1".to_string()),
            home: Some(b_project),
        })
        .await
        .expect("visitor");
    let local = p.b.external_client("orchestrator").await;
    let advisor = p.b.external_client("advisor").await;
    let err = advisor
        .send(&bridle_api::types::SendRequest {
            to: Some("external:orchestrator@m1".to_string()),
            body: "must not loop".to_string(),
            ..Default::default()
        })
        .await
        .expect_err("refused");
    assert!(err.to_string().contains("own project"), "{err}");
    drop(local);
    assert!(inbox_bodies(&p.b, "external:orchestrator").await.is_empty());
}

/// B holds a visitor `external:orchestrator@m1` whose home is A, and can reach A.
async fn visiting_orchestrator_on_b(p: &Pair) {
    orchestrator_on_b(p, Some("m1")).await;
}

/// B holds an orchestrator token whose home is A: a visitor from `machine`, or with None the
/// machine's own direct token (br-j86p).
async fn orchestrator_on_b(p: &Pair, machine: Option<&str>) -> bridle_api::Client {
    let a_project = p.a.running.info.project.clone();
    let a_peer =
        p.a.client
            .create_peer_token(&PeerTokenCreateRequest {
                machine: "m2".to_string(),
                replace: false,
            })
            .await
            .expect("peer token")
            .token;
    let b_home = machine_home_dir(p._tmp_b.path());
    std::fs::create_dir_all(&b_home).expect("home");
    let a_port = p.a.running.url.rsplit(':').next().expect("port");
    std::fs::write(
        b_home.join("config.toml"),
        format!(
            "[machine]\nname = \"m2\"\n[machines]\nm1 = \"127.0.0.1\"\nm2 = \"127.0.0.1\"\n\
             [projects]\n{a_project} = {{ machine = \"m1\", port = {a_port} }}\n"
        ),
    )
    .expect("config");
    store_credential(
        &b_home.join("credentials.toml"),
        "peer",
        &a_project,
        &a_peer,
    )
    .expect("credentials");
    let created =
        p.b.client
            .create_token(&TokenCreateRequest {
                name: "orchestrator".to_string(),
                machine: machine.map(str::to_string),
                home: Some(a_project),
            })
            .await
            .expect("orchestrator token");
    bridle_api::Client::new(p.b.running.url.clone(), Some(created.token))
}

/// Slice 5: a wake B raises for the orchestrator (an incident here) is sent home to A as one
/// `system` message, where the orchestrator's single waiter hears it; B's own wake path still
/// works.
#[tokio::test]
async fn a_wake_on_a_remote_project_is_forwarded_home_once_as_a_system_message() {
    let p = pair().await;
    visiting_orchestrator_on_b(&p).await;
    let orch = p.a.external_client("orchestrator").await;

    p.b.client
        .new_task(&NewTicketRequest {
            ticket: None,
            parent: None,
            for_human: false,
            priority: None,
            components: Vec::new(),
            title: "the build is red".to_string(),
            kind: TicketKind::Incident,
            body: "red".to_string(),
            size: None,
        })
        .await
        .expect("incident");

    // The wake loop ticks every 10 s.
    let got = wait_for("forwarded wake", || async {
        let m = orch
            .list_messages(&MessageQuery {
                to: Some("external:orchestrator".to_string()),
                ..Default::default()
            })
            .await
            .expect("messages");
        (!m.is_empty()).then_some(m)
    })
    .await;
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].kind, MessageKind::System);
    assert_eq!(got[0].from, "system@m2");
    assert!(
        got[0].body.starts_with("[incident_created] incident "),
        "{}",
        got[0].body
    );
    // Not sent again on later ticks.
    tokio::time::sleep(std::time::Duration::from_secs(12)).await;
    let again = orch
        .list_messages(&MessageQuery {
            to: Some("external:orchestrator".to_string()),
            ..Default::default()
        })
        .await
        .expect("messages");
    assert_eq!(again.len(), 1, "{again:?}");
}

/// br-j86p: the machine's own orchestrator keeps a direct token with a recorded home: it has the
/// orchestrator's rights on B (a cross-machine visitor does not), and B's wakes still go home.
#[tokio::test]
async fn a_direct_orchestrator_with_a_home_keeps_its_rights_and_its_wakes_forward() {
    let p = pair().await;
    let direct = orchestrator_on_b(&p, None).await;
    let orch_home = p.a.external_client("orchestrator").await;

    let t =
        p.b.client
            .new_task(&NewTicketRequest {
                ticket: None,
                parent: None,
                for_human: false,
                priority: None,
                components: Vec::new(),
                title: "the build is red".to_string(),
                kind: TicketKind::Incident,
                body: "red".to_string(),
                size: None,
            })
            .await
            .expect("incident");
    // Rights: it may comment on a ticket it did not file.
    direct
        .note_task(&t.id, "looking")
        .await
        .expect("a direct orchestrator comments");
    // The same call as a cross-machine visitor is refused.
    let created =
        p.b.client
            .create_token(&TokenCreateRequest {
                name: "advisor".to_string(),
                machine: Some("m9".to_string()),
                home: None,
            })
            .await
            .expect("visitor");
    let visitor = bridle_api::Client::new(p.b.running.url.clone(), Some(created.token));
    assert!(visitor.note_task(&t.id, "hi").await.is_err());
    // Readying is the orchestrator's too; the visitor is refused.
    assert!(visitor.ready_task(&t.id).await.is_err());
    direct
        .ready_task(&t.id)
        .await
        .expect("a direct orchestrator readies");

    // Its wake goes home to A once the loop ticks (every 10 s).
    let got = wait_for("forwarded wake", || async {
        let m = orch_home
            .list_messages(&MessageQuery {
                to: Some("external:orchestrator".to_string()),
                ..Default::default()
            })
            .await
            .expect("messages");
        (!m.is_empty()).then_some(m)
    })
    .await;
    assert_eq!(got[0].kind, MessageKind::System);
    assert_eq!(got[0].from, "system@m2");

    // Mail for it on B is forwarded home too.
    p.b.client
        .send(&bridle_api::types::SendRequest {
            to: Some("external:orchestrator".to_string()),
            body: "ping".to_string(),
            ..Default::default()
        })
        .await
        .expect("send");
    wait_for_bodies(&p.a, "external:orchestrator", 2).await;
}

/// br-dg5g: mail for the machine's orchestrator goes where its waiter is, not to its recorded
/// home. B's direct token records A as home, but a waiter held on B keeps B's mail on B; with the
/// waiter's note gone, the mail goes home again.
#[tokio::test]
async fn orchestrator_mail_follows_the_waiter_not_the_recorded_home() {
    let p = pair().await;
    orchestrator_on_b(&p, None).await;
    p.a.external_client("orchestrator").await;
    let home = machine_home_dir(p._tmp_b.path());
    let b_project = p.b.running.info.project.clone();
    let now = chrono::Utc::now().timestamp();
    std::fs::write(
        home.join("orchestrator-waiter"),
        format!("{b_project} {now}\n"),
    )
    .expect("note");

    async fn send(p: &Pair, body: &str) -> Result<(), bridle_api::ClientError> {
        p.b.client
            .send(&bridle_api::types::SendRequest {
                to: Some("external:orchestrator".to_string()),
                body: body.to_string(),
                ..Default::default()
            })
            .await
            .map(|_| ())
    }
    send(&p, "stays").await.expect("send");
    let kept = inbox_bodies(&p.b, "external:orchestrator").await;
    assert_eq!(kept.len(), 1, "{kept:?}");
    assert!(inbox_bodies(&p.a, "external:orchestrator").await.is_empty());

    std::fs::remove_file(home.join("orchestrator-waiter")).expect("note");
    send(&p, "goes home").await.expect("send");
    wait_for_bodies(&p.a, "external:orchestrator", 1).await;
}

/// br-nncj: a daemon with its own orchestrator keeps its wakes for it; a visiting orchestrator
/// homed on another machine gets no copy.
#[tokio::test]
async fn a_wake_is_not_forwarded_to_a_visiting_orchestrator_when_there_is_a_local_one() {
    let p = pair().await;
    visiting_orchestrator_on_b(&p).await;
    p.b.external_client("orchestrator").await;
    let orch = p.a.external_client("orchestrator").await;

    p.b.client
        .new_task(&NewTicketRequest {
            ticket: None,
            parent: None,
            for_human: false,
            priority: None,
            components: Vec::new(),
            title: "the build is red".to_string(),
            kind: TicketKind::Incident,
            body: "red".to_string(),
            size: None,
        })
        .await
        .expect("incident");

    // The wake loop ticks every 10 s; the forwarding test above sees its copy within that.
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    let got = orch
        .list_messages(&MessageQuery {
            to: Some("external:orchestrator".to_string()),
            ..Default::default()
        })
        .await
        .expect("messages");
    assert!(got.is_empty(), "{got:?}");
}
#[tokio::test]
async fn the_sender_sees_a_message_queued_then_arrived_then_delivered() {
    let p = pair().await;
    let advisor = p.b.external_client("advisor").await;
    let sender = p.a.external_client("orchestrator").await;

    // Peer unreachable (wrong token): queued, and the status carries the outbox line.
    p.point_a_at_b(&"0".repeat(64));
    let q = sender
        .send_outbox(&out(&p.b_project, "external:advisor", "hello"))
        .await
        .expect("accepted");
    assert_eq!(q.state, "queued");
    assert_eq!(sender.outbox_entry(&q.id).await.expect("e").state, "queued");
    let status = sender.status().await.expect("status");
    assert_eq!(status.outbox.len(), 1, "{:?}", status.outbox);
    assert_eq!(status.outbox[0].project, p.b_project);
    assert_eq!(status.outbox[0].queued, 1);
    assert!(status.outbox[0].last_error.is_some());

    // Back up: stored on B, its recipient not yet woken.
    p.point_a_at_b(&p.peer_token);
    let a_peer =
        p.a.client
            .create_peer_token(&PeerTokenCreateRequest {
                machine: "m2".to_string(),
                replace: false,
            })
            .await
            .expect("peer token")
            .token;
    Client::new(p.a.running.url.clone(), Some(a_peer))
        .hello(&HelloRequest {
            daemon: p.b_project.clone(),
        })
        .await
        .expect("hello");
    let arrived = wait_for("arrival", || async {
        let e = sender.outbox_entry(&q.id).await.ok()?;
        (e.state != "queued").then_some(e)
    })
    .await;
    assert_eq!(arrived.state, "arrived", "{arrived:?}");
    assert!(sender.status().await.expect("status").outbox.is_empty());

    // The recipient takes it: delivered.
    advisor
        .list_messages(&MessageQuery {
            to: Some("me".to_string()),
            mark_read: true,
            ..Default::default()
        })
        .await
        .expect("read");
    let done = sender.outbox_entry(&q.id).await.expect("entry");
    assert_eq!(done.state, "delivered", "{done:?}");

    // Someone else's message is not yours to look up.
    let other = p.a.external_client("other").await;
    assert!(other.outbox_entry(&q.id).await.is_err());
}

/// B can reach A as machine `m2` (A mints the peer token; B's config and credentials point at
/// A). `Pair` already lets A reach B. Returns A's project.
async fn point_b_at_a(p: &Pair) -> String {
    let a_project = p.a.running.info.project.clone();
    let a_peer =
        p.a.client
            .create_peer_token(&PeerTokenCreateRequest {
                machine: "m2".to_string(),
                replace: false,
            })
            .await
            .expect("peer token")
            .token;
    let b_home = machine_home_dir(p._tmp_b.path());
    std::fs::create_dir_all(&b_home).expect("home");
    let a_port = p.a.running.url.rsplit(':').next().expect("port");
    std::fs::write(
        b_home.join("config.toml"),
        format!(
            "[machine]\nname = \"m2\"\n[machines]\nm1 = \"127.0.0.1\"\nm2 = \"127.0.0.1\"\n\
             [projects]\n{a_project} = {{ machine = \"m1\", port = {a_port} }}\n"
        ),
    )
    .expect("config");
    store_credential(
        &b_home.join("credentials.toml"),
        "peer",
        &a_project,
        &a_peer,
    )
    .expect("credentials");
    a_project
}

async fn task_updates(b: &TestDaemon, to: &str) -> Vec<String> {
    b.client
        .list_messages(&MessageQuery {
            to: Some(to.to_string()),
            ..Default::default()
        })
        .await
        .expect("messages")
        .into_iter()
        .filter(|m| m.kind == MessageKind::TaskUpdate)
        .map(|m| m.body)
        .collect()
}

#[tokio::test]
async fn a_principal_of_one_project_watches_a_task_of_another_through_the_peer_token() {
    let p = pair().await;
    let a_project = point_b_at_a(&p).await;
    let advisor_b = p.b.external_client("advisor").await;
    let task =
        p.a.client
            .new_task(&NewTicketRequest {
                ticket: None,
                parent: None,
                for_human: false,
                priority: None,
                components: Vec::new(),
                title: "cross watch".to_string(),
                kind: TicketKind::Feature,
                body: "b".to_string(),
                size: None,
            })
            .await
            .expect("task");
    let watch = |watch: bool| RemoteWatchRequest {
        project: a_project.clone(),
        task: task.id.clone(),
        watch,
    };

    let watched = advisor_b.remote_watch(&watch(true)).await.expect("watch");
    assert!(
        watched
            .watchers
            .iter()
            .any(|w| w == &format!("remote:external:advisor@{}", p.b_project)),
        "{:?}",
        watched.watchers
    );

    // A change on A arrives in B's inbox once.
    p.a.client.note_task(&task.id, "first").await.expect("note");
    let got = wait_for("a task update on B", || async {
        let u = task_updates(&p.b, "external:advisor").await;
        (!u.is_empty()).then_some(u)
    })
    .await;
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].contains("first"), "{got:?}");

    // B unreachable (A holds a wrong token for it): the update waits in A's outbox, and a later
    // one queues behind it; both arrive, in order, once B is reachable again.
    p.point_a_at_b(&"0".repeat(64));
    p.a.client
        .note_task(&task.id, "second")
        .await
        .expect("note");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    assert_eq!(task_updates(&p.b, "external:advisor").await.len(), 1);
    p.point_a_at_b(&p.peer_token);
    p.a.client.note_task(&task.id, "third").await.expect("note");
    let got = wait_for("queued task updates on B", || async {
        let u = task_updates(&p.b, "external:advisor").await;
        (u.len() >= 3).then_some(u)
    })
    .await;
    assert_eq!(got.len(), 3, "{got:?}");
    assert!(
        got[1].contains("second") && got[2].contains("third"),
        "{got:?}"
    );

    // Unwatching stops them.
    let unwatched = advisor_b
        .remote_watch(&watch(false))
        .await
        .expect("unwatch");
    assert!(unwatched.watchers.iter().all(|w| !w.starts_with("remote:")));
    p.a.client
        .note_task(&task.id, "fourth")
        .await
        .expect("note");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    assert_eq!(task_updates(&p.b, "external:advisor").await.len(), 3);

    // A task that does not exist is refused up front.
    let missing = RemoteWatchRequest {
        task: "br-nope".to_string(),
        ..watch(true)
    };
    assert!(advisor_b.remote_watch(&missing).await.is_err());
}

#[tokio::test]
async fn a_visitor_cannot_use_the_cross_project_watch() {
    let p = pair().await;
    let a_project = point_b_at_a(&p).await;
    let visitor =
        p.b.client
            .create_token(&TokenCreateRequest {
                name: "orchestrator".to_string(),
                machine: Some("m1".to_string()),
                home: Some(a_project.clone()),
            })
            .await
            .expect("visitor");
    let visitor = Client::new(p.b.running.url.clone(), Some(visitor.token));
    let err = visitor
        .remote_watch(&RemoteWatchRequest {
            project: a_project.clone(),
            task: "br-x".to_string(),
            watch: true,
        })
        .await
        .expect_err("a visitor may not");
    assert!(err.to_string().contains("forbidden"), "{err}");

    // Only a peer token registers a watch on someone's behalf.
    let ordinary = p.a.external_client("someone").await;
    let err = ordinary
        .peer_watch(&PeerWatchRequest {
            task: "br-x".to_string(),
            who: "agent:w1".to_string(),
            home: p.b_project.clone(),
            watch: true,
        })
        .await
        .expect_err("not a peer");
    assert!(err.to_string().contains("forbidden"), "{err}");
}

/// br-4cgx: `@project:role` in a ticket comment goes to that project's role through the outbox,
/// in the short and the explicit `external:` form alike; an unknown project is reported back.
#[tokio::test]
async fn a_project_qualified_mention_reaches_the_role_on_that_project() {
    let p = pair().await;
    let orch_b = p.b.external_client("orchestrator").await;
    let task =
        p.a.client
            .new_open_task(&NewTicketRequest {
                ticket: None,
                parent: None,
                title: "t".to_string(),
                kind: TicketKind::Feature,
                body: String::new(),
                size: None,
                components: Vec::new(),
                for_human: false,
                priority: None,
            })
            .await
            .expect("new task");
    let b = &p.b_project;
    p.a.client
        .note_task(
            &task.id,
            &format!("@{b}:orchestrator hi, also @{b}:external:orchestrator and @nowhere:aide"),
        )
        .await
        .expect("note");
    let got = wait_for("forwarded mention", || async {
        let msgs = orch_b
            .list_messages(&MessageQuery {
                to: Some("me".to_string()),
                ..Default::default()
            })
            .await
            .expect("messages");
        (!msgs.is_empty()).then_some(msgs)
    })
    .await;
    // Both spellings are one recipient: one message.
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(got[0].from.starts_with("human@"), "{got:?}");
    assert!(got[0].body.contains("mentioned you"), "{got:?}");
    let own = inbox_bodies(&p.a, "me").await;
    assert!(
        own.iter()
            .any(|(_, b)| b.contains("no such recipient for @nowhere:aide")),
        "{own:?}"
    );
}

fn reply(body: &str, reply_to: &str) -> bridle_api::types::SendRequest {
    bridle_api::types::SendRequest {
        to: None,
        body: body.to_string(),
        kind: MessageKind::Note,
        when: When::Now,
        reply_to: Some(reply_to.to_string()),
        task: None,
    }
}

/// br-eupm: an aide on A writes to B's orchestrator; the orchestrator answers with `reply_to`
/// and no recipient, and the answer lands in that aide's inbox on A, not on B.
#[tokio::test]
async fn a_reply_goes_back_to_the_sender_on_the_daemon_it_sent_from() {
    let p = pair().await;
    let a_project = point_b_at_a(&p).await;
    let aide = p.a.external_client("aide").await;
    let orch = p.b.external_client("orchestrator").await;
    // A same-named principal on B: the bare-role reply the bug sent there must not be used.
    let b_aide = p.b.external_client("aide").await;

    aide.send_outbox(&out(&p.b_project, "external:orchestrator", "question"))
        .await
        .expect("queued");
    let asked = wait_for_bodies(&p.b, "external:orchestrator", 1).await;
    assert_eq!(asked[0].0, "external:aide@m1");
    let original = orch
        .list_messages(&MessageQuery {
            to: Some("me".to_string()),
            ..Default::default()
        })
        .await
        .expect("inbox")
        .remove(0);

    let sent = orch
        .send(&reply("answer", &original.id))
        .await
        .expect("routed");
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].to, "external:aide");

    let got = wait_for("the reply on A", || async {
        let m = aide
            .list_messages(&MessageQuery {
                to: Some("me".to_string()),
                ..Default::default()
            })
            .await
            .ok()?;
        (!m.is_empty()).then_some(m)
    })
    .await;
    assert_eq!(got[0].body, "answer");
    assert_eq!(got[0].from, "external:orchestrator@m2");
    assert_eq!(got[0].reply_to.as_deref(), Some(original.id.as_str()));
    assert!(
        inbox_bodies(&p.b, "external:aide").await.is_empty(),
        "nothing for B's own aide: {a_project}"
    );
    drop(b_aide);
}

#[tokio::test]
async fn a_reply_that_cannot_be_routed_fails_with_the_fix() {
    let p = pair().await;
    // B knows nothing of A (no point_b_at_a): the sender's daemon can't be reached.
    let aide = p.a.external_client("aide").await;
    let orch = p.b.external_client("orchestrator").await;
    aide.send_outbox(&out(&p.b_project, "external:orchestrator", "question"))
        .await
        .expect("queued");
    wait_for_bodies(&p.b, "external:orchestrator", 1).await;
    let original = orch
        .list_messages(&MessageQuery {
            to: Some("me".to_string()),
            ..Default::default()
        })
        .await
        .expect("inbox")
        .remove(0);

    let err = orch
        .send(&reply("answer", &original.id))
        .await
        .expect_err("unroutable")
        .to_string();
    assert!(err.contains("can't route the reply"), "{err}");
    assert!(err.contains("bridle send --project"), "{err}");

    let err = orch
        .send(&reply("answer", "m-nope"))
        .await
        .expect_err("unknown")
        .to_string();
    assert!(err.contains("no message m-nope to reply to"), "{err}");
}

/// br-jpph: the sender is another machine's orchestrator (the case that was reported failing):
/// a bare `reply_to` answers it on its own daemon, in the orchestrator's inbox there.
#[tokio::test]
async fn a_reply_to_another_machines_orchestrator_goes_back_to_it() {
    let p = pair().await;
    point_b_at_a(&p).await;
    let orch_a = p.a.external_client("orchestrator").await;
    let orch_b = p.b.external_client("orchestrator").await;

    orch_a
        .send_outbox(&out(&p.b_project, "external:orchestrator", "question"))
        .await
        .expect("queued");
    let asked = wait_for_bodies(&p.b, "external:orchestrator", 1).await;
    assert_eq!(asked[0].0, "external:orchestrator@m1");
    let original = orch_b
        .list_messages(&MessageQuery {
            to: Some("me".to_string()),
            ..Default::default()
        })
        .await
        .expect("inbox")
        .remove(0);

    let sent = orch_b
        .send(&reply("answer", &original.id))
        .await
        .expect("routed");
    assert_eq!(sent[0].to, "external:orchestrator");
    let got = wait_for_bodies(&p.a, "external:orchestrator", 1).await;
    assert_eq!(
        got[0],
        ("external:orchestrator@m2".to_string(), "answer".to_string())
    );
}
