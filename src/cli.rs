use std::path::PathBuf;
use pdf_merger::merge_pdfs_with_progress;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <input1.pdf> <input2.pdf> ... <output.pdf>", args[0]);
        std::process::exit(1);
    }

    let output_path = PathBuf::from(&args[args.len() - 1]);
    let input_paths: Vec<PathBuf> = args[1..args.len() - 1]
        .iter()
        .map(|arg| PathBuf::from(arg))
        .collect();

    if let Some(err) = validate_inputs(&input_paths) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }

    match merge_pdfs_with_progress::<fn(usize, usize, &PathBuf)>(input_paths, output_path, args.len() - 2, None) {
        Ok(()) => println!("PDFs merged successfully!"),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

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
