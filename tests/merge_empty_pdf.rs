use pdf_merger::merge_pdfs_with_progress;
use printpdf::{BuiltinFont, Mm, PdfDocument};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;
use tempfile::tempdir;

fn create_single_page_pdf(path: PathBuf, text: &str) {
    let (doc, page1, layer1) = PdfDocument::new(text, Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).expect("font");
    let layer = doc.get_page(page1).get_layer(layer1);
    layer.use_text(text, 12.0, Mm(10.0), Mm(280.0), &font);
    doc.save(&mut BufWriter::new(File::create(&path).expect("file")))
        .expect("save");
}

#[test]
fn rejects_empty_pdf_file() {
    let dir = tempdir().expect("tmp dir");
    let valid = dir.path().join("ok.pdf");
    create_single_page_pdf(valid.clone(), "hello");

    let empty = dir.path().join("empty.pdf");
    fs::write(&empty, []).expect("write empty");

    let output = dir.path().join("merged.pdf");
    let result = merge_pdfs_with_progress::<fn(usize, usize, &PathBuf)>(vec![valid, empty], output, 2, None);
    assert!(
        result.is_err(),
        "expected merge to fail when encountering empty PDF"
    );
}
