use chrono::Local;
use std::{borrow::Cow, cell::RefCell};

use inkjet::{
    constants::HIGHLIGHT_NAMES,
    theme::{vendored, Theme},
    tree_sitter_highlight::{Highlight, HighlightEvent},
    Highlighter as InkjetHighlighter,
};
use nu_ansi_term::Style;
use reedline::{
    EditCommand, FileBackedHistory, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus,
    Reedline, Signal, ValidationResult,
};
mod parsing;
use crossterm::style::Stylize;
use crossterm::terminal;
use irust_repl::{EvalResult, Repl as ReplEngine};
use parsing::{incomplete_input, input_is_cmd_or_shell, is_a_statement, parse_command, Command};

pub static DEFAULT_PROMPT_INDICATOR: &str = "In: ";
pub static DEFAULT_MULTILINE_INDICATOR: &str = "..: ";
pub static WELCOME_MESSAGE: &str = "Welcome to IRust";

fn wait(mut proc: std::process::Child) {
    loop {
        match proc.try_wait() {
            Ok(Some(_)) => {
                return;
            }
            Err(e) => println!("{e}"),
            _ => {}
        }
    }
}

struct Validator;
impl reedline::Validator for Validator {
    fn validate(&self, line: &str) -> ValidationResult {
        if incomplete_input(line) && !input_is_cmd_or_shell(line) {
            ValidationResult::Incomplete
        } else {
            ValidationResult::Complete
        }
    }
}

struct Highlighter {
    high: RefCell<InkjetHighlighter>,
    theme: Theme,
}

impl Highlighter {
    fn highlights_to_style(&self, highlights: &[Highlight]) -> Style {
        highlights
            .first()
            .map(|Highlight(highlight_index): &Highlight| {
                let color = self
                    .theme
                    .get_style(HIGHLIGHT_NAMES[*highlight_index])
                    .and_then(|s| s.fg)
                    .unwrap_or(self.theme.fg);
                Style::new().fg(nu_ansi_term::Color::Rgb(color.r, color.g, color.b))
            })
            .unwrap_or_default()
    }
}

fn get_now() -> String {
    let now = Local::now();
    format!("{:>}", now.format("%m/%d/%Y %I:%M:%S %p"))
}

struct Prompt {}
impl reedline::Prompt for Prompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Owned(String::from(""))
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Owned(get_now())
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<str> {
        DEFAULT_PROMPT_INDICATOR.into()
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed(DEFAULT_MULTILINE_INDICATOR)
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        // NOTE: magic strings, given there is logic on how these compose I am not sure if it
        // is worth extracting in to static constant
        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }
}

impl reedline::Highlighter for Highlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> reedline::StyledText {
        let mut active_highlights: Vec<Highlight> = Vec::new();
        let mut results: Vec<(Style, String)> = Vec::new();
        for highlight_event in self
            .high
            .borrow_mut()
            .highlight_raw(inkjet::Language::Rust, &line)
            .unwrap()
            .filter_map(|r| r.ok())
        {
            match highlight_event {
                HighlightEvent::HighlightStart(highlight) => active_highlights.push(highlight),
                HighlightEvent::HighlightEnd => {
                    active_highlights.pop();
                }
                HighlightEvent::Source { start, end } => results.push((
                    self.highlights_to_style(&active_highlights),
                    line[start..end].to_string(),
                )),
            }
        }
        reedline::StyledText { buffer: results }
    }
}

struct IRust {
    frontend: Reedline,
    repl_engine: ReplEngine,
    to_exit: bool,
}

impl IRust {
    fn start(&mut self) {
        let prompt = Prompt {};
        loop {
            match self.frontend.read_line(&prompt) {
                Ok(Signal::Success(buffer)) => {
                    self.exec_command(buffer.trim_end());
                    if self.to_exit {
                        break;
                    }
                }
                Ok(Signal::CtrlD) => {
                    println!("\nAborted!");
                    break;
                }
                Ok(Signal::CtrlC) => self.frontend.run_edit_commands(&[EditCommand::Clear]),
                _ => {}
            }
        }
    }
    fn exec_command(&mut self, cmd_trimmed: &str) {
        match parse_command(cmd_trimmed) {
            Command::Reset => self.repl_engine.reset().unwrap(),
            Command::Show => println!("{}", self.repl_engine.show()),
            Command::Pop => self.repl_engine.pop(),
            Command::Exit => self.to_exit = true,
            Command::Help(argument) => println!("farts are {}", argument),
            Command::Add(argument) => {
                wait(
                    self.repl_engine
                        .add_dep(
                            argument
                                .split_whitespace()
                                .map(String::from)
                                .collect::<Vec<_>>()
                                .as_ref(),
                        )
                        .unwrap(),
                );
                self.repl_engine.build().unwrap();
                self.repl_engine
                    .cargo
                    .cargo_check(irust_repl::ToolChain::Stable)
                    .unwrap();
            }
            Command::Rust(code) => self.exec_rust(&code),
        }
    }
    fn exec_rust(&mut self, cmd_trimmed: &str) {
        if cmd_trimmed.chars().last() == Some(';') || is_a_statement(cmd_trimmed) {
            let check_result = self
                .repl_engine
                .eval_check(cmd_trimmed.to_string())
                .unwrap();
            if !check_result.status.success() {
                println!("{}", check_result.output);
            } else {
                self.repl_engine.insert(cmd_trimmed)
            }
        } else if !cmd_trimmed.is_empty() {
            let EvalResult { output, status: _ } = self
                .repl_engine
                .eval_with_configuration(irust_repl::EvalConfig {
                    input: cmd_trimmed,
                    interactive_function: None,
                    color: true,
                    evaluator: &["println!(\"{:?}\", {\n".into(), "\n});".into()],
                    compile_mode: irust_repl::CompileMode::Debug,
                })
                .unwrap();

            if output.trim() != "()" {
                print!("{output}");
            }
        }
    }
}

impl Default for IRust {
    fn default() -> Self {
        let history = Box::new(
            FileBackedHistory::with_file(500, "his.txt".into())
                .expect("Error configuring history with file"),
        );
        let highlighter = Highlighter {
            high: InkjetHighlighter::new().into(),
            theme: Theme::from_helix(vendored::GRUVBOX).unwrap(),
        };
        Self {
            frontend: Reedline::create()
                .with_history(history)
                .use_bracketed_paste(true)
                .with_validator(Box::new(Validator {}))
                .with_highlighter(Box::new(highlighter)),
            repl_engine: ReplEngine::default(),
            to_exit: false,
        }
    }
}

fn main() {
    let bounds = "-"
        .repeat((terminal::size().unwrap().0 as usize - WELCOME_MESSAGE.len()) / 2)
        .blue();
    print!("{}{}{}\n\n", bounds, WELCOME_MESSAGE.blue(), bounds);
    IRust::default().start();
}
