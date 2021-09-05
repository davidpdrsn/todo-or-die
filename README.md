# todo-or-die

`todo-or-die` provides procedural macros that act as checked reminders.

[![Build status](https://github.com/davidpdrsn/todo-or-die/actions/workflows/CI.yml/badge.svg?branch=main)](https://github.com/davidpdrsn/todo-or-die/actions/workflows/CI.yml)
[![Crates.io](https://img.shields.io/crates/v/todo-or-die)](https://crates.io/crates/todo-or-die)
[![Documentation](https://docs.rs/todo-or-die/badge.svg)](https://docs.rs/todo-or-die)

# Examples

```rust
// trigger a compile error if we're past a certain date
todo_or_die::after_date!(3000, 1, 1); // its the year 3000!

// or a GitHub issue has closed
todo_or_die::issue_closed!("rust-lang", "rust", 44265); // GATs are here!

// or the latest version of a crate matches some expression
todo_or_die::crates_io!("serde", ">1.0.9000"); // its over 9000!
```

More information about this crate can be found in the [crate documentation][docs].

## License

This project is licensed under the [MIT license](LICENSE).

[docs](https://docs.rs/todo-or-die)
