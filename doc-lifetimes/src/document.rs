use crate::section::Section;

pub struct Document {
    pub sections: Vec<Section>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        Document {
            sections: Vec::new(),
        }
    }

    // returned section reference is tied to the document's lifetime
    pub fn get_section(&mut self, idx: usize) -> Option<&mut Section> {
        self.sections.get_mut(idx)
    }

    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }
    pub fn print_all_titles(&self) {
        for (idx, section) in self.sections.iter().enumerate() {
            println!("Section {}: {}", idx, section.title);
        }
    }
}
