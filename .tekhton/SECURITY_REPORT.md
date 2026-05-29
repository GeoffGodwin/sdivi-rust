## Summary
This change adds a new `class_hierarchy` pattern category consisting entirely of compile-time static string constants (`NODE_KINDS`, `CATALOG_ENTRIES` entries, `CATEGORIES` index references, and `PATTERN_KINDS` additions in five language extractors). No user input is processed, no I/O is performed, no network calls are made, no authentication or authorization logic is touched, and no cryptographic material is involved. The new `impl_item` node kind added to the Rust extractor's `PATTERN_KINDS` passes through the existing 256-byte truncation guard in `collect_hints`, providing the same memory-pressure protection as all other pattern kinds. No new attack surface is introduced.

## Findings
None

## Verdict
CLEAN
