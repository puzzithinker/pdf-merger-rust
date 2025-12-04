use std::path::PathBuf;
use lopdf::Document;

pub fn merge_pdfs_with_progress(
    file_paths: Vec<PathBuf>,
    output_path: PathBuf,
    _total_files: usize,
) -> Result<(), String> {
    if file_paths.is_empty() {
        return Err("No files to merge.".to_string());
    }

    // Create a new document for merging
    let mut merged_doc = Document::with_version("1.5");
    let mut next_id = merged_doc.max_id + 1;
    let mut all_page_ids = Vec::new();

    // Process each PDF file
    for path in file_paths.iter() {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        // Load the source document
        let mut doc = Document::load(path).map_err(|e| {
            format!(
                "Failed to load '{}': {}. {}",
                file_name,
                e,
                if e.to_string().contains("encrypted") || e.to_string().contains("password") {
                    "This PDF may be password-protected."
                } else {
                    "The file may be corrupted."
                }
            )
        })?;

        if doc.is_encrypted() {
            return Err(format!(
                "PDF '{}' is encrypted (password-protected) and cannot be merged.",
                file_name
            ));
        }

        // Get pages from source document
        let source_pages = doc.get_pages();
        if source_pages.is_empty() {
            return Err(format!("PDF '{}' has no pages.", file_name));
        }

        // Renumber objects in source document to avoid ID conflicts
        doc.renumber_objects_with(next_id);
        next_id = doc.max_id + 1;

        // Get pages again AFTER renumbering to get updated object IDs
        let renumbered_pages = doc.get_pages();

        // Sort pages by page number to maintain correct order
        let mut page_list: Vec<(u32, (u32, u16))> = renumbered_pages
            .iter()
            .map(|(page_num, (obj_id, gen_num))| (*page_num, (*obj_id, *gen_num)))
            .collect();
        page_list.sort_by_key(|(page_num, _)| *page_num);

        // Copy ALL objects from source document (this ensures all dependencies are available)
        for (obj_id, obj) in doc.objects.into_iter() {
            merged_doc.objects.insert(obj_id, obj);
        }

        // Collect page object IDs in the correct order
        for (_, (obj_id, gen_num)) in page_list {
            all_page_ids.push((obj_id, gen_num));
        }
    }

    // Ensure max_id accounts for all imported objects before adding new ones
    merged_doc.max_id = next_id.saturating_sub(1);

    // Build a CLEAN page tree structure (don't use any existing page trees)
    use lopdf::Object;
    use lopdf::Dictionary;

    if all_page_ids.is_empty() {
        return Err("No pages to merge.".to_string());
    }

    // Create Kids array with ONLY the actual page references (not existing page trees)
    let kids: Vec<Object> = all_page_ids
        .iter()
        .map(|&(obj_id, gen_num)| Object::Reference((obj_id, gen_num)))
        .collect();

    // Create a NEW root Pages dictionary
    let pages_dict = Dictionary::from_iter(vec![
        ("Type", "Pages".into()),
        ("Kids", kids.into()),
        ("Count", (all_page_ids.len() as i32).into()),
    ]);

    let pages_id = merged_doc.add_object(pages_dict);

    // Update ONLY the actual page objects to reference our new Pages dictionary
    // and ensure they have the correct Type
    for &(page_id, page_gen) in &all_page_ids {
        if let Ok(page_obj) = merged_doc.get_object_mut((page_id, page_gen)) {
            if let Ok(page_dict) = page_obj.as_dict_mut() {
                // Remove any existing Parent reference that might point to old page trees
                page_dict.remove(b"Parent");
                // Set the Parent reference to our new Pages dictionary
                page_dict.set("Parent", Object::Reference(pages_id));
                // Ensure Type is set to Page
                page_dict.set("Type", "Page");
            }
        }
    }

    // Create the Catalog dictionary that points to our new Pages dictionary
    let catalog_dict = Dictionary::from_iter(vec![
        ("Type", "Catalog".into()),
        ("Pages", Object::Reference(pages_id)),
    ]);

    let catalog_id = merged_doc.add_object(catalog_dict);

    // Update the trailer with the root catalog
    merged_doc.trailer.set("Root", Object::Reference(catalog_id));

    // Ensure document has proper version
    merged_doc.version = "1.5".to_string();

    // Save the merged document
    merged_doc
        .save(&output_path)
        .map_err(|e| format!("Failed to save merged PDF: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_pdfs_function_exists() {
        // This test ensures our merge function compiles correctly
        let _func = merge_pdfs_with_progress;
        assert!(true);
    }
}
