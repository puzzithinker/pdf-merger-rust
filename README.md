# PDF Merger (Windows Desktop) - Rust Version

A simple, modern **PDF Merger** desktop application for Windows built with Rust and ICED.

## Features

- **Select or Drag & Drop**: Choose multiple PDF files at once, or drag them into the list.
- **Reorder**: Move files Up/Down or jump to Top/Bottom.
- **List Management**: Remove selected item, clear the whole list, and see order + file size.
- **Merge PDFs**: Choose an output file via Save As dialog; open the output location after merge.
- **Status & Progress**: Live status text and progress polling during merge.
- **Validation & Errors**: Early checks for missing/empty/non-PDF files; clear errors for encrypted/corrupted PDFs and save failures.

## Requirements

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Windows 10/11

## Running the Rust Application

From the project root directory:

```bash
cargo run --release
```

## Building a Standalone EXE (Rust)

The Rust version compiles to a native Windows executable:

```bash
cargo build --release
```

After it finishes:

- The standalone EXE will be at: `target/release/pdf-merger.exe`
- You can copy `pdf-merger.exe` to any folder and run it on Windows without any dependencies (it's a fully native executable).

If you're building on Windows, you can also use the provided build script:
```bash
build-windows.bat
```

For detailed instructions on building on Windows, see [README-Windows.md](README-Windows.md).

## Learning Resources

If you're a student or new to Rust, check out our detailed tutorial: [RUST-TUTORIAL.md](RUST-TUTORIAL.md)
This tutorial explains the codebase in depth, covering everything from basic Rust concepts to GUI programming and PDF manipulation.

## PDF Merge Implementation

The Rust version uses `lopdf` for PDF manipulation. The merge implementation:
- ✅ Loads and validates PDF files
- ✅ Checks for encrypted/password-protected PDFs
- ✅ Improved page tree management with proper object copying
- ✅ Copies page objects and maintains correct page order
- ✅ Copies referenced objects (fonts, resources, XObjects, etc.)
- ✅ Progress updates during merge operation
- ✅ Better error handling and validation

## Testing

See `TESTING.md` for detailed testing instructions and known limitations.

**Note**: The merge has been improved with better page tree management. It should work well for most standard PDFs. Complex PDFs with advanced features (forms, annotations, etc.) may need additional testing.
