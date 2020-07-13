use crate::irust::{IRust, IRustError};
use crossterm::style::Color;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Options {
    add_irust_cmd_to_history: bool,
    add_shell_cmd_to_history: bool,
    pub ok_color: Color,
    pub eval_color: Color,
    pub irust_color: Color,
    pub irust_warn_color: Color,
    pub out_color: Color,
    pub shell_color: Color,
    pub err_color: Color,
    pub input_color: Color,
    pub insert_color: Color,
    pub welcome_msg: String,
    pub welcome_color: Color,
    pub racer_inline_suggestion_color: Color,
    pub racer_suggestions_table_color: Color,
    pub racer_selected_suggestion_color: Color,
    pub racer_max_suggestions: usize,
    pub first_irust_run: bool,
    pub enable_racer: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            // [Histroy]
            add_irust_cmd_to_history: true,
            add_shell_cmd_to_history: false,

            // [Colors]
            ok_color: Color::Blue,
            eval_color: Color::White,
            irust_color: Color::DarkBlue,
            irust_warn_color: Color::Cyan,
            out_color: Color::Red,
            shell_color: Color::DarkYellow,
            err_color: Color::DarkRed,
            input_color: Color::Yellow,
            insert_color: Color::White,

            // [Welcome]
            welcome_msg: String::new(),
            welcome_color: Color::DarkBlue,

            // [Racer]
            enable_racer: true,
            racer_inline_suggestion_color: Color::Cyan,
            racer_suggestions_table_color: Color::Green,
            racer_selected_suggestion_color: Color::DarkRed,
            racer_max_suggestions: 5,

            //other
            first_irust_run: true,
        }
    }
}

impl Options {
    pub fn save(&mut self) -> Result<(), IRustError> {
        if let Some(path) = Self::config_path() {
            Self::write_config_file(path, &self)?;
        }
        Ok(())
    }

    pub fn new() -> Result<Self, IRustError> {
        if let Some(config_path) = Options::config_path() {
            match std::fs::File::open(&config_path) {
                Ok(mut config_file) => {
                    let mut config_data = String::new();
                    config_file.read_to_string(&mut config_data)?;

                    toml::from_str(&config_data).map_err(|e| e.into())
                }
                Err(_) => Options::reset_config(config_path),
            }
        } else {
            Ok(Options::default())
        }
    }

    pub fn reset_config(config_path: std::path::PathBuf) -> Result<Self, IRustError> {
        let default = Options::default();
        Options::write_config_file(config_path, &default)?;
        Ok(default)
    }

    pub fn config_path() -> Option<std::path::PathBuf> {
        let config_dir = match dirs_next::config_dir() {
            Some(dir) => dir.join("irust"),
            None => return None,
        };

        let _ = std::fs::create_dir(&config_dir);
        let config_path = config_dir.join("config");

        Some(config_path)
    }

    fn write_config_file(
        config_path: std::path::PathBuf,
        options: &Options,
    ) -> Result<(), IRustError> {
        let config = toml::to_string(options)?;

        let mut config_file = std::fs::File::create(&config_path)?;

        write!(config_file, "{}", config)?;
        Ok(())
    }
}

impl IRust {
    pub fn should_push_to_history(&self, buffer: &str) -> bool {
        let buffer: Vec<char> = buffer.chars().collect();

        if buffer.is_empty() {
            return false;
        }
        if buffer.len() == 1 {
            return buffer[0] != ':';
        }

        let irust_cmd = buffer[0] == ':' && buffer[1] != ':';
        let shell_cmd = buffer[0] == ':' && buffer[1] == ':';

        (irust_cmd && self.options.add_irust_cmd_to_history)
            || (shell_cmd && self.options.add_shell_cmd_to_history)
            || (!irust_cmd && !shell_cmd)
    }
}
