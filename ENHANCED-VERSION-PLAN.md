# PDF Merger Enhanced Version - Implementation Plan

## Overview
This document outlines the plan for creating an enhanced version of the PDF Merger application with PDF live preview and page trimming/selection capabilities.

## Project Goals
- Keep the current basic version stable and functional
- Create an enhanced version with advanced features:
  - PDF page preview (thumbnails)
  - Individual page selection
  - Page range trimming
  - Visual page reordering
  - Per-file page management

## Architecture Strategy

### Version Separation
- **Current Version**: `pdf-merger` (simple, lightweight)
- **Enhanced Version**: `pdf-merger-pro` or `pdf-merger-enhanced`
  - Separate binary target
  - Shared core merge library
  - Enhanced UI with preview capabilities

### Project Structure
```
pdf-merger-rust/
├── src/
│   ├── lib.rs              # Core merge functionality (shared)
│   ├── main.rs             # Basic version (current)
│   └── main_enhanced.rs    # Enhanced version (new)
├── src/preview/
│   ├── mod.rs              # Preview module
│   ├── renderer.rs         # PDF to image conversion
│   ├── cache.rs            # Thumbnail caching
│   └── page_selector.rs    # Page selection logic
├── Cargo.toml              # Updated with new dependencies
└── docs/
    └── ENHANCED-VERSION-PLAN.md  # This file
```

---

## Phase 1: Research & Foundation (Week 1)

### Task 1.1: Research PDF Rendering Libraries
**Objective**: Evaluate and choose the best PDF rendering library for Rust

**Options to evaluate**:
1. **pdfium-render** - Google's PDFium wrapper
   - Pros: High quality, well-maintained, cross-platform
   - Cons: Large binary size, external dependency

2. **pdf-render** - Pure Rust (using lopdf + image)
   - Pros: No native dependencies, smaller
   - Cons: Limited rendering quality for complex PDFs

3. **mupdf-rs** - MuPDF bindings
   - Pros: Fast, excellent rendering
   - Cons: Complex build process, licensing considerations

**Deliverable**: Decision document on which library to use

**Recommended**: Start with `pdfium-render` for best quality

### Task 1.2: Set Up Enhanced Version Structure
**Objective**: Create separate binary target without breaking current version

**Steps**:
1. Update `Cargo.toml` with new binary target
2. Create `src/main_enhanced.rs` with basic structure
3. Add conditional compilation for preview features
4. Verify both versions build successfully

**Deliverable**: Two working binaries (`pdf-merger` and `pdf-merger-enhanced`)

### Task 1.3: Add PDF Rendering Dependency
**Objective**: Integrate chosen PDF rendering library

**Steps**:
1. Add dependency to `Cargo.toml`
2. Create `src/preview/mod.rs` module
3. Write basic test to render a single PDF page
4. Verify rendering works on Windows

**Deliverable**: Working proof-of-concept: PDF page → PNG image

---

## Phase 2: Data Model & Core Preview Logic (Week 2)

### Task 2.1: Extend Data Model
**Objective**: Add data structures to support page-level operations

**New structures**:
```rust
struct PdfFileInfo {
    path: PathBuf,
    page_count: u32,
    selected_pages: Vec<u32>,  // Which pages to include
    thumbnails: HashMap<u32, Option<Image>>,  // Cached thumbnails
    error: Option<String>,
}

struct PageInfo {
    file_index: usize,
    page_number: u32,
    is_selected: bool,
}
```

**Steps**:
1. Define new data structures
2. Update message types for page operations
3. Add page selection/deselection logic
4. Implement page range parser (e.g., "1-5,8,10")

**Deliverable**: Data model that can track per-page selections

### Task 2.2: Implement PDF Info Extraction
**Objective**: Load PDFs and extract metadata without rendering

**Steps**:
1. Create function to get page count from PDF
2. Extract basic page dimensions
3. Detect pages with content vs. blank pages
4. Add error handling for corrupted PDFs

**Deliverable**: Function that returns `PdfFileInfo` for any PDF

### Task 2.3: Thumbnail Rendering System
**Objective**: Convert PDF pages to thumbnail images efficiently

