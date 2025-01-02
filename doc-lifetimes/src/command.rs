use crate::document::Document;
use crate::markdown::to_markdown;
use crate::paragraph::Paragraph;
use crate::section::Section;
use std::fs;

/// An enum representing all possible commands in our small REPL.
#[derive(Debug)]
pub enum Command {
    Help,
    Quit,
    ListSections,
    Save(String),
    ShowSec(usize),
    AddSec(String),
    EditSec(usize, String),
    AddPar(usize, String),
    EditPar(usize, usize, String),
    Unknown, // fallback if we can’t parse the user input
}

/// Command::from(some_str) results an Command Option
impl From<&str> for Command {
    fn from(line: &str) -> Self {
        // Split the input into whitespace-separated tokens
        let tokens: Vec<_> = line.split_whitespace().collect();
        if tokens.is_empty() {
            // Empty line
            return Command::Unknown;
        }

        // The primary command is always the first token in lowercase
        let main_cmd = tokens[0].to_lowercase();

        match main_cmd.as_str() {
            // "help" or "h"
            "help" | "h" => Command::Help,

            // "quit" or "q"
            "quit" | "q" => Command::Quit,

            // "list sec"
            "list" => {
                // Safe check for second token
                if tokens.get(1).map(|s| s.to_lowercase()) == Some("sec".into()) {
                    Command::ListSections
                } else {
                    Command::Unknown
                }
            }

            // "save <filename>"
            "save" => {
                let filename = tokens.get(1).unwrap_or(&"").to_string();
                if filename.is_empty() {
                    Command::Unknown
                } else {
                    Command::Save(filename)
                }
            }

            // "show sec <idx>"
            "show" => {
                let entity = tokens.get(1).unwrap_or(&"").to_lowercase();
                if entity == "sec" {
                    if let Some(idx_str) = tokens.get(2) {
                        if let Ok(idx) = idx_str.parse::<usize>() {
                            return Command::ShowSec(idx);
                        }
                    }
                }
                Command::Unknown
            }

            // "add sec <title>" OR "add par <idx> <content>"
            "add" => {
                let entity = tokens.get(1).unwrap_or(&"").to_lowercase();
                match entity.as_str() {
                    "sec" => {
                        // Everything after "add sec" is the title
                        let title = tokens[2..].join(" ");
                        if title.is_empty() {
                            Command::Unknown
                        } else {
                            Command::AddSec(title)
                        }
                    }
                    "par" => {
                        // tokens[2] should be <idx>, everything after is paragraph content
                        if let Some(idx_str) = tokens.get(2) {
                            if let Ok(idx) = idx_str.parse::<usize>() {
                                let content = tokens[3..].join(" ");
                                return Command::AddPar(idx, content);
                            }
                        }
                        Command::Unknown
                    }
                    _ => Command::Unknown,
                }
            }

            // "edit sec <idx> <title>" OR "edit par <sidx> <pidx> <content>"
            "edit" => {
                let entity = tokens.get(1).unwrap_or(&"").to_lowercase();
                match entity.as_str() {
                    "sec" => {
                        // tokens[2] = <idx>, tokens[3..] = new title
                        if let Some(idx_str) = tokens.get(2) {
                            if let Ok(idx) = idx_str.parse::<usize>() {
                                let new_title = tokens[3..].join(" ");
                                return Command::EditSec(idx, new_title);
                            }
                        }
                        Command::Unknown
                    }
                    "par" => {
                        // tokens[2] = <sidx>, tokens[3] = <pidx>, tokens[4..] = content
                        if let (Some(sidx_str), Some(pidx_str)) = (tokens.get(2), tokens.get(3)) {
                            if let (Ok(sidx), Ok(pidx)) =
                                (sidx_str.parse::<usize>(), pidx_str.parse::<usize>())
                            {
                                let content = tokens[4..].join(" ");
                                return Command::EditPar(sidx, pidx, content);
                            }
                        }
                        Command::Unknown
                    }
                    _ => Command::Unknown,
                }
            }

            // Anything else is unknown
            _ => Command::Unknown,
        }
    }
}

pub fn handle_command(line: &str, doc: &mut Document) -> std::io::Result<()> {
    let command = Command::from(line);
    match command {
        Command::Help => {
            println!("Commands:");
            println!("  help (h)                       - print this help");
            println!("  quit (q)                       - exit the program");
            println!("  list sec                       - list all sections");
            println!("  save <filename>                - save document");
            println!("  show sec <idx>                 - show paragraphs in section <idx>");
            println!("  add sec <title>                - add a new section");
            println!("  edit sec <idx> <title>         - edit a section's title");
            println!("  add par <idx> <content>        - add paragraph to section <idx>");
            println!("  edit par <sidx> <pidx> <text>  - edit paragraph <pidx> of section <sidx>");
            Ok(())
        }
        Command::Quit => {
            std::process::exit(1);
        }
        Command::ListSections => {
            doc.print_all_titles();
            Ok(())
        }
        Command::Save(out_file) => {
            if out_file.is_empty() {
                println!("No filename provided for save.");
            } else {
                let md = to_markdown(doc);
                fs::write(&out_file, md)?;
                println!("Document saved to '{}'.", out_file);
            }
            Ok(())
        }
        Command::ShowSec(idx) => {
            // Show paragraphs in the specified section
            if let Some(sec) = doc.get_section(idx) {
                println!("Section {}: {}", idx, sec.title);
                sec.print_all_para();
            } else {
                println!("No section at index {}", idx);
            }
            Ok(())
        }
        Command::AddSec(title) => {
            // Add a new section with the given title
            doc.add_section(Section::new(title));
            println!("Added new section.");
            Ok(())
        }
        Command::EditSec(idx, new_title) => {
            // Edit the title of section <idx>
            if let Some(sec) = doc.get_section(idx) {
                sec.title = new_title;
                println!("Section {} title updated.", idx);
            } else {
                println!("No section at index {}", idx);
            }
            Ok(())
        }
        Command::AddPar(sidx, content) => {
            // Add a paragraph to section <sidx>
            if let Some(sec) = doc.get_section(sidx) {
                sec.add_paragraph(Paragraph::new(content));
                println!("Added paragraph to section {}.", sidx);
            } else {
                println!("No section at index {}", sidx);
            }
            Ok(())
        }
        Command::EditPar(sidx, pidx, new_content) => {
            // Edit the paragraph <pidx> in section <sidx>
            if let Some(sec) = doc.get_section(sidx) {
                if let Some(para) = sec.get_paragraph(pidx) {
                    para.edit_content(new_content);
                    println!("Paragraph {} in section {} updated.", pidx, sidx);
                } else {
                    println!("No paragraph at index {} in section {}.", pidx, sidx);
                }
            } else {
                println!("No section at index {}", sidx);
            }
            Ok(())
        }
        Command::Unknown => {
            if !line.is_empty() {
                println!("Unknown command. Type 'help' for usage.");
            }
            Ok(())
        }
    }
}
