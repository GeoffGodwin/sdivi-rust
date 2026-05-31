//! CALL_DISPATCH registry disjointness and precedence tests.
//!
//! Verifies that the `CALL_DISPATCH` registry in `classify_hint` resolves each
//! callee string to exactly the expected category, and that any callee string
//! matching two or more categories' regexes is documented in `KNOWN_OVERLAPS`.
//!
//! **Adding a new M35+ category that uses `call_expression`:**
//! 1. Insert it at its precedence slot in `CALL_DISPATCH` (do not append).
//! 2. Add corpus entries for the new category below.
//! 3. For any callee that legitimately matches two categories, add a
//!    `KNOWN_OVERLAPS` entry with the winner named.

use sdivi_patterns::queries::{
    async_patterns, classify_hint, collection_pipelines, data_access, framework_hooks, logging,
    schema_validation, state_store,
};
use sdivi_patterns::PatternHintInput;

fn hint(node_kind: &str, text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: node_kind.to_string(),
        text: text.to_string(),
    }
}

/// Collect every dispatch category that matches this callee text.
/// At M40, P1/P4/P5/P6/P8/P9/P10 are active; future milestones extend this list.
fn all_matching_categories(text: &str, language: &str) -> Vec<&'static str> {
    let mut matched = Vec::new();
    if async_patterns::matches_callee(text, language) {
        matched.push("async_patterns");
    }
    if schema_validation::matches_callee(text, language) {
        matched.push("schema_validation");
    }
    if state_store::matches_callee(text, language) {
        matched.push("state_store");
    }
    if framework_hooks::matches_callee(text, language) {
        matched.push("framework_hooks");
    }
    if logging::matches_callee(text, language) {
        matched.push("logging");
    }
    if data_access::matches_callee(text, language) {
        matched.push("data_access");
    }
    if collection_pipelines::matches_callee(text, language) {
        matched.push("collection_pipelines");
    }
    matched
}

// ── KNOWN_OVERLAPS ─────────────────────────────────────────────────────────────
// Callee strings that match two or more categories' `matches_callee` functions.
// Format: (callee_text, language, winning_category, losing_category).
// The test asserts the winner is what CALL_DISPATCH returns (first-match-wins).
// Future milestones MUST add entries here before introducing overlapping regexes.
const KNOWN_OVERLAPS: &[(&str, &str, &str, &str)] = &[
    // `fetch().catch()` outer node: async_patterns (P1) regex `\.(catch)\(` AND
    // data_access (P9) regex `^(fetch|axios)\b` both match.
    // async_patterns wins because P1 < P9 in CALL_DISPATCH order.
    (
        "fetch(url).catch(err => {})",
        "javascript",
        "async_patterns",
        "data_access",
    ),
    // `logger.get(x)`: logging (P8) regex `^(console|logger|log)\.` AND
    // data_access (P9) regex `\b(get)\(` both match (word boundary before `get(`).
    // logging wins because P8 < P9 in CALL_DISPATCH order. Caller intent (logger
    // object) dominates over method-name pattern matching.
    ("logger.get(\"x\")", "typescript", "logging", "data_access"),
    // React-Redux hooks match both state_store (P5) `^use(Selector|Dispatch|Store)\b`
    // AND framework_hooks (P6) `^use[A-Z]`. state_store wins (P5 < P6) — more
    // specific wins over the broad hook-ecosystem regex.
    (
        "useSelector(s => s.user)",
        "typescript",
        "state_store",
        "framework_hooks",
    ),
    (
        "useDispatch()",
        "typescript",
        "state_store",
        "framework_hooks",
    ),
    ("useStore()", "javascript", "state_store", "framework_hooks"),
];

