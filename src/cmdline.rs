#![allow(dead_code)]
#![allow(unused_assignments)]
mod builtin_handlers;
mod executable;
use builtin_handlers::*;
use executable::Executable;
use std::fs::{File, OpenOptions};
use std::process::{Command, Stdio};

pub struct Cmdline {
    // piped stages
    pub stages: Vec<Cmd>,
}

pub struct Cmd {
    // The command to execute (corresponds to existing Command enum)
    pub exec: Executable,

    // Arguments for the command
    pub args: Vec<String>,

    // Redirection information
    pub stdout_redirect: Option<RedirectInfo>,
    pub stderr_redirect: Option<RedirectInfo>,
    pub stdin_redirect: Option<RedirectInfo>,
}

pub struct RedirectInfo {
    pub path: String,
    // descriptor number (default: 1 for stdout)
    pub fd: u32,
    // redirection mode
    pub mode: RedirectMode,
}

pub enum RedirectMode {
    // > - Write to file (create or truncate)
    Write,
    // >| - Force write even with noclobber
    ForceWrite,
    // >> - Append to file
    Append,
    // < - Read from file
    Read,
}

impl Cmd {
    pub fn new(input: &str) -> Self {
        let args = Cmd::parse_args(input);
        let (stdout_redirect, stderr_redirect, stdin_redirect) =
            Cmd::ensure_redirect(&args).unwrap();
        let exec = if let Some(cmd_name) = args.first() {
            Executable::from(cmd_name.clone())
        } else {
            Executable::Unknown("".to_string())
        };
        Self {
            exec,
            args,
            stdout_redirect,
            stderr_redirect,
            stdin_redirect,
        }
    }
    pub fn handle_execs(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self.exec {
            Executable::Cd => handle_cd(&self.args),
            Executable::Exit => handle_exit(&self.args),
            Executable::Echo => handle_echo(self)?,
            Executable::Pwd => handle_pwd(),
            Executable::Type => handle_type(&self.args),
            Executable::Unknown(_) => self.cmd_exec()?,
        }
        Ok(())
    }

    fn parse_args(input: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut in_single_quotes = false;
        let mut in_double_quotes = false;
        let mut escaped = false;

        for c in input.chars() {
            if escaped {
                if in_double_quotes {
                    if c == '$' || c == '`' || c == '"' || c == '\\' || c == '\n' {
                        current_arg.push(c);
                    } else {
                        current_arg.push('\\');
                        current_arg.push(c);
                    }
                } else if in_single_quotes {
                    // Inside single quotes, backslash has no special meaning
                    current_arg.push('\\');
                    current_arg.push(c);
                } else if c == '\n' {
                    // Line continuation outside quotes
                    // Skip both the backslash and newline
                } else {
                    // Outside quotes, backslash escapes any character
                    current_arg.push(c);
                }
                escaped = false;
            } else if c == '\\' {
                // Potential escape sequence
                escaped = true;
            } else if c == '\'' && !in_double_quotes {
                // Toggle single quotes
                in_single_quotes = !in_single_quotes;
            } else if c == '"' && !in_single_quotes {
                // Toggle double quotes
                in_double_quotes = !in_double_quotes;
            } else if c.is_whitespace() && !in_single_quotes && !in_double_quotes {
                // End of argument
                if !current_arg.is_empty() {
                    args.push(current_arg);
                    current_arg = String::new();
                }
            } else {
                current_arg.push(c);
            }
        }

        // Handle any trailing escape character
        if escaped {
            if !in_single_quotes && !in_double_quotes {
                current_arg.push('\\');
            } else if in_single_quotes {
                current_arg.push('\\');
            }
            // In double quotes, a trailing backslash is just a backslash
            if in_double_quotes {
                current_arg.push('\\');
            }
        }

        // Add the last argument if not empty
        if !current_arg.is_empty() {
            args.push(current_arg);
        }

        args
    }

