#![cfg(test)]

use super::{tables, yaml_blocks};

#[test]
fn yaml_blocks_skip_other_languages_and_unfenced_text() {
    let md =
        "intro `yaml` word\n\n```yaml\na: 1\n```\n\n```rust\nfn x() {}\n```\n\n```yml\nb: 2\n```\n";
    assert_eq!(
        yaml_blocks(md),
        vec!["a: 1\n".to_string(), "b: 2\n".to_string()]
    );
}

#[test]
fn table_header_and_rows_keep_blank_cells() {
    let md = "# R\n\n| run | result | notes |\n|---|---|---|\n| r1 | 5.1 | |\n| r2 | | wind |\n";
    let t = &tables(md)[0];
    assert_eq!(t.header, vec!["run", "result", "notes"]);
    assert_eq!(t.rows, vec![vec!["r1", "5.1", ""], vec!["r2", "", "wind"]]);
    assert_eq!(t.column("notes"), Some(2));
}

#[test]
fn table_cells_drop_inline_formatting() {
    let md = "| a | b |\n|---|---|\n| **x** | `y` |\n";
    assert_eq!(tables(md)[0].rows, vec![vec!["x", "y"]]);
}
