use doc_lifetimes::{parse_command, Command, Document, Paragraph, Section};
use std::io::{self, Write};
use std::{env, fs};

/// parse md file into Document
fn parse_markdown(content: &str) -> Document
{
  let lines: Vec<&str> = content.lines().collect();
  let mut doc = Document::new();

  let mut cur_sec: Option<Section> = None;

  for line in lines {
    if line.starts_with('#') {
      if let Some(sec) = cur_sec.take() {
        doc.add_section(sec);
      }

      let title = line.trim_start_matches('#').trim().to_string();
      cur_sec = Some(Section::new(title));
    } else {
      let trimmed = line.trim();
      if !trimmed.is_empty() {
        if let Some(sec) = cur_sec.as_mut() {
          sec.add_paragraph(Paragraph::new(trimmed.to_string()));
        }
      }
    }
  }

  // remaining section
  if let Some(sec) = cur_sec.take() {
    doc.add_section(sec);
  }

  doc
}

/// convert the `Document` back to Markdown text.
fn to_markdown(doc: &Document) -> String
{
  let mut output = String::new();

  for section in &doc.sections {
    output.push_str("# ");
    output.push_str(&section.title);
    output.push('\n');
    for para in &section.paragraphs {
      output.push_str(para.content());
      output.push('\n');
    }
  }
  output
}

fn main() -> io::Result<()>
{
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    eprintln!("Usage: {} <filename>",
              args.first().unwrap_or(&"main".to_string()));
    return Ok(());
  }

  let filename = &args[1];
  let content = fs::read_to_string(filename)?;
  let mut doc = parse_markdown(&content);

  // Command loop
  loop {
    print!("> ");
    io::stdout().flush()?;

    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let line = line.trim();

    // Parse into our Command enum
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
      },
      Command::Quit => {
        break;
      },
      Command::ListSections => {
        doc.print_all_titles();
      },
      Command::Save(out_file) => {
        if out_file.is_empty() {
          println!("No filename provided for save.");
        } else {
          let md = to_markdown(&doc);
          fs::write(&out_file, md)?;
          println!("Document saved to '{}'.", out_file);
        }
      },
      Command::ShowSec(idx) => {
        // Show paragraphs in the specified section
        if let Some(sec) = doc.get_section(idx) {
          println!("Section {}: {}", idx, sec.title);
          sec.print_all_para();
        } else {
          println!("No section at index {}", idx);
        }
      },
      Command::AddSec(title) => {
        // Add a new section with the given title
        doc.add_section(Section::new(title));
        println!("Added new section.");
      },
      Command::EditSec(idx, new_title) => {
        // Edit the title of section <idx>
        if let Some(sec) = doc.get_section(idx) {
          sec.title = new_title;
          println!("Section {} title updated.", idx);
        } else {
          println!("No section at index {}", idx);
        }
      },
      Command::AddPar(sidx, content) => {
        // Add a paragraph to section <sidx>
        if let Some(sec) = doc.get_section(sidx) {
          sec.add_paragraph(Paragraph::new(content));
          println!("Added paragraph to section {}.", sidx);
        } else {
          println!("No section at index {}", sidx);
        }
      },
      Command::EditPar(sidx, pidx, new_content) => {
        // Edit the paragraph <pidx> in section <sidx>
        if let Some(sec) = doc.get_section(sidx) {
          if let Some(para) = sec.get_paragraph(pidx) {
            para.edit_content(new_content);
            println!("Paragraph {} in section {} updated.", pidx, sidx);
          } else {
            println!("No paragraph at index {} in section {}.",
                     pidx, sidx);
          }
        } else {
          println!("No section at index {}", sidx);
        }
      },
      Command::Unknown => {
        if !line.is_empty() {
          println!("Unknown command. Type 'help' for usage.");
        }
      },
    }
  }

  Ok(())
}
