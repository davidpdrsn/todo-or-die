use anyhow::{Context as _, Result};
use hyper::{
    client::{connect::dns::GaiResolver, HttpConnector},
    header::HeaderValue,
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Body, Client, Request,
};
use hyper_rustls::HttpsConnector;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::str::FromStr;
use tokio::runtime::Runtime;

pub fn issue_closed(input: syn::LitStr) -> Result<Option<String>> {
    #[derive(Deserialize, Debug)]
    struct Issue {
        closed_at: Option<String>,
    }

    RUNTIME.block_on(async move {
        let OrgRepoIssue {
            org,
            repo,
            issue: issue_number,
        } = input.value().parse()?;

        let issue = request::<Issue>(
            Request::builder()
                .uri(format!(
                    "https://api.github.com/repos/{}/{}/issues/{}",
                    org, repo, issue_number
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await?;

        if issue.closed_at.is_some() {
            Ok(Some(format!(
                "{}/{}#{} is closed. Time to act on this!",
                org, repo, issue_number
            )))
        } else {
            Ok(None)
        }
    })
}

#[derive(Deserialize, Debug)]
struct PullRequest {
    state: String,
}

pub fn pr_closed(input: syn::LitStr) -> Result<Option<String>> {
    RUNTIME.block_on(async move {
        let OrgRepoIssue {
            org,
            repo,
            issue: issue_number,
        } = input.value().parse()?;

        let pr = request::<PullRequest>(
            Request::builder()
                .uri(format!(
                    "https://api.github.com/repos/{}/{}/pulls/{}",
                    org, repo, issue_number
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await?;

        if pr.state == "closed" {
            Ok(Some(format!(
                "{}/{}#{} is closed. Time to act on this!",
                org, repo, issue_number
            )))
        } else {
            Ok(None)
        }
    })
}

struct OrgRepoIssue {
    org: String,
    repo: String,
    issue: u64,
}

impl FromStr for OrgRepoIssue {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (org, rest) = input.split_once('/').ok_or_else(|| {
            anyhow::format_err!("Parse error. Input must be of the form `org/repo#issue`")
        })?;

        let (repo, rest) = rest.split_once('#').ok_or_else(|| {
            anyhow::format_err!("Parse error. Input must be of the form `org/repo#issue`")
        })?;

        let issue = rest.parse().map_err(|_| {
            anyhow::format_err!("Parse error. Input must be of the form `org/repo#issue`")
        })?;

        Ok(Self {
            org: org.to_string(),
            repo: repo.to_string(),
            issue,
        })
    }
}

async fn request<T>(mut request: Request<Body>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    request.headers_mut().insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github.v3+json"),
    );

    request
        .headers_mut()
        .insert(USER_AGENT, HeaderValue::from_static("todo-or-die"));

    if let Some(auth_token) = auth_token() {
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", auth_token))
                .context("GitHub auth token contained invalid header value")?,
        );
    }

    let response = http_client()
        .request(request)
        .await
        .context("HTTP request to GitHub API failed")?;

    let status = response.status();
    if !status.is_success() {
        let body = hyper::body::to_bytes(response)
            .await
            .context("Failed to read GitHub API response")?;
        let body = String::from_utf8_lossy(&body);
        anyhow::bail!(
            "Received non-success response from GitHub API. status={}, body={:?}",
            status,
            body
        );
    }

    let body = hyper::body::to_bytes(response)
        .await
        .context("Failed to read GitHub API response")?;
    let value =
        serde_json::from_slice::<T>(&body).context("Failed to parse GitHub API response")?;

    Ok(value)
}

fn auth_token() -> Option<String> {
    std::env::var("TODO_OR_DIE_GITHUB_TOKEN")
        .ok()
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
}

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime")
});

type HyperTlsClient = Client<HttpsConnector<HttpConnector<GaiResolver>>, Body>;

fn http_client() -> &'static HyperTlsClient {
    static CLIENT: Lazy<HyperTlsClient> = Lazy::new(|| {
        let mut tls = rustls::ClientConfig::new();
        tls.set_protocols(&["h2".into(), "http/1.1".into()]);
        tls.root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

        let mut http = hyper::client::HttpConnector::new();
        http.enforce_http(false);

        hyper::Client::builder().build::<_, Body>(hyper_rustls::HttpsConnector::from((http, tls)))
    });

    &*CLIENT
}

/// # `issue_closed`
///
/// closed issue
/// ```compile_fail
/// todo_or_die::issue_closed!("tokio-rs/axum#1");
/// ```
///
/// open issue
/// ```
/// // the oldest open rust-lang issue. Probably wont be close anytime soon :shrug:
/// todo_or_die::issue_closed!("rust-lang/rust#1563");
/// ```
///
/// # `pr_closed`
///
/// closed pr
/// ```compile_fail
/// todo_or_die::pr_closed!("tokio-rs/axum#266");
/// ```
///
/// merged pr
/// ```compile_fail
/// todo_or_die::pr_closed!("tokio-rs/axum#294");
/// ```
///
/// open pr
/// ```
/// todo_or_die::pr_closed!("davidpdrsn/keep#1");
/// ```
#[allow(dead_code)]
fn tests() {}
