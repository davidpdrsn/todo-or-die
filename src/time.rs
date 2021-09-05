use anyhow::Result;
use chrono::prelude::*;

pub fn after(input: syn::LitStr) -> Result<Option<String>> {
    let input = input.value().parse::<NaiveDate>()?;
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

/// ```compile_fail
/// todo_or_die::after!("1990-01-01");
/// ```
///
/// ```
/// todo_or_die::after!("3000-01-01");
/// ```
#[allow(dead_code)]
fn tests() {}
