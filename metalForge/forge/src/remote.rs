// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Remote substrate discovery via biomeOS capability routing.
//!
//! Extends metalForge's local-only `Inventory` with substrates advertised
//! by remote NUCLEUS nodes. Remote nodes respond to `metalforge.discover`
//! capability calls with their local inventory serialized as JSON.
//!
//! # Protocol
//!
//! 1. Caller sends `capability.call("metalforge.discover", {})` via Neural API
//! 2. biomeOS routes to all nodes that registered this capability
//! 3. Each node returns its `Inventory` as JSON (substrate list)
//! 4. Caller merges remote substrates into local inventory with `RemoteOrigin`
//!
//! # Latency Awareness
//!
//! Remote substrates carry a `RemoteOrigin` tag with the node ID and
//! estimated latency. Dispatch can factor in network overhead when choosing
//! between a local CPU and a remote GPU.

use crate::substrate::{Capability, Identity, Properties, Substrate, SubstrateKind};

/// Origin information for a remote substrate.
#[derive(Debug, Clone)]
pub struct RemoteOrigin {
    /// NUCLEUS node identifier (e.g. "strandgate", "biomegate").
    pub node_id: String,
    /// Whether this node is on the local LAN (covalent bond).
    pub is_lan: bool,
    /// Estimated round-trip latency in milliseconds (0 = unknown).
    pub estimated_latency_ms: u32,
}

/// A substrate discovered on a remote NUCLEUS node.
#[derive(Debug, Clone)]
pub struct RemoteSubstrate {
    /// The substrate itself (same struct as local).
    pub substrate: Substrate,
    /// Where it came from.
    pub origin: RemoteOrigin,
}

/// Parse a remote inventory response into substrates.
///
/// Expects a JSON array of substrate descriptors from a remote node's
/// `metalforge.discover` response. Each entry has:
/// - `kind`: "gpu" | "npu" | "cpu"
/// - `name`: human-readable device name
/// - `capabilities`: array of capability labels
/// - `memory_bytes`: optional u64
/// - `gpu_arch`: optional architecture string
///
/// Unknown capabilities or kinds are silently skipped.
#[must_use]
pub fn parse_remote_inventory(node_id: &str, json_response: &str) -> Vec<RemoteSubstrate> {
    let mut results = Vec::new();
    let origin = RemoteOrigin {
        node_id: node_id.to_string(),
        is_lan: true,
        estimated_latency_ms: 0,
    };

    // Minimal JSON array parsing without pulling in serde.
    // Remote inventory responses are structured as one-substrate-per-line.
    for entry in split_json_array(json_response) {
        if let Some(sub) = parse_substrate_entry(entry.trim(), &origin) {
            results.push(sub);
        }
    }

    results
}

/// Merge remote substrates into a local inventory.
///
/// Remote substrates get their `Identity.name` prefixed with the node ID
/// to distinguish them from local devices in dispatch decisions and logs.
#[must_use]
pub fn merge_remote(local: Vec<Substrate>, remote: &[RemoteSubstrate]) -> Vec<Substrate> {
    let mut merged = local;
    for rs in remote {
        let mut sub = rs.substrate.clone();
        sub.identity.name = format!("{}@{}", sub.identity.name, rs.origin.node_id);
        merged.push(sub);
    }
    merged
}

/// Build the JSON request body for `metalforge.discover` capability call.
#[must_use]
pub fn discover_request_json() -> String {
    r#"{"include_properties":true}"#.to_string()
}

// ─── Internal Parsing ────────────────────────────────────────────────────────

fn split_json_array(s: &str) -> Vec<&str> {
    let trimmed = s.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(trimmed);

    if inner.trim().is_empty() {
        return Vec::new();
    }

    let mut entries = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (i, ch) in inner.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    entries.push(&inner[start..=i]);
                    start = i + 1;
                }
            }
            ',' if depth == 0 => {
                start = i + 1;
            }
            _ => {}
        }
    }

    entries
}

