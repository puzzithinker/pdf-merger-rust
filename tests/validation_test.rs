use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Replicate validation logic from main.rs for testing
fn validate_inputs(paths: &[PathBuf]) -> Option<String> {
    for path in paths {
        if !path.exists() {
            return Some(format!("File not found: {}", path.display()));
        }
        if path.metadata().map(|m| m.len()).unwrap_or(0) == 0 {
            return Some(format!("File is empty: {}", path.display()));
        }
        if path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) != Some("pdf".to_string()) {
            return Some(format!("Not a PDF: {}", path.display()));
        }
    }
    None
}

#[test]
fn test_validate_nonexistent_file() {
    let paths = vec![PathBuf::from("/nonexistent/file.pdf")];
    let result = validate_inputs(&paths);

    assert!(result.is_some());
    assert!(result.unwrap().contains("File not found"));
}

#[test]
fn test_validate_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let empty_file = temp_dir.path().join("empty.pdf");
    fs::write(&empty_file, []).unwrap();

    let paths = vec![empty_file];
    let result = validate_inputs(&paths);

    assert!(result.is_some());
    assert!(result.unwrap().contains("File is empty"));
}

#[test]
fn test_validate_non_pdf_extension() {
    let temp_dir = TempDir::new().unwrap();
    let txt_file = temp_dir.path().join("document.txt");
    fs::write(&txt_file, "some content").unwrap();

    let paths = vec![txt_file];
    let result = validate_inputs(&paths);

    assert!(result.is_some());
    assert!(result.unwrap().contains("Not a PDF"));
}

#[test]
fn test_validate_case_insensitive_pdf_extension() {
    let temp_dir = TempDir::new().unwrap();
    let uppercase_pdf = temp_dir.path().join("document.PDF");
    fs::write(&uppercase_pdf, "dummy content").unwrap();

    let paths = vec![uppercase_pdf];
    let result = validate_inputs(&paths);

    // Should pass extension check (case insensitive)
    // May fail on actual PDF structure, but extension check passes
    assert!(result.is_none() || !result.unwrap().contains("Not a PDF"));
}

#[test]
fn test_validate_multiple_files_first_invalid() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_file = PathBuf::from("/nonexistent.pdf");
    let valid_file = temp_dir.path().join("valid.pdf");
    fs::write(&valid_file, "content").unwrap();

    let paths = vec![invalid_file, valid_file];
    let result = validate_inputs(&paths);

    // Should fail on first file
    assert!(result.is_some());
    assert!(result.unwrap().contains("File not found"));
}

#[test]
fn test_validate_multiple_files_second_invalid() {
    let temp_dir = TempDir::new().unwrap();
    let valid_file = temp_dir.path().join("valid.pdf");
    fs::write(&valid_file, "content").unwrap();
    let empty_file = temp_dir.path().join("empty.pdf");
    fs::write(&empty_file, []).unwrap();

    let paths = vec![valid_file, empty_file];
    let result = validate_inputs(&paths);

    // Should fail on second file
    assert!(result.is_some());
    assert!(result.unwrap().contains("File is empty"));
}

#[test]
fn test_validate_empty_list() {
    let paths: Vec<PathBuf> = vec![];
    let result = validate_inputs(&paths);

    // Empty list should pass validation (business logic handles it separately)
    assert!(result.is_none());
}
