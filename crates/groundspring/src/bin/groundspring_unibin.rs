// SPDX-License-Identifier: AGPL-3.0-or-later

//! groundSpring UniBin — eukaryotic single binary.
//!
//! Absorbs certification (L0-L4), validation scenarios (10 tracks),
//! and status reporting into a single deployable binary.
//!
//! # Subcommands
//!
//! - `certify` — Run certification layers (L0 bare, L2-L4 NUCLEUS)
//! - `validate` — Run validation scenarios by tier/track
//! - `status` — Print composition health and discovery summary
//! - `version` — Print version information

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};

use groundspring::certification;
use groundspring::validation;
use groundspring::validation::scenarios::registry::Tier;

#[derive(Parser)]
#[command(
    name = "groundspring",
    about = "groundSpring UniBin — measurement science certification and validation",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run certification engine (L0-L4).
    Certify {
        /// Maximum layer to certify (0-4). Default: 4.
        #[arg(long)]
        layer: Option<u8>,

        /// Run bare checks only (L0, no primals needed).
        #[arg(long)]
        bare: bool,
    },

    /// Run validation scenarios.
    Validate {
        /// Filter by track (e.g. "ecology", "condensed-matter").
        #[arg(long)]
        track: Option<String>,

        /// Filter by scenario ID.
        #[arg(long)]
        scenario: Option<String>,

        /// Filter by tier: "rust", "live", "both", or "all".
        #[arg(long, default_value = "all")]
        tier: String,

        /// List all scenarios without running them.
        #[arg(long)]
        list: bool,
    },

    /// Print composition health and discovery summary.
    Status,

    /// Print version information.
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Certify { layer, bare } => cmd_certify(layer, bare),
        Commands::Validate {
            track,
            scenario,
            tier,
            list,
        } => cmd_validate(track, scenario, tier, list),
        Commands::Status => cmd_status(),
        Commands::Version => cmd_version(),
    }
}

fn cmd_certify(layer: Option<u8>, bare: bool) {
    let max_layer = if bare {
        0
    } else {
        layer.unwrap_or(certification::MAX_LAYER)
    };

    let result = certification::certify(max_layer);

    if result.exit_code() == 2 {
        eprintln!("[unibin] Bare-only: no primals discovered.");
    }

    std::process::exit(result.exit_code());
}

fn cmd_validate(
    track_filter: Option<String>,
    scenario_filter: Option<String>,
    tier_str: String,
    list: bool,
) {
    let registry = validation::build_registry();

    if list {
        println!(
            "{:<30} {:<25} {:<6} {}",
            "ID", "TRACK", "TIER", "DESCRIPTION"
        );
        println!("{}", "-".repeat(90));
        for s in registry.all() {
            println!(
                "{:<30} {:<25} {:<6} {}",
                s.meta.id, s.meta.track, s.meta.tier, s.meta.description
            );
        }
        return;
    }

    let tier = match tier_str.as_str() {
        "rust" => Some(Tier::Rust),
        "live" => Some(Tier::Live),
        "both" | "all" => None,
        other => {
            eprintln!("Unknown tier: {other}. Use rust, live, both, or all.");
            std::process::exit(1);
        }
    };

    let mut v =
        primalspring::validation::ValidationResult::new("groundSpring validation scenarios");
    let mut ctx =
        primalspring::composition::CompositionContext::from_live_discovery_with_fallback();

    let mut ran = 0;
    for s in registry.all() {
        if let Some(ref t) = track_filter {
            if !format!("{}", s.meta.track).contains(t.as_str()) {
                continue;
            }
        }
        if let Some(ref id) = scenario_filter {
            if s.meta.id != id.as_str() {
                continue;
            }
        }
        if let Some(t) = tier {
            if s.meta.tier != t && s.meta.tier != Tier::Both {
                continue;
            }
        }

        v.section(&format!("[{}] {}", s.meta.id, s.meta.description));
        (s.run)(&mut v, &mut ctx);
        ran += 1;
    }

    if ran == 0 {
        eprintln!("No scenarios matched filters.");
        std::process::exit(1);
    }

    v.finish();
    std::process::exit(v.exit_code());
}

fn cmd_status() {
    println!("groundSpring UniBin v{}", env!("CARGO_PKG_VERSION"));
    println!("certification layers: L0–L{}", certification::MAX_LAYER);

    let registry = validation::build_registry();
    println!("validation scenarios: {}", registry.len());

    for s in registry.all() {
        println!("  [{:<6}] {:<30} {}", s.meta.tier, s.meta.id, s.meta.track);
    }
}

fn cmd_version() {
    println!(
        "groundspring_unibin {} (primalspring v0.9.25 pin)",
        env!("CARGO_PKG_VERSION")
    );
    println!("certification: L0–L{}", certification::MAX_LAYER);
    println!("scenarios: {}", validation::build_registry().len());
    println!("edition: 2024");
}
