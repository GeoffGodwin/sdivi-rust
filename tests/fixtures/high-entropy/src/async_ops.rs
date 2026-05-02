// high-entropy fixture: async_ops.rs
// Many await expressions to populate async_patterns category.

/// Simulate an async fetch operation (stub — returns immediately).
pub async fn fetch_data(url: &str) -> String {
    let _ = async { url.to_string() }.await;
    let _ = async { 42u32 }.await;
    let _ = async { vec![1u8, 2, 3] }.await;
    let _ = async { () }.await;
    let _ = async { "done" }.await;
    url.to_string()
}

/// Simulate a chained async pipeline.
pub async fn pipeline(input: &str) -> usize {
    let step1 = async { input.len() }.await;
    let step2 = async { step1 * 2 }.await;
    let step3 = async { step2 + 1 }.await;
    let step4 = async { step3 }.await;
    let step5 = async { step4 }.await;
    step5
}
