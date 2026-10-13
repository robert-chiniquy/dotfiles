//! Markdown access for the experiment files: fenced YAML blocks and GitHub
//! tables, read with a CommonMark parser so that a pipe or a fence inside
//! prose is never mistaken for data.

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

mod tests;

/// One GitHub table: header cells and body rows, cell text trimmed. Inline
/// formatting (emphasis, code spans) is reduced to its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    /// Index of the header cell named `name`. Columns are located by name so
    /// that a person may reorder or add columns in the run sheet.
    pub fn column(&self, name: &str) -> Option<usize> {
        self.header.iter().position(|h| h == name)
    }
}

/// Bodies of the fenced code blocks whose info string names YAML (`yaml` or
/// `yml`), in document order.
pub fn yaml_blocks(markdown: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current: Option<String> = None;
    for event in Parser::new_ext(markdown, Options::empty()) {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                let lang = info.split_whitespace().next().unwrap_or("");
                if lang == "yaml" || lang == "yml" {
                    current = Some(String::new());
                }
            }
            Event::Text(text) => {
                if let Some(body) = current.as_mut() {
                    body.push_str(&text);
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(body) = current.take() {
                    blocks.push(body);
                }
            }
            _ => {}
        }
    }
    blocks
}

/// Every GitHub table in the document, in order.
pub fn tables(markdown: &str) -> Vec<Table> {
    let mut tables = Vec::new();
    let mut table: Option<Table> = None;
    let mut row: Vec<String> = Vec::new();
    let mut cell: Option<String> = None;
    let mut in_head = false;
    for event in Parser::new_ext(markdown, Options::ENABLE_TABLES) {
        match event {
            Event::Start(Tag::Table(_)) => {
                table = Some(Table {
                    header: Vec::new(),
                    rows: Vec::new(),
                });
            }
            Event::Start(Tag::TableHead) => {
                in_head = true;
                row.clear();
            }
            Event::Start(Tag::TableRow) => row.clear(),
            Event::Start(Tag::TableCell) => cell = Some(String::new()),
            Event::Text(text) | Event::Code(text) => {
                if let Some(c) = cell.as_mut() {
                    c.push_str(&text);
                }
            }
            Event::End(TagEnd::TableCell) => {
                if let Some(c) = cell.take() {
                    row.push(c.trim().to_string());
                }
            }
            Event::End(TagEnd::TableHead) => {
                in_head = false;
                if let Some(t) = table.as_mut() {
                    t.header = std::mem::take(&mut row);
                }
            }
            Event::End(TagEnd::TableRow) => {
                // pulldown-cmark wraps header cells in TableHead directly; a
                // TableRow end inside the head would otherwise steal the header.
                if !in_head {
                    if let Some(t) = table.as_mut() {
                        t.rows.push(std::mem::take(&mut row));
                    }
                }
            }
            Event::End(TagEnd::Table) => {
                if let Some(t) = table.take() {
                    tables.push(t);
                }
            }
            _ => {}
        }
    }
    tables
}
