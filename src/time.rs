use anyhow::Result;
use chrono::prelude::*;
use syn::parse::Parse;

pub(crate) fn after_date(input: Input) -> Result<Option<String>> {
    let input = NaiveDate::from_ymd(input.year, input.month, input.day);
    let today = Local::today().naive_local();

    if input <= today {
        Ok(Some(format!(
            "{} is now in the past. Time to act on this!",
            input
        )))
    } else {
        Ok(None)
    }
}

pub(crate) struct Input {
    year: i32,
    month: u32,
    day: u32,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let year = input.parse::<syn::LitInt>()?.base10_parse()?;
        input.parse::<syn::token::Comma>()?;

        let month = input.parse::<syn::LitInt>()?.base10_parse()?;
        input.parse::<syn::token::Comma>()?;

        let day = input.parse::<syn::LitInt>()?.base10_parse()?;

        input.parse::<syn::token::Comma>().ok();

        Ok(Self { year, month, day })
    }
}

/// ```compile_fail
/// todo_or_die::after_date!(1990, 01, 01);
/// ```
///
/// ```
/// todo_or_die::after_date!(3000, 01, 01);
/// ```
#[allow(dead_code)]
fn tests() {}