**Steps**:
1. Implement page → image conversion (target: 200x300px thumbnails)
2. Add background thread for async rendering
3. Implement LRU cache for thumbnails
4. Add progress callback for long operations

**Deliverable**: Fast thumbnail generation with caching

---

## Phase 3: Preview UI Components (Week 3)

### Task 3.1: File List with Page Count
**Objective**: Enhance current file list to show page information

**Changes**:
- Show page count next to file size: "document.pdf (15 pages, 2.3 MB)"
- Add expand/collapse button to show pages
- Display loading indicator while counting pages

**Deliverable**: Enhanced file list with page info

### Task 3.2: Page Thumbnail Grid View
**Objective**: Create expandable thumbnail view for each file

**UI Design**:
```
📄 document.pdf (15 pages, 2.3 MB)     [v Expand]
    ┌─────────────────────────────────────────┐
    │ [✓] Page 1  [✓] Page 2  [ ] Page 3     │
    │  [thumb]     [thumb]     [thumb]        │
    │                                          │
    │ [✓] Page 4  [✓] Page 5  [✓] Page 6     │
    │  [thumb]     [thumb]     [thumb]        │
    └─────────────────────────────────────────┘
    Select: [All] [None] [Range: 1-5]
```

**Steps**:
1. Create collapsible section widget
2. Implement grid layout for thumbnails
3. Add checkboxes for page selection
4. Show loading placeholders while rendering

**Deliverable**: Interactive thumbnail grid for each PDF

### Task 3.3: Page Range Selection UI
**Objective**: Add quick selection tools

**Features**:
- "Select All" / "Select None" buttons
- Page range input: "1-5, 8, 10-15"
- "Every Nth page" selector
- Visual feedback for selected pages

**Deliverable**: Multiple ways to select pages quickly

### Task 3.4: Page Preview Dialog
**Objective**: Full-size page preview on click

**Features**:
- Click thumbnail → show full page preview
- Navigation: Previous/Next page buttons
- Zoom controls
- Toggle selection from preview

**Deliverable**: Modal dialog with full page preview

---

## Phase 4: Page Management Features (Week 4)

### Task 4.1: Individual Page Reordering
**Objective**: Drag and drop pages across files

**UI Design**:
- Drag page thumbnails to reorder
- Visual drop zones between pages
- Cross-file dragging support
- Undo/redo for page operations

**Deliverable**: Fully interactive page reordering

### Task 4.2: Page Operations Menu
**Objective**: Context menu for page actions

**Actions**:
- Remove page
- Duplicate page
- Rotate page (90°, 180°, 270°)
- Extract page to separate file
- Copy/paste pages

**Deliverable**: Right-click context menu for pages

### Task 4.3: Batch Page Operations
**Objective**: Apply operations to multiple pages

**Features**:
- Multi-select pages (Ctrl+click, Shift+click)
- Batch remove/rotate/duplicate
- Apply to all pages in file
- Apply to selected pages across files

**Deliverable**: Efficient bulk page operations

---

## Phase 5: Enhanced Merge Logic (Week 5)

### Task 5.1: Page-Level Merge Function
**Objective**: Update merge logic to handle page selection

**Changes**:
```rust
// Old: merge entire files
merge_pdfs_with_progress(files: Vec<PathBuf>, ...)

// New: merge selected pages
merge_pages_with_progress(
    pages: Vec<(PathBuf, Vec<u32>)>,  // File + selected pages
    ...
)
```

**Steps**:
1. Modify `lib.rs` merge function
2. Support page subset extraction
3. Maintain page order as specified
4. Handle edge cases (empty selections, duplicates)

**Deliverable**: Page-aware merge function

### Task 5.2: Merge Preview Summary
**Objective**: Show what will be merged before merging

**UI**:
```
Merge Summary:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Source Files: 3
Total Pages Selected: 42
Output Size: ~5.2 MB (estimated)

Pages to merge:
  • document1.pdf: Pages 1-5, 8, 10 (7 pages)
  • document2.pdf: Pages 1-20 (20 pages)
  • document3.pdf: Pages 3, 5, 7-21 (15 pages)

[Cancel] [Proceed with Merge]
```

**Deliverable**: Confirmation dialog with merge details

### Task 5.3: Progress Tracking Enhancement
**Objective**: More detailed progress for page-level operations

