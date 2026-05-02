//! [`LeidenPartition`] — the output type of the Leiden community detection stage.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Quality function used during community detection.
///
/// # Examples
///
/// ```rust
/// use sdivi_detection::partition::QualityFunction;
///
/// let q = QualityFunction::Modularity;
/// assert!(matches!(q, QualityFunction::Modularity));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum QualityFunction {
    /// Newman–Girvan modularity (default).
    #[default]
    Modularity,
    /// Constant Potts Model with resolution parameter `gamma`.
    Cpm {
        /// Resolution parameter; higher values produce smaller communities.
        gamma: f64,
    },
}

/// Configuration for a Leiden run.
///
/// # Examples
///
/// ```rust
/// use sdivi_detection::partition::LeidenConfig;
///
/// let cfg = LeidenConfig::default();
/// assert_eq!(cfg.seed, 42);
/// assert_eq!(cfg.max_iterations, 100);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeidenConfig {
    /// Random seed for deterministic partition results.
    pub seed: u64,
    /// Maximum iterations of the outer Leiden loop.
    pub max_iterations: usize,
    /// Quality function to optimise.
    pub quality: QualityFunction,
    /// Resolution parameter forwarded to CPM; ignored for Modularity.
    pub gamma: f64,
}

impl Default for LeidenConfig {
    fn default() -> Self {
        LeidenConfig {
            seed: 42,
            max_iterations: 100,
            quality: QualityFunction::Modularity,
            gamma: 1.0,
        }
    }
}

impl LeidenConfig {
    /// Creates a `LeidenConfig` from a [`sdivi_config::Config`].
    pub fn from_sdivi_config(cfg: &sdivi_config::Config) -> Self {
        LeidenConfig {
            seed: cfg.core.random_seed,
            max_iterations: 100,
            quality: QualityFunction::Modularity,
            gamma: cfg.boundaries.leiden_gamma,
        }
    }
}

/// The result of a Leiden community detection run.
///
/// Community IDs are stable integers starting at zero, assigned in ascending
/// order of the lowest node index within each community.  This guarantees
/// deterministic JSON output given the same input graph and seed.
///
/// # Examples
///
/// ```rust
/// use sdivi_detection::partition::LeidenPartition;
/// use std::collections::BTreeMap;
///
/// let p = LeidenPartition {
///     assignments: BTreeMap::from([(0, 0), (1, 0), (2, 1)]),
///     stability: BTreeMap::from([(0, 0.8), (1, 1.0)]),
///     modularity: 0.42,
///     seed: 42,
/// };
/// assert_eq!(p.assignments[&0], 0);
/// assert_eq!(p.community_count(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LeidenPartition {
    /// Node index → community ID.
    pub assignments: BTreeMap<usize, usize>,
    /// Community ID → stability score (internal edge density, `[0, 1]`).
    pub stability: BTreeMap<usize, f64>,
    /// Overall modularity of the final partition.
    pub modularity: f64,
    /// Seed used to produce this partition.
    pub seed: u64,
}

impl LeidenPartition {
    /// Number of communities in the partition.
    pub fn community_count(&self) -> usize {
        self.stability.len()
    }

    /// Returns the community ID for node index `node`.
    pub fn community_of(&self, node: usize) -> Option<usize> {
        self.assignments.get(&node).copied()
    }

    /// Returns the file path associated with the largest community (most nodes).
    pub fn largest_community_size(&self) -> usize {
        let mut counts: BTreeMap<usize, usize> = BTreeMap::new();
        for &comm in self.assignments.values() {
            *counts.entry(comm).or_insert(0) += 1;
        }
        counts.values().copied().max().unwrap_or(0)
    }

    /// Groups node indices by community, sorted for determinism.
    pub fn communities(&self) -> BTreeMap<usize, Vec<usize>> {
        let mut map: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (&node, &comm) in &self.assignments {
            map.entry(comm).or_default().push(node);
        }
        map
    }

    /// Serialises the partition to compact JSON.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Deserialises a partition from JSON.
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}
