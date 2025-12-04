use assert_cmd::prelude::*;
use printpdf::{BuiltinFont, Mm, PdfDocument};
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

fn create_single_page_pdf(dir: &Path, filename: &str, text: &str) -> PathBuf {
    let path = dir.join(filename);
    let (doc, page1, layer1) = PdfDocument::new(text, Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .expect("builtin font");
    let layer = doc.get_page(page1).get_layer(layer1);
    layer.use_text(text, 12.0, Mm(10.0), Mm(280.0), &font);
    doc.save(&mut BufWriter::new(File::create(&path).expect("file create")))
        .expect("save pdf");
    path
}

#[test]
fn cli_merges_single_page_pdfs_without_duplication() {
    let dir = tempdir().expect("tmp dir");
    let pdf1 = create_single_page_pdf(dir.path(), "1.pdf", "first");
    let pdf2 = create_single_page_pdf(dir.path(), "2.pdf", "second");
    let output = dir.path().join("merged_cli.pdf");

    Command::cargo_bin("pdf-merger-cli")
        .expect("binary exists")
        .args([
            pdf1.to_str().unwrap(),
            pdf2.to_str().unwrap(),
            output.to_str().unwrap(),
        ])
        .assert()
        .success();

    let merged = lopdf::Document::load(&output).expect("load merged");
    let pages = merged.get_pages();
    assert_eq!(pages.len(), 2, "CLI merge should produce 2 pages");
}
