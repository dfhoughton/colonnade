# Change Log

## 1.1.0
This change actually severely breaks the API, but since no one else is using this crate yet
I figure it's safe to deviate a little from semantic versioning best practices.
* made all configuration methods chainable
* made addrow utf8-safe
* change macerate so it returns rows of lines of cells, the better to keep the mapping between
rows added and pieces returned
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
