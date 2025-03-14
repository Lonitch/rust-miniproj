use crate::cmdline::executable::Executable;
use rustyline::completion::{Completer, Pair};
use rustyline::history::{History, SearchDirection};
use std::env;
use std::fs;
use std::path::Path;

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

    fn get_executables_from_path(&self) -> Vec<String> {
        let mut executables = Vec::new();
        let path_var = match env::var("PATH") {
            Ok(val) => val,
            Err(_) => return executables,
        };

        for path in path_var.split(':') {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.is_file() && self.is_executable(&path) {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            executables.push(name.to_string());
                        }
                    }
                }
            }
        }

        executables
    }

    fn is_executable(&self, path: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(path) {
                return metadata.permissions().mode() & 0o111 != 0;
            }
        }

        #[cfg(not(unix))]
        {
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                return ext == "exe" || ext == "bat" || ext == "cmd";
            }
        }

        false
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

        // If we're at the first word, complete with built-in commands and executables from PATH
        if words.is_empty() {
            let mut matches = Vec::new();

            // Add builtin commands
            let builtin_commands = self.get_builtin_commands();
            for cmd in builtin_commands.iter().filter(|cmd| cmd.starts_with(word)) {
                matches.push(Pair {
                    display: cmd.clone(),
                    replacement: format!("{} ", cmd),
                });
            }

            // Add executables from PATH
            let path_executables = self.get_executables_from_path();
            for cmd in path_executables.iter().filter(|cmd| cmd.starts_with(word)) {
                // Skip if already added as a builtin
                if !builtin_commands.contains(cmd) {
                    matches.push(Pair {
                        display: cmd.clone(),
                        replacement: format!("{} ", cmd),
                    });
                }
            }

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
