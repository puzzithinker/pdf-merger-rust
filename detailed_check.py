import fitz  # PyMuPDF
import os

def detailed_content_check():
    """Detailed check of merged.pdf content"""
    try:
        doc = fitz.open("merged.pdf")
        print("merged.pdf detailed analysis:")
        print("  Pages: {}".format(doc.page_count))
        print("  File size: {} bytes".format(os.path.getsize("merged.pdf")))
        
        # Extract content from each page
        page_texts = []
        for i in range(doc.page_count):
            page = doc[i]
            text = page.get_text()
            page_texts.append(text)
            print("\nPage {}:".format(i+1))
            print("  Content length: {} characters".format(len(text)))
            # Show first 100 characters
            preview = text[:100].replace('\n', '\\n')
            print("  Preview: '{}'".format(preview))
        
        # Check for duplicates
        print("\nDuplicate check:")
        for i in range(len(page_texts)):
            for j in range(i+1, len(page_texts)):
                if page_texts[i] == page_texts[j]:
                    print("  Pages {} and {} are IDENTICAL".format(i+1, j+1))
                else:
                    print("  Pages {} and {} are DIFFERENT".format(i+1, j+1))
                    # Show difference in length
                    len_diff = abs(len(page_texts[i]) - len(page_texts[j]))
                    print("    Length difference: {} characters".format(len_diff))
        
        doc.close()
    except Exception as e:
        print("Error: {}".format(e))

detailed_content_check()