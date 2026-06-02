# Security Notes

Generated: 2026-06-01 18:02:36

## Non-Blocking Findings (MEDIUM/LOW)
- [LOW] [category:A06] [.github/workflows/wasm.yml:171] fixable:yes — `npm install` step adds `--no-audit`, which suppresses the npm advisory check for the TypeScript dev-tool installation. TypeScript is a pinned build-time dependency (`typescript@5.5.4`) and not a runtime dependency, so the blast radius is minimal; however, suppressing the audit removes one automated advisory signal. Suggestion: replace `--no-audit` with `--audit-level=none` (which still performs the audit request but does not fail on any advisory level) if the goal is suppressing noisy exit-code behavior, or accept the low risk given the version pin.
