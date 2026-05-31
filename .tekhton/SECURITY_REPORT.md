## Summary
M35 adds the `framework_hooks` pattern category: a new module (`framework_hooks.rs`) with a single `LazyLock<Regex>` and a `matches_callee` dispatcher, wired into `CALL_DISPATCH` in `mod.rs` and registered in `categories.rs`. The change introduces no I/O, no network calls, no authentication, no cryptography, no shell execution, and no external user-controlled input paths. The regex `^use[A-Z]` is a compile-time constant anchored at the start, with no quantifiers that could trigger catastrophic backtracking. The overall security posture of the change is sound.

## Findings
None

## Verdict
CLEAN
