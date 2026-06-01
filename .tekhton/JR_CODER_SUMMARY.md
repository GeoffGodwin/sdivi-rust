# Junior Coder Summary

## What Was Fixed

- **`bindings/sdivi-wasm/tests/typecheck/negative.ts` lines 33–64**: Corrected four `// @ts-expect-error` directive placements.
  - **Case 1** (line 34): Moved explanatory text before the `@ts-expect-error` directive so it applies to the `await init()` call on line 35.
  - **Case 2a** (line 44): Moved explanatory text before the `@ts-expect-error` directive so it applies to the `const _badEdgeWeights...` assignment on line 45.
  - **Case 2b** (line 53): Collapsed multi-line object literal to single line and placed `@ts-expect-error` immediately before it so the directive applies to line 54.
  - **Case 3** (line 63): Moved explanatory text before the `@ts-expect-error` directive so it applies to the `const _badBracket...` assignment on line 64.

TypeScript `@ts-expect-error` applies only to the immediately next **non-comment** physical line. Prior to the fix, continuation comments on N+1 were receiving the suppression (becoming unused → TS2578) while the actual type errors on N+2 were not suppressed and reported as real errors. The fix ensures each directive is positioned to suppress the line it's intended for.

## Files Modified

- `bindings/sdivi-wasm/tests/typecheck/negative.ts`
