mod utils;
use utils::unmatched_brackets;

pub enum Command {
    Reset,
    Show,
    Pop,
    Exit,
    Add(String),
    Help(String),
    Rust(String),
}

pub fn parse_command(cmd_trimmed: &str) -> Command {
    match cmd_trimmed {
        ":reset" => Command::Reset,
        ":show" => Command::Show,
        ":pop" => Command::Pop,
        ":exit" => Command::Exit,
        _ if cmd_trimmed.starts_with(":help") => {
            Command::Help(cmd_trimmed[5..].trim_start().to_string())
        }
        _ if cmd_trimmed.starts_with(":add") => {
            Command::Add(cmd_trimmed[5..].trim_start().to_string())
        }
        _ => Command::Rust(cmd_trimmed.to_string()),
    }
}

pub fn incomplete_input(buffer: &str) -> bool {
    unmatched_brackets(buffer) || buffer.trim_end().ends_with([':', '.', '='])
}

pub fn input_is_cmd_or_shell(buffer: &str) -> bool {
    buffer.starts_with(':') || buffer.starts_with("::")
}

pub fn is_a_statement(buffer_trimmed: &str) -> bool {
    match buffer_trimmed
        .split_whitespace()
        .collect::<Vec<_>>()
        .as_slice()
    {
        // async fn|const fn|unsafe fn
        [_, "fn", ..]
        | ["fn", ..]
        | ["enum", ..]
        | ["struct", ..]
        | ["trait", ..]
        | ["impl", ..]
        | ["pub", ..]
        | ["extern", ..]
        | ["macro", ..] => true,
        ["macro_rules!", ..] => true,
        // attribute exp:
        // #[derive(Debug)]
        // struct B{}
        [tag, ..] if tag.starts_with('#') => true,
        _ => false,
    }
}
