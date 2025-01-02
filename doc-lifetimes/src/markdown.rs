use crate::document::Document;
use crate::paragraph::Paragraph;
use crate::section::Section;

/// parse md file into Document
pub fn parse_markdown(content: &str) -> Document {
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
pub fn to_markdown(doc: &Document) -> String {
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
