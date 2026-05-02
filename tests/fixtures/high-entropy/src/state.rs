// high-entropy fixture: state.rs
// Many closure expressions to populate state_management category.

/// Apply a series of transformations using closures.
pub fn transform_pipeline(values: Vec<i32>) -> Vec<i32> {
    let add_one = |x: i32| x + 1;
    let double = |x: i32| x * 2;
    let negate = |x: i32| -x;
    let clamp = |x: i32| x.clamp(-100, 100);
    let absolute = |x: i32| x.abs();

    values
        .into_iter()
        .map(add_one)
        .map(double)
        .map(negate)
        .map(clamp)
        .map(absolute)
        .collect()
}

/// Accumulate values with a closure-based fold.
pub fn accumulate(values: &[i32]) -> i32 {
    let summer = |acc: i32, &x: &i32| acc + x;
    values.iter().fold(0, summer)
}

/// Filter values with multiple closure predicates.
pub fn filter_pipeline(values: Vec<i32>) -> Vec<i32> {
    let is_positive = |&x: &i32| x > 0;
    let is_even = |&x: &i32| x % 2 == 0;
    let under_hundred = |&x: &i32| x < 100;

    values
        .into_iter()
        .filter(is_positive)
        .filter(is_even)
        .filter(under_hundred)
        .collect()
}
