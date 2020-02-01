# Change Log

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
