# Human Action Required

The pipeline identified items that need your attention. Review each item
and check it off when addressed. The pipeline will display a banner until
all items are resolved.

## Action Items

- [ ] **`CLAUDE.md` § Code Conventions — add doc comment placement rule.** Inserting a `pub fn` immediately before an existing documented `pub fn` silently re-attaches the existing function's doc block to the new function. This has recurred across three milestone runs. Proposed addition: *"When inserting a function or item immediately before an existing documented item, verify that a non-`///` line separates the two doc blocks. A `///` comment block attaches to the next item — displacing an existing comment is a silent correctness bug, not a style issue."*

## Resolved

