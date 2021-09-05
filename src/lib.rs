//! TODOs you cannot forget!
//!
//! `todo-or-die` provides procedural macros that act as checked reminders.
//!
//! The name was shamelessly stolen from the ruby gem [`todo_or_die`][ruby].
//!
//! # Examples
//!
//! ```
//! // trigger a compile error if we're past a certain date
//! todo_or_die::after_date!(3000, 1, 1); // its the year 3000!
//!
//! // or a GitHub issue has closed
//! todo_or_die::issue_closed!("rust-lang", "rust", 44265); // GATs are here!
//!
//! // or the latest version of a crate matches some expression
//! todo_or_die::crates_io!("serde", ">1.0.9000"); // its over 9000!
//! ```
//!
//! # Skipping checks
//!
//! If the environment variable `TODO_OR_DIE_SKIP` is set all macros will do nothing and
//! immediately succeed. This can for example be used to skip checks locally and only perform them
//! on CI.
//!
//! # Feature flags
//!
//! The following optional features are available:
//!
//! - `crate`: Enables checking versions of crates.
//! - `github`: Enables checking if issues or pull requests are closed.
//! - `rust`: Enables checking the current rust version.
//! - `time`: Enables checking things to do with time.
//!
//! Note that _none_ of the features are enabled by default.
//!
//! [ruby]: https://rubygems.org/gems/todo_or_die

#![warn(
    clippy::all,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    missing_debug_implementations,
    missing_docs
)]
#![deny(unreachable_pub, private_in_public)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]

#[cfg(feature = "__internal_http")]
mod http;

#[cfg(feature = "github")]
mod github;

#[cfg(feature = "time")]
mod time;

#[cfg(feature = "crate")]
mod krate;

#[cfg(feature = "rust")]
mod rust;

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
/// todo_or_die::issue_closed!("tokio-rs", "axum", 1);
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
/// // todo_or_die::pr_closed!("tokio-rs", "axum", 266);
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
/// todo_or_die::after_date!(1994, 10, 22);
/// ```
#[cfg(feature = "time")]
#[proc_macro]
pub fn after_date(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    perform_check(input, time::after_date)
}

/// Trigger a compile error if the latest version of a crate hosted on crates.io matches some
/// expression.
///
/// Note that this will make network requests during compile which may make your builds flaky at
/// times.
///
/// Requires the `crate` feature to be enabled.
///
/// # Example
///
/// ```compile_fail
/// todo_or_die::crates_io!("tokio", ">=1.0");
/// ```
///
/// Any version requirement supported by [`semver::VersionReq::parse`] is supported.
///
/// [`semver::VersionReq::parse`]: https://docs.rs/semver/latest/semver/struct.VersionReq.html#method.parse
#[cfg(feature = "crate")]
#[proc_macro]
pub fn crates_io(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    perform_check(input, krate::crates_io)
}

/// Trigger a compile error if the currently used version of rust used matches some expression.
///
/// Note that release channels (like `nightly` or `beta`) are ignored.
///
/// Requires the `rust` feature to be enabled.
///
/// # Example
///
/// ```compile_fail
/// todo_or_die::rust_version!(">1.50");
/// ```
///
/// Any version requirement supported by [`semver::VersionReq::parse`] is supported.
///
/// [`semver::VersionReq::parse`]: https://docs.rs/semver/latest/semver/struct.VersionReq.html#method.parse
#[cfg(feature = "rust")]
#[proc_macro]
pub fn rust_version(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    perform_check(input, rust::rust_version)
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
                ::std::compile_error!(#err);
            }
            .into();
        }
    };

    match f(input) {
        Ok(None) => {}
        Ok(Some(msg)) => {
            return quote::quote! {
                ::std::compile_error!(#msg);
            }
            .into();
        }
        Err(err) => {
            eprintln!("something went wrong\n\n{:?}", err);
        }
    }

    Default::default()
}
