//! Note that this will make network requests during compile which may make your builds flaky at
//! times.
//!
//! If the environment variable `TODO_OR_DIE_SKIP` is set all macros will do nothing and
//! immediately succeed.

use anyhow::Result;
use hyper::{
    client::{connect::dns::GaiResolver, HttpConnector},
    Body, Client,
};
use hyper_rustls::HttpsConnector;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

mod github;

/// Trigger a compile error if an issue has been closed.
///
/// # Example
///
/// ```compile_fail
/// todo_or_die::issue_closed!("tokio-rs/axum#1");
/// ```
///
/// # Authentication
///
/// `issue_closed` will first look for the environment variable `TODO_OR_DIE_GITHUB_TOKEN` and then
/// `GITHUB_TOKEN`, if either are found its value will be used as the auth token when making
/// requests to the GitHub API. This allows you to access private repo and get more generous
/// rate limits.
#[proc_macro]
pub fn issue_closed(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    perform_check(input, github::issue_closed)
}

/// Trigger a compile error if a pull request has been closed or merged.
///
/// # Example
///
/// ```compile_fail
/// todo_or_die::pr_closed!("tokio-rs/axum#266");
/// ```
///
/// # Authentication
///
/// `pr_closed` will first look for the environment variable `TODO_OR_DIE_GITHUB_TOKEN` and then
/// `GITHUB_TOKEN`, if either are found its value will be used as the auth token when making
/// requests to the GitHub API. This allows you to access private repo and get more generous
/// rate limits.
#[proc_macro]
pub fn pr_closed(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    perform_check(input, github::pr_closed)
}

fn perform_check<F, T>(input: proc_macro::TokenStream, f: F) -> proc_macro::TokenStream
where
    F: FnOnce(T) -> Result<Option<String>>,
    T: syn::parse::Parse,
{
    if std::env::var("TODO_OR_DIE_SKIP").is_ok() {
        return Default::default();
    }

    let input = match syn::parse::<T>(input) {
        Ok(value) => value,
        Err(err) => {
            let err = err.to_string();
            return quote::quote! {
                ::std::compile_error!(#err)
            }
            .into();
        }
    };

    match f(input) {
        Ok(None) => {}
        Ok(Some(msg)) => {
            return quote::quote! {
                ::std::compile_error!(#msg)
            }
            .into();
        }
        Err(err) => {
            eprintln!("something went wrong: {}", err);
        }
    }

    Default::default()
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
