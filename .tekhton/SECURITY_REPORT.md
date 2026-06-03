## Summary

M49.2 restructures `leiden_recursive` to fix an O(max_iter^depth) algorithmic hang, un-ignores the regression test, adds a brute-force termination sweep, bumps the WASM package.json version, and updates CHANGELOG/MIGRATION_NOTES. All changes are confined to internal algorithm logic and test code with no authentication, cryptography, network communication, or user-controlled input handling. RNG usage follows project rules (`StdRng::seed_from_u64`, explicit seed from config). Tracing log calls interpolate only numeric internal values. No security issues were found.

## Findings

None

## Verdict

CLEAN
