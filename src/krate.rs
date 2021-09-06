use crate::http::request;
use anyhow::{Context as _, Result};
use hyper::Request;
use semver::{Version, VersionReq};
use serde::Deserialize;
use syn::parse::Parse;

pub(crate) fn crates_io(input: Input) -> Result<Option<String>> {
    #[derive(Debug, Deserialize)]
    struct Response {
        versions: Vec<CrateVersion>,
    }

    #[derive(Debug, Deserialize)]
    struct CrateVersion {
        num: String,
    }

    let data = request::<Response>(
        Request::builder()
            .uri(format!("https://crates.io/api/v1/crates/{}", input.krate))
            .body(())
            .unwrap(),
    )?;

    let latest_version = data
        .versions
        .first()
        .context("No versions found for crate")?
        .num
        .parse::<Version>()?;

    if input.version_req.matches(&latest_version) {
        Ok(Some(format!(
            "Latest version of {} is {}. Time to act on this!",
            input.krate, latest_version
        )))
    } else {
        Ok(None)
    }
}

pub(crate) struct Input {
    krate: String,
    version_req: VersionReq,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let krate = input.parse::<syn::LitStr>()?.value();

        input.parse::<syn::token::Comma>()?;

        let lit = input.parse::<syn::LitStr>()?;
        let version_req = lit
            .value()
            .parse()
            .map_err(|err| syn::Error::new(lit.span(), err))?;

        input.parse::<syn::token::Comma>().ok();

        Ok(Self { krate, version_req })
    }
}

/// ```compile_fail
/// todo_or_die::crates_io!("tokio", ">=1.0");
/// ```
///
/// ```
/// todo_or_die::crates_io!("tokio", ">=10.0");
/// ```
#[allow(dead_code)]
fn tests() {}