**Features**:
- Per-page progress tracking
- Current file and page being processed
- Time remaining estimation
- Cancellation support

**Deliverable**: Enhanced progress display

---

## Phase 6: Optimization & Polish (Week 6)

### Task 6.1: Performance Optimization
**Objective**: Ensure smooth performance with large PDFs

**Focus areas**:
- Lazy loading: Only render visible thumbnails
- Virtual scrolling for large page lists
- Async thumbnail generation
- Memory management for cached images
- Background thread for rendering

**Deliverable**: Smooth UI with 100+ page PDFs

### Task 6.2: Persistent Settings
**Objective**: Remember user preferences

**Settings to save**:
- Thumbnail size preference
- Default page selection mode
- Last output directory
- Window size/position
- Recently opened files

**Deliverable**: Settings persistence across sessions

### Task 6.3: Keyboard Shortcuts Enhancement
**Objective**: Add shortcuts for new features

**New shortcuts**:
- **Ctrl+E**: Expand/collapse selected file
- **Ctrl+A**: Select all pages in current file
- **Ctrl+Shift+A**: Select all pages in all files
- **Ctrl+I**: Invert selection
- **Space**: Toggle page selection
- **Ctrl+P**: Open preview for selected page
- **Del**: Remove selected pages

**Deliverable**: Comprehensive keyboard navigation

### Task 6.4: Error Handling & Validation
**Objective**: Robust error handling for edge cases

**Scenarios**:
- Corrupted PDF pages
- Out of memory during rendering
- Invalid page selections
- Cancelled operations
- Disk space issues

**Deliverable**: Graceful error handling

---

## Phase 7: Testing & Documentation (Week 7)

### Task 7.1: Unit Tests
**Objective**: Comprehensive test coverage

**Test areas**:
- Page selection logic
- Page range parsing
- Thumbnail caching
- Merge with page selection
- Data model operations

**Deliverable**: >80% test coverage for new code

### Task 7.2: Integration Tests
**Objective**: End-to-end testing

**Test scenarios**:
- Select pages from multiple PDFs and merge
- Reorder pages and verify output
- Handle large PDFs (100+ pages)
- Concurrent rendering operations
- Memory leak testing

**Deliverable**: Automated integration test suite

### Task 7.3: User Documentation
**Objective**: Document enhanced features

**Documents to create**:
- **USER-GUIDE-ENHANCED.md**: How to use preview and page selection
- **PERFORMANCE-TIPS.md**: Best practices for large PDFs
- Update **README.md** with enhanced version info

**Deliverable**: Complete user documentation

### Task 7.4: Performance Benchmarks
**Objective**: Document performance characteristics

**Benchmarks**:
- Thumbnail generation time vs. page count
- Memory usage with different cache sizes
- Merge speed with page selection vs. full files
- UI responsiveness metrics

**Deliverable**: Performance benchmark report

---

## Phase 8: Release Preparation (Week 8)

### Task 8.1: Cross-Platform Testing
**Objective**: Verify functionality on all platforms

**Platforms**:
- Windows 10/11
- macOS (if available)
- Linux (Ubuntu/Fedora)

**Test cases**:
- Build process
- PDF rendering quality
- UI responsiveness
- File dialog compatibility

**Deliverable**: Verified builds for all platforms

### Task 8.2: Binary Optimization
**Objective**: Minimize file size and optimize performance

**Steps**:
- Enable LTO (Link Time Optimization)
- Strip debug symbols
- Optimize dependencies
- Consider dynamic linking for large deps

**Deliverable**: Optimized release binaries

### Task 8.3: Release Documentation
**Objective**: Prepare for release

**Documents**:
- **CHANGELOG.md**: Feature list for enhanced version
- **MIGRATION-GUIDE.md**: Differences from basic version
- Build instructions for enhanced version
- System requirements

**Deliverable**: Complete release documentation

### Task 8.4: Version Tagging & Release
**Objective**: Create GitHub release

**Steps**:
1. Tag version: `v1.0.0-enhanced`
2. Build release binaries
3. Create GitHub release
4. Upload binaries
5. Write release notes

**Deliverable**: Public release on GitHub

---

