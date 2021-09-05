//! If the environment variable `TODO_OR_DIE_SKIP` is set all macros will do nothing and
//! immediately succeed.
//!
//! # Feature flags
//!
//! The following optional features are available:
//!
//! - `github`: Enables checking if issues or pull requests are closed.
//! - `time`: Enables checking things to do with time.
//!
//! Note that _none_ of the features are enabled by default.

#[cfg(feature = "github")]
mod github;

#[cfg(feature = "time")]
mod time;

/// Trigger a compile error if an issue has been closed.
///
/// Note that this will make network requests during compile which may make your builds flaky at
/// times.
///
/// Requires the `github` feature to be enabled.
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
/// requests to the GitHub API. This allows you to access private repos and get more generous
/// rate limits.
#[cfg(feature = "github")]
#[proc_macro]
pub fn issue_closed(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    perform_check(input, github::issue_closed)
}

/// Trigger a compile error if a pull request has been closed or merged.
///
/// Note that this will make network requests during compile which may make your builds flaky at
/// times.
///
/// Requires the `github` feature to be enabled.
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
/// requests to the GitHub API. This allows you to access private repos and get more generous
/// rate limits.
#[cfg(feature = "github")]
#[proc_macro]
pub fn pr_closed(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    perform_check(input, github::pr_closed)
}

/// Trigger a compile error if today is after the given date
///
/// Requires the `time` feature to be enabled.
///
/// # Example
///
/// ```compile_fail
/// todo_or_die::after!("1990-01-01");
/// ```
#[cfg(feature = "time")]
#[proc_macro]
pub fn after(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    perform_check(input, time::after)
}

#[allow(dead_code)]
fn perform_check<F, T>(input: proc_macro::TokenStream, f: F) -> proc_macro::TokenStream
where
    F: FnOnce(T) -> anyhow::Result<Option<String>>,
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
            eprintln!("something went wrong\n\n{:?}", err);
        }
    }

    Default::default()
}