    fn ensure_redirect(
        args: &[String],
    ) -> Result<
        (
            Option<RedirectInfo>,
            Option<RedirectInfo>,
            Option<RedirectInfo>,
        ),
        Box<dyn std::error::Error>,
    > {
        let mut output_target = None;
        let mut stdout_redirect = None;
        let mut stderr_redirect = None;
        let mut stdin_redirect = None;
        // process arguments to find redirection
        let mut i = 1;
        while i < args.len() {
            if (args[i] == ">" || args[i] == "1>") && i + 1 < args.len() {
                output_target = Some(args[i + 1].clone());
                stdout_redirect = Some(RedirectInfo {
                    fd: 1,
                    path: args[i + 1].clone(),
                    mode: RedirectMode::Write,
                });
            }
            if args[i] == ">>" && i + 1 < args.len() {
                output_target = Some(args[i + 1].clone());
                stdout_redirect = Some(RedirectInfo {
                    fd: 1,
                    path: args[i + 1].clone(),
                    mode: RedirectMode::Append,
                });
            }
            if args[i] == ">|" && i + 1 < args.len() {
                output_target = Some(args[i + 1].clone());
                stdout_redirect = Some(RedirectInfo {
                    fd: 1,
                    path: args[i + 1].clone(),
                    mode: RedirectMode::ForceWrite,
                });
            }
            if (args[i] == "2>") && i + 1 < args.len() {
                output_target = Some(args[i + 1].clone());
                stderr_redirect = Some(RedirectInfo {
                    fd: 2,
                    path: args[i + 1].clone(),
                    mode: RedirectMode::Write,
                });
            }
            if args[i] == "2>>" && i + 1 < args.len() {
                output_target = Some(args[i + 1].clone());
                stderr_redirect = Some(RedirectInfo {
                    fd: 2,
                    path: args[i + 1].clone(),
                    mode: RedirectMode::Append,
                });
            }
            if args[i] == "<" && i - 1 > 0 {
                output_target = Some(args[i - 1].clone());
                stdin_redirect = Some(RedirectInfo {
                    fd: 0,
                    path: args[i - 1].clone(),
                    mode: RedirectMode::Read,
                });
            }

            i += 1;
        }

        if let Some(file_path) = output_target {
            if let Some(parent) = std::path::Path::new(&file_path).parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
        }
        Ok((stdout_redirect, stderr_redirect, stdin_redirect))
    }

    fn executable_exists(&self) -> bool {
        match &self.exec {
            Executable::Unknown(x) => {
                if let Ok(path) = std::env::var("PATH") {
                    for dir in path.split(':') {
                        let path = std::path::Path::new(dir).join(x.trim());
                        if path.exists() {
                            return true;
                        }
                    }
                }
                println!("{}: command not found", x.trim());
                false
            }
            _ => true,
        }
    }

    pub fn cmd_exec(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some((prog, args)) = self.args.split_first() {
            if let Executable::Unknown(_) = Executable::from(prog.clone()) {
                if self.executable_exists() {
                    let mut command = Command::new(prog);
                    let mut skip_next = false;
                    for (i, arg) in args.iter().enumerate() {
                        if skip_next {
                            skip_next = false;
                            continue;
                        }

                        // Skip redirection operators and their targets
                        if (arg == ">"
                            || arg == ">>"
                            || arg == "1>"
                            || arg == "1>>"
                            || arg == "2>"
                            || arg == "2>>"
                            || arg == "<"
                            || arg == ">|")
                            && i + 1 < args.len()
                        {
                            skip_next = true;
                            continue;
                        }

                        command.arg(arg);
                    }

                    if let Some(stdout_info) = &self.stdout_redirect {
                        let file = match stdout_info.mode {
                            RedirectMode::Write | RedirectMode::ForceWrite => {
                                File::create(&stdout_info.path)?
                            }
                            RedirectMode::Append => OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(&stdout_info.path)?,
                            _ => return Err("Invalid redirection mode for stdout".into()),
                        };
                        command.stdout(Stdio::from(file));
                    } else {
                        command.stdout(Stdio::inherit());
                    }

                    if let Some(stderr_info) = &self.stderr_redirect {
                        let file = match stderr_info.mode {
                            RedirectMode::Write | RedirectMode::ForceWrite => {
                                File::create(&stderr_info.path)?
                            }
                            RedirectMode::Append => OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(&stderr_info.path)?,
                            _ => return Err("Invalid redirection mode for stderr".into()),
                        };
                        command.stderr(Stdio::from(file));
                    } else {
                        command.stderr(Stdio::inherit());
                    }

                    // Setup stdin redirection
                    if let Some(stdin_info) = &self.stdin_redirect {
                        let file = File::open(&stdin_info.path)?;
                        command.stdin(Stdio::from(file));
                    } else {
                        command.stdin(Stdio::inherit());
                    }

                    // Execute command
                    command.status()?;
                }
            }
        }
        Ok(())
    }
}
