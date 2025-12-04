import fitz  # PyMuPDF

def detailed_content_check():
    """Check content of merged PDF in detail"""
    try:
        doc = fitz.open("merged.pdf")
        print(f"Merged PDF has {doc.page_count} pages")
        
        print("\nPage content analysis:")
        content_hashes = []
        for i in range(doc.page_count):
            page = doc[i]
            text = page.get_text()
            # Get first few lines for quick identification
            lines = text.strip().split('\n')
            preview = ' '.join(lines[:3])[:100] + "..." if len(' '.join(lines[:3])) > 100 else ' '.join(lines[:3])
            
            content_hash = hash(text.strip())
            content_hashes.append(content_hash)
            
            print(f"Page {i+1}: {preview}")
        
        # Check for duplicates
        print("\nDuplicate analysis:")
        duplicates = []
        for i in range(len(content_hashes)):
            for j in range(i+1, len(content_hashes)):
                if content_hashes[i] == content_hashes[j]:
                    duplicates.append((i+1, j+1))
        
        if duplicates:
            print(f"Found duplicates between pages: {duplicates}")
        else:
            print("No duplicate pages found")
            
        doc.close()
    except Exception as e:
        print(f"Error: {e}")

detailed_content_check()