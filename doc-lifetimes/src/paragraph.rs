pub struct Paragraph {
    content: String,
}

impl Paragraph {
    pub fn new(content: String) -> Self {
        Paragraph { content }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn edit_content(&mut self, new_content: String) {
        self.content = new_content;
    }
}
