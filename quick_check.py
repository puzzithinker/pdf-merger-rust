import fitz  # PyMuPDF

def analyze_pdf_brief(filename):
    """Brief analysis of a PDF"""
    try:
        doc = fitz.open(filename)
        print(f"{filename}: {doc.page_count} pages")
        doc.close()
    except Exception as e:
        print(f"{filename}: Error - {e}")

# Check the input files
analyze_pdf_brief("email.pdf")
analyze_pdf_brief("Forms.pdf")
analyze_pdf_brief("merged.pdf")