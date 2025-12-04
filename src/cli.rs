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

    match merge_pdfs_with_progress(input_paths, output_path, args.len() - 2) {
        Ok(()) => println!("PDFs merged successfully!"),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}