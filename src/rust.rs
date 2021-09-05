use anyhow::{Context as _, Result};
use semver::VersionReq;
use syn::parse::Parse;
use version_check::Version;

pub(crate) fn rust_version(input: Input) -> Result<Option<String>> {
    let current_version = Version::read()
        .context("Unable to get current rust version")?
        .to_string()
        .parse::<semver::Version>()
        .context("Couldn't parse rust version")?;

    if input.version_req.matches(&current_version) {
        Ok(Some(format!(
            "Your active version of rust is {}. Time to act on this!",
            current_version
        )))
    } else {
        Ok(None)
    }
}

pub(crate) struct Input {
    version_req: VersionReq,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit = input.parse::<syn::LitStr>()?;
        let version_req = lit
            .value()
            .parse()
            .map_err(|err| syn::Error::new(lit.span(), err))?;

        input.parse::<syn::token::Comma>().ok();

        Ok(Self { version_req })
    }
}

/// ```compile_fail
/// todo_or_die::rust_version!(">1.50");
/// ```
///
/// ```
/// todo_or_die::rust_version!("=2.0.0");
/// ```
#[allow(dead_code)]
fn tests() {}
