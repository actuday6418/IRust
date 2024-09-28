fn _insert_at_char_idx(buffer: &mut String, idx: usize, character: char) {
    let mut buffer_chars: Vec<char> = buffer.chars().collect();
    buffer_chars.insert(idx, character);
    *buffer = buffer_chars.into_iter().collect();
}

fn _remove_at_char_idx(buffer: &mut String, idx: usize) -> Option<char> {
    let mut buffer_chars: Vec<char> = buffer.chars().collect();

    let removed_char = if buffer_chars.len() > idx {
        Some(buffer_chars.remove(idx))
    } else {
        None
    };

    *buffer = buffer_chars.into_iter().collect();

    removed_char
}

fn _is_multiline(string: &str) -> bool {
    string.chars().filter(|c| *c == '\n').count() > 1
}

fn remove_comments(s: &str) -> String {
    s.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .map(|l| {
            let mut quote = false;
            let mut d_quote = false;

            let mut l = l.chars().peekable();
            let mut purged_line = String::new();

            loop {
                match (l.next(), l.peek()) {
                    (Some('/'), Some('/')) => {
                        if !quote && !d_quote {
                            break;
                        }
                    }
                    (Some('\''), _) => {
                        quote = !quote;
                        purged_line.push('\'');
                    }
                    (Some('"'), _) => {
                        d_quote = !d_quote;
                        purged_line.push('"');
                    }
                    (Some(c), _) => purged_line.push(c),
                    _ => break,
                }
            }
            purged_line + "\n"
        })
        .collect()
}

pub fn unmatched_brackets(s: &str) -> bool {
    let s = remove_comments(s);
    let mut braces = std::collections::HashMap::new();
    braces.insert('(', 0);
    braces.insert('[', 0);
    braces.insert('{', 0);

    let mut quote = false;
    let mut double_quote = false;
    let mut previous_char = ' ';
    for character in s.chars() {
        // safe unwraps ahead
        match character {
            '(' => {
                if !quote && !double_quote {
                    *braces.get_mut(&'(').unwrap() += 1;
                }
            }
            ')' => {
                if !quote && !double_quote {
                    *braces.get_mut(&'(').unwrap() -= 1;
                }
            }
            '[' => {
                if !quote && !double_quote {
                    *braces.get_mut(&'[').unwrap() += 1;
                }
            }
            ']' => {
                if !quote && !double_quote {
                    *braces.get_mut(&'[').unwrap() -= 1;
                }
            }
            '{' => {
                if !quote && !double_quote {
                    *braces.get_mut(&'{').unwrap() += 1;
                }
            }
            '}' => {
                if !quote && !double_quote {
                    *braces.get_mut(&'{').unwrap() -= 1;
                }
            }
            '"' => {
                if previous_char != '\\' {
                    double_quote = !double_quote;
                }
            }
            '\'' => {
                if previous_char != '\\' {
                    quote = !quote;
                }
            }
            _ => (),
        }
        previous_char = character;
    }

    braces[&'('] != 0 || braces[&'['] != 0 || braces[&'{'] != 0
}
