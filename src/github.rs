use crate::http::request;
use syn::parse::Parse;
use anyhow::{Context as _, Result};
use hyper::{
    header::HeaderValue,
    header::{ACCEPT, AUTHORIZATION},
    Body, Request,
};
use serde::Deserialize;

pub(crate) fn issue_closed(input: OrgRepoIssue) -> Result<Option<String>> {
    #[derive(Deserialize, Debug)]
    struct Issue {
        closed_at: Option<String>,
    }

    let OrgRepoIssue {
        org,
        repo,
        issue: issue_number,
    } = input;

    let issue = request::<Issue>(github_request(
        Request::builder()
            .uri(format!(
                "https://api.github.com/repos/{}/{}/issues/{}",
                org, repo, issue_number
            ))
            .body(Body::empty())
            .unwrap(),
    )?)?;

    if issue.closed_at.is_some() {
        Ok(Some(format!(
            "{}/{}#{} is closed. Time to act on this!",
            org, repo, issue_number
        )))
    } else {
        Ok(None)
    }
}

#[derive(Deserialize, Debug)]
struct PullRequest {
    state: String,
}

pub(crate) fn pr_closed(input: OrgRepoIssue) -> Result<Option<String>> {
    let OrgRepoIssue {
        org,
        repo,
        issue: issue_number,
    } = input;

    let pr = request::<PullRequest>(github_request(
        Request::builder()
            .uri(format!(
                "https://api.github.com/repos/{}/{}/pulls/{}",
                org, repo, issue_number
            ))
            .body(Body::empty())
            .unwrap(),
    )?)?;

    if pr.state == "closed" {
        Ok(Some(format!(
            "{}/{}#{} is closed. Time to act on this!",
            org, repo, issue_number
        )))
    } else {
        Ok(None)
    }
}

fn github_request(mut request: Request<Body>) -> Result<Request<Body>> {
    request.headers_mut().insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github.v3+json"),
    );

    if let Some(auth_token) = auth_token() {
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", auth_token))
                .context("GitHub auth token contained invalid header value")?,
        );
    }

    Ok(request)
}

fn auth_token() -> Option<String> {
    std::env::var("TODO_OR_DIE_GITHUB_TOKEN")
        .ok()
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
}

pub(crate) struct OrgRepoIssue {
    org: String,
    repo: String,
    issue: u64,
}

impl Parse for OrgRepoIssue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let org = input.parse::<syn::LitStr>()?.value();
        input.parse::<syn::token::Comma>()?;

        let repo = input.parse::<syn::LitStr>()?.value();
        input.parse::<syn::token::Comma>()?;

        let issue = input.parse::<syn::LitInt>()?.base10_parse()?;

        input.parse::<syn::token::Comma>().ok();

        Ok(Self {
            org, repo, issue
        })
    }
}

/// # `issue_closed`
///
/// closed issue
/// ```compile_fail
/// todo_or_die::issue_closed!("tokio-rs", "axum", 1);
/// ```
///
/// open issue
/// ```
/// // the oldest open rust-lang issue. Probably wont be close anytime soon :shrug:
/// todo_or_die::issue_closed!("rust-lang", "rust", 1563);
/// ```
///
/// # `pr_closed`
///
/// closed pr
/// ```compile_fail
/// todo_or_die::pr_closed!("tokio-rs", "axum", 266);
/// ```
///
/// merged pr
/// ```compile_fail
/// todo_or_die::pr_closed!("tokio-rs", "axum", 294);
/// ```
///
/// open pr
/// ```
/// todo_or_die::pr_closed!("davidpdrsn", "keep", 1);
/// ```
#[allow(dead_code)]
fn tests() {}
