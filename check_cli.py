import fitz  # PyMuPDF

def check_cli_result():
    """Check CLI result"""
    try:
        doc = fitz.open("cli_test.pdf")
        print(f"CLI test result: {doc.page_count} pages")
        
        # Check first few pages
        for i in range(min(3, doc.page_count)):
            page = doc[i]
            text = page.get_text()
            lines = text.strip().split('\n')
            print(f"Page {i+1}: {len(lines)} lines of content")
            if lines:
                print(f"  First line: {lines[0][:50]}")
        
        doc.close()
    except Exception as e:
        print(f"CLI test error: {e}")

check_cli_result()