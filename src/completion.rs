use crate::cmdline::executable::Executable;
use rustyline::completion::{Completer, Pair};
use rustyline::history::{History, SearchDirection};

pub struct ShellCompleter {
    history: Option<Box<dyn History>>,
}

impl ShellCompleter {
    pub fn new() -> Self {
        ShellCompleter { history: None }
    }

    pub fn set_history(&mut self, history: Box<dyn History>) {
        self.history = Some(history);
    }

    fn get_builtin_commands(&self) -> Vec<String> {
        Executable::get_builtin_str()
    }
}

impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (word_start, word) = find_word_at_pos(line, pos);
        let words: Vec<&str> = line[..word_start].split_whitespace().collect();

        // If we're at the first word, complete with built-in commands
        if words.is_empty() {
            let builtin_commands = self.get_builtin_commands();
            let matches: Vec<Pair> = builtin_commands
                .iter()
                .filter(|cmd| cmd.starts_with(word))
                .map(|cmd| Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                })
                .collect();

            return Ok((word_start, matches));
        }

        // If we're completing arguments and have history, use it for suggestions
        if let Some(ref history) = self.history {
            // Get the command (first word)
            let command = words[0];

            // Find history entries that start with the same command
            let mut suggestions = Vec::new();
            let mut seen_args = std::collections::HashSet::new();

            // Examine history entries using the correct API
            for i in 0..history.len() {
                // Use the correct get method signature with SearchDirection
                if let Ok(Some(search_result)) = history.get(i, SearchDirection::Forward) {
                    let entry = search_result.entry;

                    // Skip if this entry doesn't start with our command
                    if !entry.starts_with(command) {
                        continue;
                    }

                    // Split the history entry into words
                    let entry_words: Vec<&str> = entry.split_whitespace().collect();
                    if entry_words.len() <= words.len() {
                        continue;
                    }

                    // Check if the beginning of the history entry matches our current input
                    let mut matches = true;
                    for (i, &input_word) in words.iter().enumerate() {
                        if i >= entry_words.len() || input_word != entry_words[i] {
                            matches = false;
                            break;
                        }
                    }

                    if matches {
                        // Extract the next argument from history as a suggestion
                        let next_arg = entry_words[words.len()];
                        if next_arg.starts_with(word) && !seen_args.contains(next_arg) {
                            seen_args.insert(next_arg.to_string());
                            suggestions.push(Pair {
                                display: next_arg.to_string(),
                                replacement: next_arg.to_string(),
                            });
                        }
                    }
                }
            }

            // Return argument suggestions
            if !suggestions.is_empty() {
                return Ok((word_start, suggestions));
            }
        }

        Ok((word_start, vec![]))
    }
}

fn find_word_at_pos(line: &str, pos: usize) -> (usize, &str) {
    let pos = std::cmp::min(pos, line.len());

    let start = line[..pos]
        .rfind(|c: char| c.is_whitespace())
        .map(|i| i + 1)
        .unwrap_or(0);

    let end = line[pos..]
        .find(|c: char| c.is_whitespace())
        .map(|i| i + pos)
        .unwrap_or(line.len());

    (start, &line[start..end])
}