// ── Per-category corpus ────────────────────────────────────────────────────────
// (callee_text, language, expected_category)
// expected_category = "" means classify_hint must return an empty Vec.
const CORPUS: &[(&str, &str, &str)] = &[
    // P1: async_patterns — Promise chains (TypeScript/JavaScript only)
    ("promise.then(resolve)", "typescript", "async_patterns"),
    (
        "fetch(url).catch(err => {})",
        "javascript",
        "async_patterns",
    ),
    ("p.finally(() => {})", "typescript", "async_patterns"),
    // P6: framework_hooks — ^use[A-Z] callee regex (TypeScript/JavaScript only)
    ("useState(0)", "typescript", "framework_hooks"),
    ("useEffect(fn, [])", "typescript", "framework_hooks"),
    ("useMemo(() => v, [])", "javascript", "framework_hooks"),
    ("useCustomHook(opts)", "typescript", "framework_hooks"),
    ("useAuth()", "javascript", "framework_hooks"),
    // Negative: lowercase second char or wrong language must NOT match framework_hooks
    ("user()", "typescript", ""),
    ("useState(0)", "python", ""),
    // P8: logging — per-language tables
    // P8>P9 overlap: logging callee with a data_access verb method name; logging wins.
    ("logger.get(\"x\")", "typescript", "logging"),
    ("console.log(\"x\")", "typescript", "logging"),
    ("logger.info(\"x\")", "typescript", "logging"),
    ("log.debug(\"x\")", "javascript", "logging"),
    ("logging.info(\"x\")", "python", "logging"),
    ("print(x)", "python", "logging"),
    ("fmt.Println(\"x\")", "go", "logging"),
    ("fmt.Printf(\"%v\", x)", "go", "logging"),
    ("fmt.Errorf(\"msg\")", "go", "logging"),
    ("fmt.Fprintf(w, \"x\")", "go", "logging"),
    ("System.out.println(\"x\")", "java", "logging"),
    ("logger.info(\"x\")", "java", "logging"),
    ("LOG.debug(\"x\")", "java", "logging"),
    // P5: state_store — ^-anchored factory calls (TS/JS only)
    ("createSlice({})", "typescript", "state_store"),
    ("configureStore({})", "typescript", "state_store"),
    ("createStore(rootReducer)", "javascript", "state_store"),
    ("atom(0)", "typescript", "state_store"),
    ("makeAutoObservable(this)", "typescript", "state_store"),
    ("signal(0)", "typescript", "state_store"),
    ("createSignal(0)", "typescript", "state_store"),
    ("create((set) => ({}))", "typescript", "state_store"),
    // React-Redux hooks: state_store (P5) beats framework_hooks (P6)
    ("useSelector(s => s.user)", "typescript", "state_store"),
    ("useDispatch()", "typescript", "state_store"),
    ("useStore()", "javascript", "state_store"),
    // Negatives: member-access calls must NOT match state_store
    ("prisma.user.create(data)", "typescript", ""),
    // P4: schema_validation — namespace-anchored (Zod/Yup/Valibot/Superstruct) + .safeParse(
    ("z.object({})", "typescript", "schema_validation"),
    ("z.string()", "javascript", "schema_validation"),
    ("yup.object().shape({})", "typescript", "schema_validation"),
    ("v.object({})", "javascript", "schema_validation"),
    ("s.object({})", "typescript", "schema_validation"),
    (
        "UserSchema.safeParse(input)",
        "typescript",
        "schema_validation",
    ),
    ("Field(default=0)", "python", "schema_validation"),
    ("constr(min_length=1)", "python", "schema_validation"),
    ("conint(gt=0)", "python", "schema_validation"),
    // Negative: bare method calls or non-schema callees must NOT match schema_validation
    ("SomeClass.parse(x)", "typescript", ""), // no namespace prefix, not .safeParse(
    // M36.1: decorators — node-kind-only via category_for_node_kind, not CALL_DISPATCH.
    // A `decorator` hint routed as `call_expression` should NOT match any dispatch entry.
    ("@Injectable()", "typescript", ""),
    // P9: data_access — per-language tables
    ("fetch(\"/api/users\")", "typescript", "data_access"),
    ("axios.get(\"/api\")", "typescript", "data_access"),
    ("db.query(sql)", "go", "data_access"),
    ("sql.Open(\"postgres\", dsn)", "go", "data_access"),
    ("cursor.execute(sql)", "python", "data_access"),
    ("requests.get(url)", "python", "data_access"),
    // P10: collection_pipelines — member-call regex (TS/JS and Go/Java)
    ("xs.map(f)", "typescript", "collection_pipelines"),
    ("xs.filter(p)", "javascript", "collection_pipelines"),
    ("xs.reduce(g, 0)", "typescript", "collection_pipelines"),
    ("data.flatMap(fn)", "typescript", "collection_pipelines"),
    ("items.forEach(cb)", "javascript", "collection_pipelines"),
    ("xs.find(p)", "typescript", "collection_pipelines"),
    ("xs.findIndex(p)", "javascript", "collection_pipelines"),
    ("xs.some(p)", "typescript", "collection_pipelines"),
    ("xs.every(p)", "typescript", "collection_pipelines"),
    ("xs.flat()", "javascript", "collection_pipelines"),
    // Negative: data_access methods must not match collection_pipelines
    ("client.read(buf)", "typescript", "data_access"),
    // Unrecognised — classify_hint must return empty Vec (represented as "")
    ("Math.max(a, b)", "typescript", ""),
    ("len(x)", "python", ""),
    ("os.Exit(1)", "go", ""),
    ("MyClass.method()", "java", ""),
];

