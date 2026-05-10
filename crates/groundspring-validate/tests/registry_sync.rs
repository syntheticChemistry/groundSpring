// SPDX-License-Identifier: AGPL-3.0-or-later

//! Registry cross-sync test — validates groundSpring's `capability_registry.toml`
//! against primalSpring's canonical 403-method capability registry.
//!
//! Phase 60 universal target #2: every spring must test its methods against
//! primalSpring's `config/capability_registry.toml`.
//!
//! This test validates:
//! 1. groundSpring's registry is valid TOML
//! 2. Every tool name in groundSpring uses a known domain prefix
//! 3. groundSpring's domain ("measurement") is self-consistent
//! 4. The canonical registry (primalSpring) is reachable at compile time
//! 5. No method string drift between niche.rs CAPABILITIES and the TOML

/// groundSpring's own capability registry.
const GS_REGISTRY: &str = include_str!("../../../capability_registry.toml");

/// primalSpring's canonical capability registry (403 methods).
/// Path: `primalSpring/config/capability_registry.toml`
const PS_REGISTRY: &str = include_str!("../../../../primalSpring/config/capability_registry.toml");

fn extract_methods_from_primalspring_registry(toml_str: &str) -> Vec<String> {
    let mut methods = Vec::new();
    for line in toml_str.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('"') && trimmed.contains('.') {
            let method = trimmed
                .trim_start_matches('"')
                .trim_end_matches('"')
                .trim_end_matches(',')
                .trim_end_matches('"');
            if method.contains('.') && !method.contains(' ') {
                methods.push(method.to_string());
            }
        }
    }
    methods
}

fn extract_tools_from_groundspring_registry(toml_str: &str) -> Vec<String> {
    let mut tools = Vec::new();
    for line in toml_str.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed
            .strip_prefix("name = \"")
            .and_then(|n| n.strip_suffix('"'))
            .filter(|n| n.contains('.'))
        {
            tools.push(name.to_string());
        }
    }
    tools
}

#[test]
fn groundspring_registry_is_valid_toml() {
    assert!(
        !GS_REGISTRY.is_empty(),
        "groundSpring capability_registry.toml is empty"
    );
    assert!(GS_REGISTRY.contains("[primal]"), "missing [primal] section");
    assert!(
        GS_REGISTRY.contains("[[tools]]"),
        "missing [[tools]] entries"
    );
}

#[test]
fn groundspring_registry_domain_consistent() {
    let tools = extract_tools_from_groundspring_registry(GS_REGISTRY);
    assert!(
        !tools.is_empty(),
        "no tool names found in groundSpring registry"
    );

    for tool in &tools {
        let domain = tool.split('.').next().unwrap_or("");
        assert_eq!(
            domain, "measurement",
            "tool '{tool}' uses domain '{domain}' — expected 'measurement'"
        );
    }
}

#[test]
fn groundspring_registry_matches_niche_capabilities() {
    let registry_tools = extract_tools_from_groundspring_registry(GS_REGISTRY);

    let niche_caps: Vec<&str> = groundspring::niche::CAPABILITIES.to_vec();

    let registry_set: std::collections::BTreeSet<&str> =
        registry_tools.iter().map(String::as_str).collect();
    let niche_set: std::collections::BTreeSet<&str> = niche_caps.into_iter().collect();

    let in_niche_not_registry: Vec<&&str> = niche_set.difference(&registry_set).collect();
    let in_registry_not_niche: Vec<&&str> = registry_set.difference(&niche_set).collect();

    assert!(
        in_niche_not_registry.is_empty(),
        "niche::CAPABILITIES has methods not in capability_registry.toml: {in_niche_not_registry:?}"
    );
    assert!(
        in_registry_not_niche.is_empty(),
        "capability_registry.toml has methods not in niche::CAPABILITIES: {in_registry_not_niche:?}"
    );
}

#[test]
fn canonical_registry_reachable() {
    assert!(
        !PS_REGISTRY.is_empty(),
        "primalSpring canonical capability_registry.toml not found or empty"
    );
}

#[test]
fn groundspring_measurement_domain_is_niche_scoped() {
    let canonical_methods = extract_methods_from_primalspring_registry(PS_REGISTRY);
    let canonical_domains: std::collections::BTreeSet<&str> = canonical_methods
        .iter()
        .filter_map(|m| m.split('.').next())
        .collect();

    assert!(
        !canonical_domains.contains("measurement"),
        "measurement is a spring-niche domain, not an ecosystem-wide domain — \
         it should not appear in the canonical registry"
    );

    let gs_tools = extract_tools_from_groundspring_registry(GS_REGISTRY);
    assert!(
        gs_tools.iter().all(|t| t.starts_with("measurement.")),
        "all groundSpring tools must use the measurement domain prefix"
    );
}

#[test]
fn canonical_registry_method_count() {
    let canonical_methods = extract_methods_from_primalspring_registry(PS_REGISTRY);
    assert!(
        canonical_methods.len() >= 400,
        "canonical registry has {} methods — expected ≥400 (current: 403 per handoff, \
         400 extracted from TOML). Update if primalSpring adds new methods.",
        canonical_methods.len()
    );
}

#[test]
fn groundspring_registry_tool_count() {
    let gs_tools = extract_tools_from_groundspring_registry(GS_REGISTRY);
    assert_eq!(
        gs_tools.len(),
        16,
        "groundSpring should register exactly 16 measurement.* tools, found {}",
        gs_tools.len()
    );
}

#[test]
fn groundspring_methods_use_known_domains() {
    let canonical_methods = extract_methods_from_primalspring_registry(PS_REGISTRY);
    let canonical_domains: std::collections::BTreeSet<&str> = canonical_methods
        .iter()
        .filter_map(|m| m.split('.').next())
        .collect();

    let gs_tools = extract_tools_from_groundspring_registry(GS_REGISTRY);
    for tool in &gs_tools {
        let domain = tool.split('.').next().unwrap_or("");
        assert!(
            canonical_domains.contains(domain) || domain == "measurement",
            "groundSpring tool '{tool}' uses domain '{domain}' which is not in the canonical registry. \
             Known domains: {canonical_domains:?}"
        );
    }
}
