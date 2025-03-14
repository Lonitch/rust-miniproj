use super::executable::Executable;
use super::{Cmd, RedirectMode};
use std::fs::OpenOptions;
use std::io::Write;

pub fn handle_cd(cmd: &[String]) {
    let dir = cmd.get(1).map_or("", |s| s.trim());
    let path = if dir.starts_with("~") {
        if let Ok(home) = std::env::var("HOME") {
            if dir == "~" {
                std::path::PathBuf::from(home)
            } else {
                std::path::Path::new(&home).join(&dir[1..])
            }
        } else {
            std::path::PathBuf::from(dir)
        }
    } else {
        std::path::PathBuf::from(dir)
    };

    if std::env::set_current_dir(&path).is_err() {
        println!("cd: {}: No such file or directory", path.display());
    }
}

pub fn handle_exit(cmd: &[String]) {
    let res = cmd.last().unwrap();
    match res.trim().parse::<i32>() {
        Ok(num) => std::process::exit(num),
        Err(_) => println!("Invalid exit command"),
    }
}

pub fn handle_echo(cmd: &Cmd) -> Result<(), Box<dyn std::error::Error>> {
    // Check for -n flag
    let mut no_newline = false;
    let mut arg_index = 1;
    let mut output_message = String::new();

    if cmd.args.len() > 1 && cmd.args[1] == "-n" {
        no_newline = true;
        arg_index = 2;
    }

    // Process arguments to find redirection
    let mut i = arg_index;
    while i < cmd.args.len() {
        if (cmd.args[i] == ">"
            || cmd.args[i] == ">>"
            || cmd.args[i] == "1>"
            || cmd.args[i] == "1>>"
            || cmd.args[i] == "2>"
            || cmd.args[i] == "2>>"
            || cmd.args[i] == "<"
            || cmd.args[i] == ">|")
            && i + 1 < cmd.args.len()
        {
            i += 2;
            break;
        }

        if i > arg_index && !output_message.is_empty() {
            output_message.push(' ');
        }
        output_message.push_str(&cmd.args[i]);
        i += 1;
    }

    match (&cmd.stdout_redirect, &cmd.stderr_redirect) {
        (Some(redirect), None) | (None, Some(redirect)) => {
            let mut file = match redirect.mode {
                RedirectMode::Write | RedirectMode::ForceWrite => OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&redirect.path)?,
                RedirectMode::Append => OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&redirect.path)?,
                _ => return Err("Invalid redirection mode".into()),
            };
            if redirect.fd == 1 {
                if no_newline {
                    write!(file, "{}", output_message)?;
                } else {
                    writeln!(file, "{}", output_message)?;
                }
            } else {
                if no_newline {
                    print!("{}", output_message);
                    std::io::stdout().flush()?;
                } else {
                    println!("{}", output_message);
                }
            }
        }
        _ => {
            // No redirection, print to stdout
            if no_newline {
                print!("{}", output_message);
                std::io::stdout().flush()?;
            } else {
                println!("{}", output_message);
            }
        }
    }

    Ok(())
}

pub fn handle_pwd() {
    if let Ok(path) = std::env::current_dir() {
        println!("{}", path.display())
    }
}

pub fn handle_type(cmd: &[String]) {
    let second = cmd.last().unwrap().trim();
    if Executable::is_builtin(second) {
        println!("{second} is a shell builtin");
    } else if let Ok(path) = std::env::var("PATH") {
        let mut cmd_exist = false;
        for dir in path.split(':') {
            let p = std::path::Path::new(dir).join(second);
            if p.exists() {
                println!("{second} is {}", p.to_str().unwrap());
                cmd_exist = true;
                break;
            }
        }
        if !cmd_exist {
            println!("{second}: not found");
        }
    } else {
        println!("{second}: not found");
    }
}
