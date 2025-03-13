use crate::cmdline::Cmd;
use crate::completion::ShellCompleter;
use rustyline::completion::{Completer, Pair};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::{MatchingBracketValidator, Validator};
use rustyline::{CompletionType, Config, EditMode, Editor, Helper};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

struct ShellHelper {
    completer: Rc<RefCell<ShellCompleter>>,
    validator: MatchingBracketValidator,
}

impl Completer for ShellHelper {
    type Candidate = Pair; // Assume ShellCompleter returns Pair

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> std::result::Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        self.completer.borrow().complete(line, pos, ctx)
    }
}

impl Hinter for ShellHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        None
    }
}

// Basic implementation of Highlighter
impl Highlighter for ShellHelper {}

impl Validator for ShellHelper {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> std::result::Result<rustyline::validate::ValidationResult, ReadlineError> {
        self.validator.validate(ctx)
    }
}

impl Helper for ShellHelper {} // No extra methods needed.

/// Shell handles user interaction and command execution.
pub struct Shell {
    // Force the Editor to use our ShellHelper and DefaultHistory.
    editor: Editor<ShellHelper, DefaultHistory>,
    // Shared completer for updating history.
    completer: Rc<RefCell<ShellCompleter>>,
}

impl Shell {
    pub fn new() -> std::result::Result<Self, Box<dyn Error>> {
        // Use DefaultHistory explicitly.
        let completer = Rc::new(RefCell::new(ShellCompleter::new()));
        let validator = MatchingBracketValidator::new();

        let config = Config::builder()
            .edit_mode(EditMode::Emacs)
            .completion_type(CompletionType::List)
            .build();

        // Force DefaultHistory as history type.
        let mut editor = Editor::<ShellHelper, DefaultHistory>::with_config(config)?;

        let helper = ShellHelper {
            completer: completer.clone(),
            validator,
        };
        editor.set_helper(Some(helper));

        editor.set_history_ignore_dups(true)?;

        Ok(Shell { editor, completer })
    }

    pub fn run(&mut self) -> std::result::Result<(), Box<dyn Error>> {
        loop {
            // Create a new history instance to pass to the completer
            let history_box: Box<dyn rustyline::history::History> = Box::new(DefaultHistory::new());
            self.completer.borrow_mut().set_history(history_box);

            match self.editor.readline("$ ") {
                Ok(line) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    self.editor.add_history_entry(line.clone())?;
                    let cmdline = Cmd::new(&line);
                    if let Err(e) = cmdline.handle_execs() {
                        eprintln!("Error: {}", e);
                    }
                }
                Err(ReadlineError::Interrupted) => continue,
                Err(ReadlineError::Eof) => break,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    break;
                }
            }
        }
        Ok(())
    }
}
