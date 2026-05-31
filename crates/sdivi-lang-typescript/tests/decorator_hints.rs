//! Tests for decorator pattern hints in the TypeScript language adapter (M36.1).

use sdivi_lang_typescript::TypeScriptAdapter;
use sdivi_parsing::adapter::LanguageAdapter;
use std::path::Path;

fn parse_ts(source: &str) -> sdivi_parsing::feature_record::FeatureRecord {
    TypeScriptAdapter.parse_file(Path::new("test.ts"), source.to_string())
}

#[test]
fn class_decorator_captured_as_pattern_hint() {
    let record = parse_ts("@Injectable()\nclass AppService {}\n");
    let has_decorator = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "decorator");
    assert!(
        has_decorator,
        "decorator must appear in pattern_hints for @Injectable() class, got hints: {:?}",
        record
            .pattern_hints
            .iter()
            .map(|h| h.node_kind.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn method_decorator_captured_as_pattern_hint() {
    let source = concat!(
        "class AppController {\n",
        "  @Get('/')\n",
        "  getRoot(): string { return 'hello'; }\n",
        "}\n",
    );
    let record = parse_ts(source);
    let has_decorator = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "decorator");
    assert!(
        has_decorator,
        "decorator must appear in pattern_hints for @Get('/') method, got hints: {:?}",
        record
            .pattern_hints
            .iter()
            .map(|h| h.node_kind.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn nestjs_shaped_controller_yields_multiple_decorator_hints() {
    let source = concat!(
        "@Controller('cats')\n",
        "class CatsController {\n",
        "  @Get('/')\n",
        "  findAll(): string { return 'all cats'; }\n",
        "  @Post('/')\n",
        "  create(): string { return 'created'; }\n",
        "}\n",
    );
    let record = parse_ts(source);
    let decorator_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "decorator")
        .count();
    assert!(
        decorator_count >= 3,
        "NestJS controller fixture must yield at least 3 decorator hints \
        (@Controller + 2 route decorators), got: {decorator_count}"
    );
}

#[test]
fn decorator_hint_text_starts_with_at_sign() {
    let record = parse_ts("@Injectable()\nclass AppService {}\n");
    let decorator_hints: Vec<_> = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "decorator")
        .collect();
    assert!(
        !decorator_hints.is_empty(),
        "must have at least one decorator hint"
    );
    for hint in &decorator_hints {
        assert!(
            hint.text.starts_with('@'),
            "decorator hint text must start with '@', got: {:?}",
            hint.text
        );
    }
}

#[test]
fn multiple_class_decorators_each_produce_a_hint() {
    let source = concat!(
        "@Module({ imports: [] })\n",
        "@Injectable()\n",
        "class AppModule {}\n",
    );
    let record = parse_ts(source);
    let decorator_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "decorator")
        .count();
    assert!(
        decorator_count >= 2,
        "two class decorators must produce at least 2 decorator hints, got: {decorator_count}"
    );
}

#[test]
fn file_with_no_decorators_produces_no_decorator_hints() {
    let record = parse_ts("function plain(): void {}\n");
    let decorator_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "decorator")
        .count();
    assert_eq!(
        decorator_count, 0,
        "a file with no decorators must produce zero decorator hints"
    );
}
