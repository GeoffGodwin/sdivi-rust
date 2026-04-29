# Junior Coder Summary — 2026-04-29

## What Was Fixed

1. **Staleness fix: ACTIVE_TREES doc comment** (`crates/sdi-parsing/src/lib.rs:16-19`)
   - Updated doc comment to accurately reflect that the counter tracks active `parse_file` invocations, not live `Tree` objects
   - Changed from: "Language adapters increment this on tree creation and decrement it on drop."
   - Changed to: "Incremented at the start of `parse_file` and decremented after the `PARSER.with` closure returns; tracks active `parse_file` invocations, not live `Tree` objects."
   - This reflects the actual implementation: `fetch_add` fires before the `PARSER.with` closure (before tree creation) and `fetch_sub` fires after the closure returns (after tree drop).

## Files Modified

- `crates/sdi-parsing/src/lib.rs`

## Verification

- ✓ `cargo check -p sdi-parsing` passes without errors or warnings
- ✓ No other files were modified
- ✓ Change is purely documentation, no logic changes
