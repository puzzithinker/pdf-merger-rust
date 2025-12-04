# PDF Merger - Testing Guide

## Testing the Merge Functionality

### What Has Been Improved

1. **Page Tree Management**
   - Improved object copying with proper page ID management
   - Better handling of page references and object dependencies
   - Enhanced resource copying (fonts, XObjects, etc.)
   - Proper page numbering in merged document

2. **Progress Updates**
   - Status messages show merge progress
   - Progress bar updates during merge operation
   - File-by-file processing feedback

3. **Error Handling**
   - Better error messages for encrypted PDFs
   - Clear messages for corrupted files
   - Validation of PDF structure before merging

## Automated Testing

### Running Tests

Run all unit and integration tests:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

### Test Coverage

**Unit Tests (src/lib.rs):**
- `test_merge_pdfs_function_exists`: Verifies function signature
- `test_merge_empty_file_list`: Tests error handling for empty input
- `test_merge_nonexistent_file`: Tests error handling for missing files
- `test_progress_callback`: Tests callback structure
- `test_output_path_validation`: Tests output file creation

**Integration Tests:**
- `cli_args.rs`: CLI argument validation
- `cli_regression.rs`: CLI merge functionality
- `merge_regression.rs`: Core merge without duplication
- `merge_empty_pdf.rs`: Empty PDF file rejection
- `validation_test.rs`: Input validation logic
  - File existence checks
  - Empty file detection
  - Extension validation (case-insensitive)
  - Multiple file validation
  - Error message verification

### How to Test

#### Basic Merge Test

1. **Prepare Test PDFs:**
   - Create or gather 2-3 simple PDF files (non-password-protected)
   - Include 1 empty file to confirm validation catches it
   - Note the page count of each file

2. **Run the Application:**
   ```powershell
   cargo run --release
   ```

3. **Test Steps:**
   - Click "Select Files" or drag/drop PDFs into the list
   - Verify files appear with order numbers and sizes; validation errors should show inline
   - Use "Move Up/Down" or "Top/Bottom" to reorder
   - Click "Merge PDFs" (button should be disabled if any file is invalid)
   - Choose output location
   - Verify the merged PDF is created and the "Open Output Location" button appears

4. **Verify Results:**
   - Open the merged PDF
   - Check that all pages are present
   - Verify page order matches the list order
   - Check that content renders correctly

#### Advanced Tests

**Test 1: Multiple Files (5+ PDFs)**
- Merge 5-10 PDF files
- Verify all pages are included
- Check that page order is correct

**Test 2: Different PDF Types**
- Test with PDFs containing:
  - Text only
  - Images
  - Mixed content (text + images)
  - Forms (if applicable)

**Test 3: Large Files**
- Test with PDFs that are several MB in size
- Verify merge completes successfully
- Check output file size is reasonable

**Test 4: Edge Cases**
- Try merging a single PDF (should work)
- Try merging PDFs with different page sizes
- Test with PDFs that have bookmarks/outlines
- Provide an empty file: merge should fail with a clear validation error
- Try a non-PDF file: merge should fail before starting

### Known Limitations

1. **Complex PDFs:**
   - PDFs with advanced features (forms, annotations, etc.) may need additional testing
   - Some complex page tree structures might not merge perfectly

2. **Progress Updates:**
   - Current implementation shows overall progress
   - Per-file progress updates require more complex async setup
   - Status messages provide file-by-file feedback

3. **Page Tree:**
   - The merge uses lopdf's internal page management
   - For very complex PDFs, manual page tree reconstruction might be needed

### Troubleshooting

**Issue: Merge fails with "encrypted" error**
- Solution: The PDF is password-protected. Remove password or use a different PDF.

**Issue: Merge completes but pages are missing**
- Solution: This may indicate a page tree issue. Try with simpler PDFs first.

**Issue: Application crashes during merge**
- Solution: Check that PDFs are not corrupted. Try with different files.

**Issue: Merged PDF is very large**
- Solution: This is normal if source PDFs are large. Consider compressing after merge.

### Reporting Issues

If you encounter problems:
1. Note which PDFs were being merged
2. Check the error message in the status bar
3. Try with simpler PDFs to isolate the issue
4. Check that all source PDFs open correctly in a PDF viewer

### Performance Notes

- Merge speed depends on:
  - Number of files
  - Total page count
  - PDF complexity
  - File sizes

- Typical performance:
  - Small PDFs (< 1MB): < 1 second
  - Medium PDFs (1-10MB): 1-5 seconds
  - Large PDFs (> 10MB): 5-30 seconds

