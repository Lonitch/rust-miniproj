use crate::cmdline::executable::Executable;
use rustyline::completion::{Completer, Pair};
use rustyline::history::{History, SearchDirection};
use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

// Global state for TAB completion
thread_local! {
    static LAST_WORD: RefCell<Option<String>> = RefCell::new(None);
    static TAB_COUNT: RefCell<usize> = RefCell::new(0);
}

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
            let mut seen_commands = std::collections::HashSet::new();

            // Add builtin commands
            let builtin_commands = self.get_builtin_commands();
            for cmd in builtin_commands.iter().filter(|cmd| cmd.starts_with(word)) {
                seen_commands.insert(cmd.clone());
                matches.push(Pair {
                    display: cmd.clone(),
                    replacement: format!("{} ", cmd),
                });
            }

            // Add executables from PATH
            let path_executables = self.get_executables_from_path();
            for cmd in path_executables.iter().filter(|cmd| cmd.starts_with(word)) {
                // Skip if already added as a builtin or already seen
                if !seen_commands.contains(cmd) {
                    seen_commands.insert(cmd.clone());
                    matches.push(Pair {
                        display: cmd.clone(),
                        replacement: format!("{} ", cmd),
                    });
                }
            }
            
            // Sort matches alphabetically
            matches.sort_by(|a, b| a.display.cmp(&b.display));

            // When we have multiple matches, find the longest common prefix
            if matches.len() > 1 {
                let display_values: Vec<&str> = matches.iter().map(|pair| pair.display.as_str()).collect();
                if let Some(common_prefix) = find_longest_common_prefix(&display_values) {
                    // If the common prefix is longer than what the user typed
                    if common_prefix.len() > word.len() {
                        // Check if there are any further common prefixes by checking if
                        // there exists a match that starts with the common prefix but is longer
                        let has_further_common_prefix = display_values.iter()
                            .any(|val| val.len() > common_prefix.len() && val.starts_with(&common_prefix));
                        
                        // Create match with appropriate replacement
                        let new_match = Pair {
                            display: common_prefix.clone(),
                            replacement: if has_further_common_prefix {
                                // If there's more to complete, don't add a space
                                common_prefix.clone()
                            } else {
                                // No more common prefix - add space
                                format!("{} ", common_prefix)
                            },
                        };
                        
                        // Reset tab count if we've added a space (no more completions)
                        if !has_further_common_prefix {
                            LAST_WORD.with(|last_word_cell| {
                                TAB_COUNT.with(|tab_count_cell| {
                                    *last_word_cell.borrow_mut() = None;
                                    *tab_count_cell.borrow_mut() = 0;
                                });
                            });
                        }
                        
                        return Ok((word_start, vec![new_match]));
                    }
                }

                // If we get here, we couldn't complete further with common prefix
                // or we already completed to the longest common prefix
                let mut should_reset = false;
                let mut should_list = false;

                LAST_WORD.with(|last_word_cell| {
                    TAB_COUNT.with(|tab_count_cell| {
                        let mut last_word = last_word_cell.borrow_mut();
                        let mut tab_count = tab_count_cell.borrow_mut();

                        // Check if this is a repeated TAB press for the same word
                        if let Some(ref stored_word) = *last_word {
                            if stored_word == word {
                                *tab_count += 1;

                                // First TAB press - ring the bell
                                if *tab_count == 1 {
                                    print!("\x07"); // Bell character
                                    io::stdout().flush().unwrap_or(());
                                    should_reset = false;
                                }
                                // Second TAB press - display all matches
                                else if *tab_count == 2 {
                                    should_list = true;
                                    should_reset = true;
                                }
                            } else {
                                // Different word, reset counter
                                *last_word = Some(word.to_string());
                                *tab_count = 1;
                                print!("\x07"); // Bell character
                                io::stdout().flush().unwrap_or(());
                                should_reset = false;
                            }
                        } else {
                            // First time seeing this word
                            *last_word = Some(word.to_string());
                            *tab_count = 1;
                            print!("\x07"); // Bell character
                            io::stdout().flush().unwrap_or(());
                            should_reset = false;
                        }
                    });
                });

                if should_list {
                    println!();
                    let display_matches: Vec<String> =
                        matches.iter().map(|pair| pair.display.clone()).collect();
                    println!("{}", display_matches.join("  "));
                    print!("$ {}", line);
                    io::stdout().flush().unwrap_or(());

                    // Reset the tab count after displaying
                    LAST_WORD.with(|last_word_cell| {
                        TAB_COUNT.with(|tab_count_cell| {
                            *last_word_cell.borrow_mut() = None;
                            *tab_count_cell.borrow_mut() = 0;
                        });
                    });

                    return Ok((word_start, vec![]));
                }

                if !should_reset {
                    return Ok((word_start, vec![]));
                }
            } else if matches.len() == 1 {
                // Single match, reset counter
                LAST_WORD.with(|last_word_cell| {
                    TAB_COUNT.with(|tab_count_cell| {
                        *last_word_cell.borrow_mut() = None;
                        *tab_count_cell.borrow_mut() = 0;
                    });
                });
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

/// Finds the longest common prefix among a collection of strings.
/// Returns None if the collection is empty.
fn find_longest_common_prefix(strings: &[&str]) -> Option<String> {
    if strings.is_empty() {
        return None;
    }
    if strings.len() == 1 {
        return Some(strings[0].to_string());
    }

    // Start with the first string as the initial prefix
    let mut prefix = strings[0].to_string();

    // Compare with each remaining string
    for s in &strings[1..] {
        // Find the common characters between current prefix and this string
        let mut common_chars = Vec::new();
        
        for (a, b) in prefix.chars().zip(s.chars()) {
            if a != b {
                break;
            }
            common_chars.push(a);
        }
        
        // Update prefix to the common part only
        prefix = common_chars.into_iter().collect();
        
        // If no common prefix, exit early
        if prefix.is_empty() {
            return None;
        }
    }

    Some(prefix)
}
