# Future Breaking Changes (for v2.0.0)

This document outlines potential breaking changes that could be introduced in a future major version bump (v2.0.0) to further modernize and improve the colonnade crate.

## API Improvements

### 1. Split into Modules
- **Current:** Single large `lib.rs` file (1900+ lines)
- **Proposed:** Split into logical modules:
  - `error.rs` - Error types and handling
  - `column.rs` - Column configuration
  - `alignment.rs` - Alignment enums
  - `layout.rs` - Layout algorithm
  - `render.rs` - Rendering logic
- **Benefit:** Better organization, easier maintenance, improved documentation

### 2. Consistent Builder Pattern
- **Current:** Some methods return `&mut Self`, others don't
- **Proposed:** All configuration methods should return `&mut Self` for chainability
- **Benefit:** More consistent API, better ergonomics

### 3. Improved Error Messages
- **Current:** Generic error messages with limited context
- **Proposed:** Implement detailed `Display` for all error types
- **Benefit:** Better user experience, easier debugging

## Performance Improvements

### 1. Optimize String Building
- **Current:** Excessive use of `String::new()` + `+=` concatenation
- **Proposed:** Use `String::with_capacity()` and `write!` macro
- **Benefit:** Reduced allocations, better performance

### 2. Fix Word Splitting Algorithm
- **Current:** Uses `Vec::remove(0)` which is O(n)
- **Proposed:** Use `VecDeque` or iterator-based approach
- **Benefit:** O(1) removal from front, better performance for large texts

### 3. Cache Character Widths
- **Current:** `true_width()` recalculates grapheme counts repeatedly
- **Proposed:** Cache results or use more efficient calculation
- **Benefit:** Reduced computation for repeated strings

### 4. Lazy ANSI Stripping
- **Current:** Strips ANSI codes from every cell unconditionally
- **Proposed:** Only strip if ANSI codes are detected
- **Benefit:** Reduced overhead when no color codes present

## Code Quality Improvements

### 1. Use `&[T]` instead of `&Vec<T>`
- **Current:** Many function signatures use `&Vec<T>`
- **Proposed:** Use `&[T]` slices for more flexibility
- **Benefit:** More idiomatic Rust, better compatibility

### 2. Remove Redundant Field Names
- **Current:** `Column::default(index: usize)` uses `index: index`
- **Proposed:** Use shorthand field initialization
- **Benefit:** Cleaner code

### 3. Use Associated Constants
- **Current:** Uses `usize::max_value()`
- **Proposed:** Use `usize::MAX`
- **Benefit:** More idiomatic, consistent with modern Rust

### 4. Simplify Match Statements
- **Current:** Some single-pattern matches that could be `if let`
- **Proposed:** Use `if let` for single pattern matching
- **Benefit:** More readable code

## New Features (Non-Breaking)

These could be added without breaking changes:

1. **Border support** - Add optional borders around tables/cells
2. **Custom separators** - Allow customization of column separators
3. **Row grouping** - Support for grouping rows visually
4. **Header/foter rows** - Special styling for header/footer rows
5. **Conditional formatting** - Format cells based on content
6. **CSV import/export** - Helper methods for CSV compatibility
7. **Markdown export** - Generate markdown tables
8. **HTML export** - Generate HTML tables

## Testing Improvements

1. **Property-based testing** - Add proptest for more robust testing
2. **Benchmarks** - Add criterion.rs benchmarks
3. **Fuzz testing** - Add cargo-fuzz for edge case testing
4. **More edge case tests** - Empty tables, single cells, very wide characters

## Documentation Improvements

1. **Architecture documentation** - Explain how the layout algorithm works
2. **More examples** - Show different use cases
3. **Performance guide** - Best practices for performance
4. **Migration guide** - For future breaking changes

---

*This document is a work-in-progress and will be updated as we identify more opportunities for improvement.*
