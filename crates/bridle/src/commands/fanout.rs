//! `--all-projects`: ask every locally registered daemon the same read, and report the ones
//! that can't answer instead of dropping them (br-t7uq). Local registry only.

use super::*;
use std::future::Future;

/// One project's answer: its items, or why there are none.
#[derive(serde::Serialize)]
pub(crate) struct ProjectAnswer<T> {
    pub project: String,
    pub items: Vec<T>,
    pub error: Option<String>,
}

/// Run `f` against each daemon in `daemons`, concurrently, sorted by project name. A missing
/// token, a refused connection or a timeout becomes that project's `error`.
pub(crate) async fn fan_out<T, F, Fut>(
    daemons: Vec<bridle_api::DaemonInfo>,
    f: F,
) -> Vec<ProjectAnswer<T>>
where
    F: Fn(Client) -> Fut,
    Fut: Future<Output = Result<Vec<T>, String>>,
{
    let timeout = std::time::Duration::from_secs(5);
    let mut out = futures::future::join_all(daemons.into_iter().map(|d| {
        let client = discovery::resolve_token(
            None,
            Some(Path::new(&d.workspace)),
            Some(&d.project),
            None,
            &ProcessEnv,
            true,
        )
        .map(|token| Client::new_with_timeout(d.url.clone(), token, timeout))
        .map_err(|e| e.to_string());
        let fut = client.map(&f);
        async move {
            let res = match fut {
                Ok(fut) => fut.await,
                Err(e) => Err(e),
            };
            match res {
                Ok(items) => ProjectAnswer {
                    project: d.project,
                    items,
                    error: None,
                },
                Err(e) => ProjectAnswer {
                    project: d.project,
                    items: Vec::new(),
                    error: Some(e),
                },
            }
        }
    }))
    .await;
    out.sort_by(|a, b| a.project.cmp(&b.project));
    out
}

pub(crate) fn print_unreachable<T>(answers: &[ProjectAnswer<T>]) {
    for a in answers {
        if let Some(e) = &a.error {
            println!("{:<16} unreachable: {e}", a.project);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn info(project: &str, workspace: &str, url: String) -> bridle_api::DaemonInfo {
        bridle_api::DaemonInfo {
            project: project.into(),
            workspace: workspace.into(),
            repo: String::new(),
            url,
            pid: 1,
            started_at: Utc::now(),
            version: "t".into(),
            task_prefix: None,
        }
    }

    #[tokio::test]
    async fn one_unreachable_daemon_is_reported_not_skipped() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let live = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            loop {
                let (mut sock, _) = listener.accept().await.unwrap();
                let mut buf = [0u8; 2048];
                let _ = sock.read(&mut buf).await;
                let body = r#"{"ok":true,"version":"t","agent_count":3}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            }
        });
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let dead = format!("http://{}", l.local_addr().unwrap());
        drop(l);

        // The human token file is what a CLI outside an agent session resolves; without it
        // (CI has no $BRIDLE_TOKEN) the live daemon would report a missing token too.
        let ws = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(ws.path().join(".bridle/tokens")).unwrap();
        std::fs::write(ws.path().join(".bridle/tokens/human"), "tok\n").unwrap();
        let ws_path = ws.path().to_str().unwrap();

        let answers = fan_out(
            vec![
                info("zeta", "/nonexistent", dead),
                info("alpha", ws_path, live),
            ],
            |c| async move {
                let h = c.health().await.map_err(|e| e.to_string())?;
                Ok(vec![h.agent_count])
            },
        )
        .await;

        assert_eq!(answers.len(), 2);
        assert_eq!(answers[0].project, "alpha");
        assert_eq!(answers[0].items, vec![3], "{:?}", answers[0].error);
        assert!(answers[0].error.is_none());
        assert_eq!(answers[1].project, "zeta");
        assert!(answers[1].items.is_empty());
        assert!(answers[1].error.is_some());
    }
}
