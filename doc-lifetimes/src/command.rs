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

/// Parse a single input line into a `Command`.
pub fn parse_command(line: &str) -> Command {
    let mut parts = line.split_whitespace();
    let main_cmd = parts.next().unwrap_or("").to_lowercase();

    match main_cmd.as_str() {
        "" => Command::Unknown, // empty line
        "help" | "h" => Command::Help,
        "quit" | "q" => Command::Quit,

        // "list sections"
        "list" => {
            let sub_cmd = parts.next().unwrap_or("").to_lowercase();
            if sub_cmd == "sec" {
                Command::ListSections
            } else {
                Command::Unknown
            }
        }

        // "save <filename>"
        "save" => {
            let filename = parts.next().unwrap_or("").to_string();
            if filename.is_empty() {
                Command::Unknown
            } else {
                Command::Save(filename)
            }
        }

        // "show sec <idx>"
        "show" => {
            let entity = parts.next().unwrap_or("").to_lowercase(); // e.g. "sec"
            if entity == "sec" {
                if let Some(idx_str) = parts.next() {
                    if let Ok(idx) = idx_str.parse::<usize>() {
                        return Command::ShowSec(idx);
                    }
                }
            }
            Command::Unknown
        }

        // "add sec <title>"
        "add" => {
            let entity = parts.next().unwrap_or("").to_lowercase(); // e.g. "sec"
            match entity.as_str() {
                "sec" => {
                    // Everything remaining is the section title (join with space)
                    let title: String = parts.collect::<Vec<&str>>().join(" ");
                    if title.is_empty() {
                        Command::Unknown
                    } else {
                        Command::AddSec(title)
                    }
                }
                "par" => {
                    // "add par <idx> <content>"
                    // Next piece is <idx>, everything else is <content>.
                    if let Some(idx_str) = parts.next() {
                        if let Ok(idx) = idx_str.parse::<usize>() {
                            // rest is the paragraph content
                            let content: String = parts.collect::<Vec<&str>>().join(" ");
                            return Command::AddPar(idx, content);
                        }
                    }
                    Command::Unknown
                }
                _ => Command::Unknown,
            }
        }

        // "edit sec <idx> <title>" or "edit par <sidx> <pidx> <content>"
        "edit" => {
            let entity = parts.next().unwrap_or("").to_lowercase(); // "sec" or "par"
            match entity.as_str() {
                "sec" => {
                    // next is <idx>, rest is <title>
                    if let Some(idx_str) = parts.next() {
                        if let Ok(idx) = idx_str.parse::<usize>() {
                            let title: String = parts.collect::<Vec<&str>>().join(" ");
                            return Command::EditSec(idx, title);
                        }
                    }
                    Command::Unknown
                }
                "par" => {
                    // next two are <sidx> <pidx>, then rest is <content>
                    if let Some(sidx_str) = parts.next() {
                        if let Ok(sidx) = sidx_str.parse::<usize>() {
                            if let Some(pidx_str) = parts.next() {
                                if let Ok(pidx) = pidx_str.parse::<usize>() {
                                    let content: String = parts.collect::<Vec<&str>>().join(" ");
                                    return Command::EditPar(sidx, pidx, content);
                                }
                            }
                        }
                    }
                    Command::Unknown
                }
                _ => Command::Unknown,
            }
        }

        _ => Command::Unknown,
    }
}

pub fn handle_command(line: &str, doc: &mut Document) -> std::io::Result<()> {
    let command = parse_command(line);
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
