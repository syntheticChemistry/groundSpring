// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario registry — metadata, tier filtering, and track taxonomy.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

/// Validation tier for scenario filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    /// Tier 1: Pure Rust structural validation — no IPC needed.
    Rust,
    /// Tier 2: Live NUCLEUS validation — requires deployed primals.
    Live,
    /// Both tiers: has structural and live phases.
    Both,
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tier::Rust => write!(f, "rust"),
            Tier::Live => write!(f, "live"),
            Tier::Both => write!(f, "both"),
        }
    }
}

/// Track taxonomy — groups related scenarios by measurement domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Track {
    /// Bias-variance decomposition and noise characterization.
    NoiseDecomposition,
    /// Ecological diversity and rarefaction analysis.
    Ecology,
    /// Anderson localization and condensed matter physics.
    CondensedMatter,
    /// FAO-56 evapotranspiration and agricultural science.
    AgriculturalScience,
    /// Statistical fitting: freeze-out, chi-squared, grid search.
    StatisticalFitting,
    /// Dynamical systems: bistable switching, ESN, quasispecies.
    DynamicalSystems,
    /// Seismic and geophysical analysis.
    Geophysics,
    /// Population genetics: drift, selection, Wright-Fisher.
    PopulationGenetics,
    /// Resampling methods: jackknife, bootstrap, RAWR.
    Resampling,
    /// Full NUCLEUS composition parity.
    CompositionParity,
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Track::NoiseDecomposition => "noise-decomposition",
            Track::Ecology => "ecology",
            Track::CondensedMatter => "condensed-matter",
            Track::AgriculturalScience => "agricultural-science",
            Track::StatisticalFitting => "statistical-fitting",
            Track::DynamicalSystems => "dynamical-systems",
            Track::Geophysics => "geophysics",
            Track::PopulationGenetics => "population-genetics",
            Track::Resampling => "resampling",
            Track::CompositionParity => "composition-parity",
        };
        write!(f, "{s}")
    }
}

/// Scenario metadata — provenance, classification, and description.
#[derive(Debug, Clone)]
pub struct ScenarioMeta {
    /// Unique scenario identifier (e.g. `"decompose-bias-variance"`).
    pub id: &'static str,
    /// Which track this scenario belongs to.
    pub track: Track,
    /// Which validation tier this scenario exercises.
    pub tier: Tier,
    /// Original experiment crate/binary name for provenance.
    pub provenance_crate: &'static str,
    /// Date of last significant update.
    pub provenance_date: &'static str,
    /// One-line description.
    pub description: &'static str,
}

/// A callable scenario: metadata + run function.
pub struct Scenario {
    /// Scenario metadata.
    pub meta: ScenarioMeta,
    /// The validation function. Takes the result accumulator and a composition
    /// context (may be unused for Tier 1 scenarios).
    pub run: fn(&mut ValidationResult, &mut CompositionContext),
}

/// Registry of all validation scenarios.
pub struct ScenarioRegistry {
    scenarios: Vec<Scenario>,
}

impl ScenarioRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            scenarios: Vec::new(),
        }
    }

    /// Register a scenario.
    pub fn register(&mut self, scenario: Scenario) {
        self.scenarios.push(scenario);
    }

    /// All registered scenarios.
    #[must_use]
    pub fn all(&self) -> &[Scenario] {
        &self.scenarios
    }

    /// Filter scenarios by tier.
    pub fn filter_by_tier(&self, tier: Tier) -> impl Iterator<Item = &Scenario> {
        self.scenarios
            .iter()
            .filter(move |s| s.meta.tier == tier || s.meta.tier == Tier::Both || tier == Tier::Both)
    }

    /// Filter scenarios by track.
    pub fn filter_by_track(&self, track: Track) -> impl Iterator<Item = &Scenario> {
        self.scenarios.iter().filter(move |s| s.meta.track == track)
    }

    /// Find a scenario by ID.
    #[must_use]
    pub fn find(&self, id: &str) -> Option<&Scenario> {
        self.scenarios.iter().find(|s| s.meta.id == id)
    }

    /// Number of registered scenarios.
    #[must_use]
    pub fn len(&self) -> usize {
        self.scenarios.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.scenarios.is_empty()
    }
}

impl Default for ScenarioRegistry {
    fn default() -> Self {
        Self::new()
    }
}