fn parse_substrate_entry(entry: &str, origin: &RemoteOrigin) -> Option<RemoteSubstrate> {
    let kind = extract_string_field(entry, "kind")?;
    let name = extract_string_field(entry, "name")?;

    let substrate_kind = match kind.as_str() {
        "gpu" => SubstrateKind::Gpu,
        "npu" => SubstrateKind::Npu,
        "cpu" => SubstrateKind::Cpu,
        _ => return None,
    };

    let caps = extract_capabilities(entry);
    let memory = extract_u64_field(entry, "memory_bytes");
    let gpu_arch = extract_string_field(entry, "gpu_arch").and_then(|s| parse_gpu_arch(&s));

    let substrate = Substrate {
        kind: substrate_kind,
        identity: Identity::named(name),
        properties: Properties {
            memory_bytes: memory,
            gpu_arch,
            has_f64: caps.contains(&Capability::F64Compute),
            ..Properties::default()
        },
        capabilities: caps,
    };

    Some(RemoteSubstrate {
        substrate,
        origin: origin.clone(),
    })
}

/// Parse a GPU architecture from its canonical name or adapter-style name.
fn parse_gpu_arch(s: &str) -> Option<crate::substrate::GpuArch> {
    match s.to_lowercase().as_str() {
        "volta" => Some(crate::substrate::GpuArch::Volta),
        "turing" => Some(crate::substrate::GpuArch::Turing),
        "ampere" => Some(crate::substrate::GpuArch::Ampere),
        "ada" => Some(crate::substrate::GpuArch::Ada),
        _ => {
            let arch = crate::substrate::GpuArch::from_name(s);
            if matches!(arch, crate::substrate::GpuArch::Other) && !s.is_empty() {
                None
            } else {
                Some(arch)
            }
        }
    }
}

fn extract_string_field(json: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{field}\"");
    let start = json.find(&pattern)?;
    let after_key = &json[start + pattern.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim_start();
    let value_start = after_colon.strip_prefix('"')?;
    let end = value_start.find('"')?;
    Some(value_start[..end].to_string())
}