## Technical Specifications

### Minimum Requirements (Enhanced Version)
- **Memory**: 2 GB RAM (4 GB recommended)
- **Storage**: 100 MB for application + cache space
- **Display**: 1280x720 minimum resolution
- **OS**: Windows 10+, macOS 10.15+, Linux (recent kernels)

### Dependencies (New)
```toml
[dependencies]
# Existing dependencies...
pdfium-render = "0.8"      # PDF rendering
image = "0.24"             # Image processing
lru = "0.12"               # Thumbnail caching
rayon = "1.8"              # Parallel processing
serde = "1.0"              # Settings persistence
serde_json = "1.0"         # Settings format
dirs = "5.0"               # User directories
```

### Performance Targets
- **Thumbnail generation**: < 200ms per page (average)
- **UI responsiveness**: 60 FPS
- **Memory usage**: < 500 MB for typical use (< 50 PDFs)
- **Cache size**: Configurable, default 100 MB
- **Merge speed**: Similar to basic version

---

## Risk Assessment

### Technical Risks
1. **PDF Rendering Quality**
   - Risk: Poor rendering for complex PDFs
   - Mitigation: Thorough testing, fallback options

2. **Performance with Large PDFs**
   - Risk: Slow/unresponsive UI
   - Mitigation: Lazy loading, caching, background threads

3. **Memory Usage**
   - Risk: Excessive memory consumption
   - Mitigation: LRU cache, configurable limits

4. **Cross-Platform Compatibility**
   - Risk: Platform-specific rendering issues
   - Mitigation: Early multi-platform testing

### Project Risks
1. **Scope Creep**
   - Risk: Feature additions延長 timeline
   - Mitigation: Strict phase boundaries, MVP focus

2. **Dependency Issues**
   - Risk: pdfium build problems on Windows
   - Mitigation: Pre-built binaries, alternative libraries

3. **User Experience Complexity**
   - Risk: Enhanced version too complicated
   - Mitigation: Keep basic version available, good UX design

---

## Success Criteria

### Must Have (MVP)
- ✅ View page count for each PDF
- ✅ Generate and display page thumbnails
- ✅ Select/deselect individual pages
- ✅ Merge only selected pages
- ✅ Basic page reordering

### Should Have
- ✅ Page range input (e.g., "1-5,8,10")
- ✅ Full-page preview on click
- ✅ Thumbnail caching
- ✅ Progress tracking for rendering

### Nice to Have
- ⭕ Page rotation
- ⭕ Cross-file page dragging
- ⭕ Page duplication
- ⭕ Batch operations
- ⭕ Settings persistence

---

## Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| 1. Foundation | 1 week | Library choice, project structure, POC |
| 2. Data Model | 1 week | Page tracking, thumbnail rendering |
| 3. Preview UI | 1 week | Thumbnail grid, page selection UI |
| 4. Page Management | 1 week | Reordering, context menu, batch ops |
| 5. Merge Logic | 1 week | Page-aware merge, preview summary |
| 6. Optimization | 1 week | Performance tuning, settings |
| 7. Testing | 1 week | Tests, benchmarks, documentation |
| 8. Release | 1 week | Cross-platform builds, release |

**Total Estimated Time**: 8 weeks for full implementation

**MVP Timeline**: 4-5 weeks (Phases 1-3 + basic Phase 5)

---

## Next Steps

### Immediate Actions
1. ✅ Create this planning document
2. ⏳ Get approval for approach and timeline
3. ⏳ Start Phase 1: Research PDF rendering libraries
4. ⏳ Set up development branch: `feature/enhanced-version`

### Questions to Resolve
- [ ] Which PDF rendering library to use? (pdfium-render recommended)
- [ ] Target platforms? (Windows primary, macOS/Linux secondary?)
- [ ] Memory/performance constraints?
- [ ] Release timeline preferences?
- [ ] MVP vs. full feature set priority?

---

## Notes
- Keep basic version maintained and stable
- Enhanced version is opt-in (separate binary)
- Consider user feedback during Phase 3-4 for UI refinement
- Regular commits and progress updates

---

**Document Version**: 1.0
**Created**: 2025-12-05
**Last Updated**: 2025-12-05
**Status**: Planning Phase
