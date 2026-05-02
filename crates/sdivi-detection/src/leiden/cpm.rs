//! Constant Potts Model (CPM) quality function helpers.
//!
//! `Q_CPM = Σ_C [e_C − γ · n_C · (n_C − 1) / 2]`
//!
//! where `e_C` is internal edges in community `C`, `n_C` is the number of
//! nodes, and `γ` is the resolution parameter.
//!
//! The gain computation is used in the local move phase for `QualityFunction::Cpm`.

/// CPM gain of moving `node` into community `to`.
///
/// `k_in_to` = edges from `node` to community `to`.
/// `n_to` = current size of community `to` (after `node` was removed from its
/// previous community).
/// `gamma` = resolution parameter.
///
/// Positive return value means the move improves CPM quality.
pub fn cpm_move_gain(k_in_to: f64, n_to: f64, gamma: f64) -> f64 {
    k_in_to - gamma * n_to
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_gain_when_few_connections() {
        // 3 connections into 4-node community, gamma=1 → gain = 3 - 4 = -1
        let gain = cpm_move_gain(3.0, 4.0, 1.0);
        assert!(gain < 0.0);
    }

    #[test]
    fn positive_gain_when_many_connections() {
        // 5 connections into 2-node community, gamma=1 → gain = 5 - 2 = 3
        let gain = cpm_move_gain(5.0, 2.0, 1.0);
        assert!(gain > 0.0);
    }
}
