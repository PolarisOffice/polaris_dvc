//! Drift check: every value in `crates/polaris-rhwpdvc-core/src/jid_registry.rs`
//! must equal the matching `#define JID_*` in the upstream header. A
//! divergence here means either the vendored snapshot moved without a
//! regen, or someone edited the generated file by hand.
//!
//! The check is a light re-parse of JsonModel.h — simpler than pulling
//! in the generator crate — and compares against Rust's source file by
//! grepping for `pub const JID_*: ErrorCode = ErrorCode::new(N);` lines.
//! Setting `POLARIS_ALLOW_JID_DRIFT=1` skips the failure so a dev who
//! just touched the header can regen without the test blocking them
//! mid-iteration.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn generated_jid_values_match_upstream_header() {
    if std::env::var_os("POLARIS_ALLOW_JID_DRIFT").is_some() {
        eprintln!("polaris: POLARIS_ALLOW_JID_DRIFT set — skipping drift check");
        return;
    }

    let root = repo_root();
    let header = fs::read_to_string(root.join("third_party/dvc-upstream/Source/JsonModel.h"))
        .expect("upstream header");
    let generated = fs::read_to_string(root.join("crates/polaris-rhwpdvc-core/src/jid_registry.rs"))
        .expect("generated registry");

    let upstream = parse_upstream(&header);
    let emitted = parse_generated(&generated);

    // 1. Every upstream JID must appear in the generated file.
    let mut missing: Vec<&String> = upstream
        .keys()
        .filter(|k| !emitted.contains_key(*k))
        .collect();
    missing.sort();
    assert!(
        missing.is_empty(),
        "generated jid_registry missing {} upstream entries; regenerate with \
         `cargo run --manifest-path tools/gen-jids/Cargo.toml`. Missing: {:?}",
        missing.len(),
        missing
    );

    // 2. Every generated value must equal the upstream value.
    for (name, emitted_val) in &emitted {
        let upstream_val = upstream.get(name).unwrap_or_else(|| {
            panic!(
                "generated registry has stale name {name} not in upstream header; \
                 regenerate with POLARIS_REGEN_JIDS-style workflow"
            )
        });
        assert_eq!(
            *emitted_val, *upstream_val,
            "drift on {name}: registry={emitted_val}, upstream={upstream_val}"
        );
    }
}

/// Parse `#define JID_FOO <literal>` and `#define JID_FOO JID_BAR+N`.
/// Uses the same resolve-until-fixed-point strategy as the generator
/// (but simpler: we don't need grouping / ordering).
fn parse_upstream(src: &str) -> HashMap<String, u32> {
    let mut resolved: HashMap<String, u32> = HashMap::new();
    let mut pending: Vec<(String, String)> = Vec::new();

    for line in src.lines() {
        let trimmed = match line.find("//") {
            Some(p) => &line[..p],
            None => line,
        }
        .trim_start();
        let Some(rest) = trimmed.strip_prefix("#define") else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(tail) = rest.strip_prefix("JID_") else {
            continue;
        };
        let split_at = tail.find(|c: char| c.is_whitespace()).unwrap_or(tail.len());
        let name = format!("JID_{}", &tail[..split_at]);
        let rhs = tail[split_at..]
            .trim()
            .trim_end_matches('\\')
            .trim()
            .to_string();
        if let Ok(v) = rhs.parse::<u32>() {
            resolved.insert(name, v);
        } else {
            pending.push((name, rhs));
        }
    }

    loop {
        let before = resolved.len();
        pending.retain(|(name, rhs)| {
            let Some(v) = eval(rhs, &resolved) else {
                return true;
            };
            resolved.insert(name.clone(), v);
            false
        });
        if resolved.len() == before {
            break;
        }
    }
    resolved
}

fn eval(rhs: &str, table: &HashMap<String, u32>) -> Option<u32> {
    let t: String = rhs.chars().filter(|c| !c.is_whitespace()).collect();
    if let Some(idx) = t.find('+') {
        let (a, b) = t.split_at(idx);
        let base = table.get(a).or_else(|| table.get(&format!("JID_{a}")))?;
        let off: i64 = b[1..].parse().ok()?;
        return (*base as i64 + off).try_into().ok();
    }
    if let Some(idx) = t.find('-') {
        if idx == 0 {
            return None;
        }
        let (a, b) = t.split_at(idx);
        let base = table.get(a).or_else(|| table.get(&format!("JID_{a}")))?;
        let off: i64 = b[1..].parse().ok()?;
        return (*base as i64 - off).try_into().ok();
    }
    table
        .get(t.as_str())
        .or_else(|| table.get(&format!("JID_{t}")))
        .copied()
}

/// Extract every `pub const JID_*: ErrorCode = ErrorCode::new(N);` from
/// the generated file. Regex-free so we don't add a dependency.
fn parse_generated(src: &str) -> HashMap<String, u32> {
    let mut out = HashMap::new();
    for line in src.lines() {
        let t = line.trim_start();
        let Some(rest) = t.strip_prefix("pub const JID_") else {
            continue;
        };
        let Some((name_tail, after_colon)) = rest.split_once(':') else {
            continue;
        };
        let name = format!("JID_{}", name_tail.trim());
        let Some(open) = after_colon.find('(') else {
            continue;
        };
        let Some(close) = after_colon[open..].find(')') else {
            continue;
        };
        let numeric = after_colon[open + 1..open + close].trim();
        if let Ok(v) = numeric.parse::<u32>() {
            out.insert(name, v);
        }
    }
    out
}
