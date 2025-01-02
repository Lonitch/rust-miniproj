use crate::paragraph::Paragraph;

pub struct Section {
    pub title: String,
    pub paragraphs: Vec<Paragraph>,
}

impl Section {
    pub fn new(title: String) -> Self {
        Section {
            title,
            paragraphs: Vec::new(),
        }
    }

    // returned paragraph reference is tied to the section's lifetime
    pub fn get_paragraph(&mut self, index: usize) -> Option<&mut Paragraph> {
        self.paragraphs.get_mut(index)
    }

    pub fn add_paragraph(&mut self, paragraph: Paragraph) {
        self.paragraphs.push(paragraph);
    }
    pub fn print_all_para(&self) {
        for (idx, para) in self.paragraphs.iter().enumerate() {
            println!("Paragraph {}: {}", idx, para.content());
        }
    }
}
