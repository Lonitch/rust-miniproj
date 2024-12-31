pub struct Paragraph
{
  content: String,
}

pub struct Section
{
  pub title: String,
  pub paragraphs: Vec<Paragraph>,
}

pub struct Document
{
  pub sections: Vec<Section>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document
{
  pub fn new() -> Self
  {
    Document { sections: Vec::new() }
  }

  // returned section reference is tied to the document's lifetime
  pub fn get_section(&mut self,
                         idx: usize)
                         -> Option<&mut Section>
  {
    self.sections.get_mut(idx)
  }

  pub fn add_section(&mut self,
                     section: Section)
  {
    self.sections.push(section);
  }
  pub fn print_all_titles(&self)
  {
    for (idx, section) in self.sections.iter().enumerate() {
      println!("Section {}: {}", idx, section.title);
    }
  }
}

impl Section
{
  pub fn new(title: String) -> Self
  {
    Section { title,
              paragraphs: Vec::new() }
  }

  // returned paragraph reference is tied to the section's lifetime
  pub fn get_paragraph(&mut self,
                           index: usize)
                           -> Option<&mut Paragraph>
  {
    self.paragraphs.get_mut(index)
  }

  pub fn add_paragraph(&mut self,
                       paragraph: Paragraph)
  {
    self.paragraphs.push(paragraph);
  }
  pub fn print_all_para(&self)
  {
    for (idx, para) in self.paragraphs.iter().enumerate() {
      println!("Paragraph {}: {}", idx, para.content());
    }
  }
}

impl Paragraph
{
  pub fn new(content: String) -> Self
  {
    Paragraph { content }
  }

  pub fn content(&self) -> &str
  {
    &self.content
  }

  pub fn edit_content(&mut self,
                      new_content: String)
  {
    self.content = new_content;
  }
}

/// An enum representing all possible commands in our small REPL.
#[derive(Debug)]
pub enum Command
{
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
pub fn parse_command(line: &str) -> Command
{
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
    },

    // "save <filename>"
    "save" => {
      let filename = parts.next().unwrap_or("").to_string();
      if filename.is_empty() {
        Command::Unknown
      } else {
        Command::Save(filename)
      }
    },

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
    },

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
        },
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
        },
        _ => Command::Unknown,
      }
    },

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
        },
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
        },
        _ => Command::Unknown,
      }
    },

    _ => Command::Unknown,
  }
}
