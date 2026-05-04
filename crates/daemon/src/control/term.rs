use std::io::{IsTerminal, stdout};

const RESET: &str = "\u{1b}[0m";
const GREEN: &str = "\u{1b}[32m";
const CYAN: &str = "\u{1b}[36m";
const DIM: &str = "\u{1b}[2m";

pub(super) fn success(text: &str) -> String {
    paint(text, GREEN)
}

pub(super) fn accent(text: &str) -> String {
    paint(text, CYAN)
}

pub(super) fn dim(text: &str) -> String {
    paint(text, DIM)
}

fn paint(text: &str, color: &str) -> String {
    if colors_enabled() {
        format!("{color}{text}{RESET}")
    } else {
        text.to_string()
    }
}

fn colors_enabled() -> bool {
    stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}