fn extract_u64_field(json: &str, field: &str) -> Option<u64> {
    let pattern = format!("\"{field}\"");
    let start = json.find(&pattern)?;
    let after_key = &json[start + pattern.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim_start();

    let end = after_colon.find(|c: char| !c.is_ascii_digit())?;
    after_colon[..end].parse().ok()
}

fn extract_capabilities(json: &str) -> Vec<Capability> {
    let mut caps = Vec::new();
    let pattern = "\"capabilities\"";
    let Some(start) = json.find(pattern) else {
        return caps;
    };
    let after = &json[start + pattern.len()..];
    let Some(bracket) = after.find('[') else {
        return caps;
    };
    let inner = &after[bracket + 1..];
    let Some(end_bracket) = inner.find(']') else {
        return caps;
    };
    let cap_list = &inner[..end_bracket];

    for label in cap_list.split(',') {
        let label = label.trim().trim_matches('"');
        match label {
            "f64" => caps.push(Capability::F64Compute),
            "f32" => caps.push(Capability::F32Compute),
            "native-f64" => caps.push(Capability::NativeF64),
            "shader" => caps.push(Capability::ShaderDispatch),
            "reduce" => caps.push(Capability::ScalarReduce),
            "quant" => caps.push(Capability::QuantizedInference { bits: 8 }),
            "batch" => caps.push(Capability::BatchInference { max_batch: 8 }),
            "weight-mut" => caps.push(Capability::WeightMutation),
            "simd" => caps.push(Capability::SimdVector),
            "timestamps" => caps.push(Capability::TimestampQuery),
            _ => {}
        }
    }

    caps
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RESPONSE: &str = r#"[
        {"kind":"gpu","name":"NVIDIA TITAN V","capabilities":["f64","f32","native-f64","shader","reduce"],"memory_bytes":12884901888,"gpu_arch":"Volta"},
        {"kind":"gpu","name":"NVIDIA GeForce RTX 4070","capabilities":["f64","f32","shader","reduce","timestamps"],"memory_bytes":12884901888,"gpu_arch":"Ada"},
        {"kind":"npu","name":"AKD1000","capabilities":["quant","batch"],"memory_bytes":0},
        {"kind":"cpu","name":"AMD EPYC 7763","capabilities":["f64","f32","simd"],"memory_bytes":274877906944}
    ]"#;

    #[test]
    fn parse_remote_inventory_extracts_all_substrates() {
        let subs = parse_remote_inventory("biomegate", SAMPLE_RESPONSE);
        assert_eq!(subs.len(), 4);
        assert_eq!(subs[0].substrate.kind, SubstrateKind::Gpu);
        assert!(subs[0].substrate.identity.name.contains("TITAN V"));
        assert_eq!(subs[0].origin.node_id, "biomegate");
    }

    #[test]
    fn parse_remote_identifies_volta() {
        let subs = parse_remote_inventory("biomegate", SAMPLE_RESPONSE);
        let titan = &subs[0].substrate;
        assert!(titan.has(&Capability::NativeF64));
        assert_eq!(
            titan.properties.gpu_arch,
            Some(crate::substrate::GpuArch::Volta)
        );
    }

    #[test]
    fn parse_remote_npu() {
        let subs = parse_remote_inventory("strandgate", SAMPLE_RESPONSE);
        let npu = &subs[2].substrate;
        assert_eq!(npu.kind, SubstrateKind::Npu);
        assert!(npu.has(&Capability::QuantizedInference { bits: 8 }));
    }

    #[test]
    fn parse_remote_cpu_with_memory() {
        let subs = parse_remote_inventory("strandgate", SAMPLE_RESPONSE);
        let cpu = &subs[3].substrate;
        assert_eq!(cpu.kind, SubstrateKind::Cpu);
        assert_eq!(cpu.properties.memory_bytes, Some(274_877_906_944));
    }

    #[test]
    fn merge_remote_prefixes_names() {
        let local = vec![Substrate {
            kind: SubstrateKind::Cpu,
            identity: Identity::named("local CPU"),
            properties: Properties::default(),
            capabilities: vec![Capability::F64Compute],
        }];
        let remote_subs = parse_remote_inventory("biomegate", SAMPLE_RESPONSE);
        let merged = merge_remote(local, &remote_subs);

        assert_eq!(merged.len(), 5);
        assert_eq!(merged[0].identity.name, "local CPU");
        assert!(merged[1].identity.name.contains("@biomegate"));
    }

    #[test]
    fn empty_response_returns_empty() {
        let subs = parse_remote_inventory("node", "[]");
        assert!(subs.is_empty());
    }

    #[test]
    fn empty_string_returns_empty() {
        let subs = parse_remote_inventory("node", "");
        assert!(subs.is_empty());
    }

    #[test]
    fn discover_request_json_valid() {
        let req = discover_request_json();
        assert!(req.contains("include_properties"));
    }

    #[test]
    fn unknown_kind_skipped() {
        let json = r#"[{"kind":"fpga","name":"Xilinx","capabilities":["f32"]}]"#;
        let subs = parse_remote_inventory("node", json);
        assert!(subs.is_empty());
    }

    #[test]
    fn unknown_capability_skipped() {
        let json =
            r#"[{"kind":"gpu","name":"Test","capabilities":["f64","unknown_cap","shader"]}]"#;
        let subs = parse_remote_inventory("node", json);
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].substrate.capabilities.len(), 2);
    }

    #[test]
    fn split_json_array_basic() {
        let parts = split_json_array(r#"[{"a":1},{"b":2}]"#);
        assert_eq!(parts.len(), 2);
    }
}