// ── Registry resolution test ───────────────────────────────────────────────────

/// Each corpus entry must resolve to the expected category (or empty) via `classify_hint`.
/// This verifies that the CALL_DISPATCH registry produces identical results to the
/// prior if-chain, and that each category is reachable through the registry.
#[test]
fn corpus_resolves_to_expected_category() {
    for &(text, lang, expected) in CORPUS {
        let result = classify_hint(&hint("call_expression", text), lang);
        if expected.is_empty() {
            assert!(
                result.is_empty(),
                "({text:?}, {lang:?}): expected empty Vec but got {result:?}"
            );
        } else {
            assert_eq!(
                result,
                vec![expected],
                "({text:?}, {lang:?}): expected [{expected:?}] but got {result:?}"
            );
        }
    }
}

/// Same corpus but routed as `call` (Python/Go node kind) — must behave identically.
#[test]
fn corpus_resolves_identically_for_call_node_kind() {
    for &(text, lang, _expected) in CORPUS {
        let via_call = classify_hint(&hint("call", text), lang);
        let via_call_expr = classify_hint(&hint("call_expression", text), lang);
        assert_eq!(
            via_call, via_call_expr,
            "({text:?}, {lang:?}): `call` and `call_expression` arms must behave identically"
        );
    }
}

// ── Disjointness enforcement ───────────────────────────────────────────────────

/// For every corpus entry, if more than one category's `matches_callee` returns true,
/// the overlap must be documented in `KNOWN_OVERLAPS`.  Undocumented overlaps fail
/// the build — add an entry to `KNOWN_OVERLAPS` with the winner named.
#[test]
fn no_undocumented_overlaps_in_corpus() {
    for &(text, lang, _expected) in CORPUS {
        let matched = all_matching_categories(text, lang);
        if matched.len() <= 1 {
            continue; // disjoint — no overlap to document
        }
        let documented = KNOWN_OVERLAPS
            .iter()
            .any(|&(ot, ol, _, _)| ot == text && ol == lang);
        assert!(
            documented,
            "Undocumented overlap for ({text:?}, {lang:?}): matches {matched:?}. \
             Add an entry to KNOWN_OVERLAPS with the winning category named."
        );
    }
}

/// For each documented overlap, the CALL_DISPATCH first-match winner must equal
/// the `winning_category` field in `KNOWN_OVERLAPS`.
#[test]
fn known_overlaps_winner_matches_dispatch_order() {
    for &(text, lang, winner, loser) in KNOWN_OVERLAPS {
        let result = classify_hint(&hint("call_expression", text), lang);
        assert_eq!(
            result,
            vec![winner],
            "KNOWN_OVERLAPS ({text:?}, {lang:?}): documented winner is {winner:?} \
             but CALL_DISPATCH returned {result:?}. Update KNOWN_OVERLAPS or fix CALL_DISPATCH order."
        );
        // The loser must also match (otherwise it shouldn't be in KNOWN_OVERLAPS).
        let loser_matches = match loser {
            "async_patterns" => async_patterns::matches_callee(text, lang),
            "schema_validation" => schema_validation::matches_callee(text, lang),
            "state_store" => state_store::matches_callee(text, lang),
            "framework_hooks" => framework_hooks::matches_callee(text, lang),
            "logging" => logging::matches_callee(text, lang),
            "data_access" => data_access::matches_callee(text, lang),
            "collection_pipelines" => collection_pipelines::matches_callee(text, lang),
            other => panic!("KNOWN_OVERLAPS references unknown category {other:?}"),
        };
        assert!(
            loser_matches,
            "KNOWN_OVERLAPS ({text:?}, {lang:?}): loser category {loser:?} does not \
             actually match — remove it from KNOWN_OVERLAPS or fix the regex."
        );
    }
}
