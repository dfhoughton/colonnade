# Change Log

## Unreleased (2.0.0)
* **Breaking Changes:**
  - Made all `Column` fields private (except `index`) with getter methods
  - Renamed all `Column` setter methods to use `set_` prefix (e.g., `left_margin()` -> `set_left_margin()`)
  - This provides better encapsulation and avoids name collisions with getters
* **Performance Improvements:**
  - Optimized word splitting algorithm by using `VecDeque` instead of `Vec` for O(1) removal from front
  - Optimized string building by using `String::with_capacity()` and `push_str()` instead of `+=` concatenation
  - Reduced redundant `true_width()` calls by caching results locally
* **Non-Breaking Improvements:**
  - Complete code refactoring with module system:
    - Split into `error`, `alignment`, `column`, `text`, and main `lib` modules
    - Improved code organization and maintainability
  - Added type aliases for complex types: `Cell`, `Line`, `MaceratedRow`, `Macerated`
  - Improved `reset()` method to return `&mut Self` for chainability
  - Enhanced error messages with detailed descriptions
  - Updated function signatures to use type aliases
  - Replaced `&Vec<T>` with `&[T]` slices where appropriate
  - Fixed clippy warnings throughout the codebase
  - Updated documentation examples

## 1.3.4 *2023-11-21*
* Upgraded to Rust 2024 edition
* Updated and pinned all dependencies
* Removed `extern crate` declarations (Rust 2018+ style)
* Added clippy linting configuration
* Added rustfmt configuration
* Fixed clippy warnings throughout the codebase
* Updated .gitignore with common patterns

## 1.3.3 *2023-10-9*
* merged PR from @Paradyx
## 1.3.2 *2023-1-28*
* bumped regex version to make cargo-audit happier
## 1.3.1 *2020-2-29*
* added justify alignment
## 1.3.0 *2020-2-8*
* added nbsp feature, which treats non-breaking whitespace -- u00A0 -- like non-whitespace
## 1.2.0
* better unicode and ansi escape handling
## 1.1.3
* fixed another bug
## 1.1.2
* remove undocumented `Colonnade#dump` method
* added the `Colonnade#width` method
## 1.1.1
* fixed index out of bounds bug in layout
## 1.1.0
This change actually severely breaks the API, but since no one else is using this crate yet
I figure it's safe to deviate a little from semantic versioning best practices.
* made all configuration methods chainable
* made `addrow` utf8-safe
* change `macerate` so it returns rows of lines of cells, the better to keep the mapping between
rows added and pieces returned
* made `lay_out` private and added `reset` method to provide equivalent functionality
* gave `tabulate` and `macerate` still more flexible signatures
* allow variable-length rows so long as no row has more columns than the colonnade
* fix wrapping/hyphenation for wide characters
* better mark when the configuration is dirty and layout needs to be redone
## 1.0.0
* simpler, more concise API
* vertical alignment
## 0.2.0
* expanded `tabulate` and `layout` signatures so they accept anything with trait `ToString`
* added the `macerate` method to facilitate adding color
* added padding, because color makes this necessary
## 0.1.1
* added this file
* fixed a typo in the documentation
