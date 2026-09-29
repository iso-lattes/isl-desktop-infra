// Temporary vendored migration copy.
// Authority: ORESoftware/ores-common-desktop-infra@26645f8c54f986c82424e5db544487b3fcf1a286
// Source blob: f211bb17ac324d7d24629537e1e95d778b424605
// Remove this copy once the shared conformance crate is distributed through an approved cross-org package path.

use serde_json::Value;
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

const REQUIRED_COMMON_CAPABILITIES: &[&str] = &[
    "consumer-validation",
    "loopback-policy",
    "secret-file-policy",
    "cloudflare-policy",
    "update-policy",
    "process-lifecycle",
    "health-readiness",
    "structured-logging",
    "desktop-auth-shell",
    "mixed-compute-plane",
    "runtime-isolation",
    "stable-ingress",
    "cross-platform-packaging",
    "typed-mutation-auth",
    "middleware-executor-isolation",
    "runtime-adapter-abi",
    "immutable-deployment-bundle",
    "durable-generation-state",
    "desktop-event-stream",
    "conformance-suite",
    "hot-reload-backends",
];

fn main() {
    if let Err(error) = run() {
        eprintln!("desktop appliance conformance failed: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let root = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let appliance_path = root.join("appliance.json");
    let compose_path = root.join(".ores-compose.yaml");

    let appliance = read_json(&appliance_path)?;
    require_string(&appliance, "/schema", "ores.desktop-appliance/v1")?;
    let product = string_at(&appliance, "/product")?;
    require_nonempty_identifier(product, "product")?;
    require_string(&appliance, "/orchestrator/engine", "ores-compose")?;
    require_string(&appliance, "/orchestrator/config", ".ores-compose.yaml")?;
    require_bool(&appliance, "/host/requires_public_ip", false)?;
    require_bool(&appliance, "/cloudflare/credentials_in_repo", false)?;
    require_string(&appliance, "/update/mode", "exact-revision")?;
    require_bool(&appliance, "/update/allow_mutable_latest", false)?;

    let listen = string_at(&appliance, "/host/daemon_listen")?;
    validate_loopback_socket(listen)?;

    let common = object_at(&appliance, "/common_layer")?;
    require_string_value(
        common.get("repository"),
        "ORESoftware/ores-common-desktop-infra",
        "common_layer.repository",
    )?;
    require_string_value(
        common.get("checkout_dir"),
        "tmp/dev/ores-common-desktop-infra",
        "common_layer.checkout_dir",
    )?;
    let common_status = common
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| "common_layer.status must be a string".to_owned())?;
    let pinned_gate = bool_at(&appliance, "/promotion_gates/common_layer_pinned")?;
    match common_status {
        "pinned" => {
            let revision = common
                .get("revision")
                .and_then(Value::as_str)
                .ok_or_else(|| "pinned common layer requires revision".to_owned())?;
            require_full_git_object_id(revision, "common_layer.revision")?;
            if !pinned_gate {
                return Err("pinned common layer requires common_layer_pinned=true".to_owned());
            }
        }
        "awaiting-repository" => {
            if !common.get("revision").is_none_or(Value::is_null) {
                return Err("awaiting common layer may not contain revision".to_owned());
            }
            if pinned_gate {
                return Err("awaiting common layer requires common_layer_pinned=false".to_owned());
            }
            if appliance.get("channel").and_then(Value::as_str) == Some("stable") {
                return Err("stable appliance may not await common layer".to_owned());
            }
        }
        other => return Err(format!("unsupported common_layer.status {other:?}")),
    }

    let capabilities = common
        .get("capabilities")
        .and_then(Value::as_array)
        .ok_or_else(|| "common_layer.capabilities must be an array".to_owned())?;
    let capability_set = capabilities
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .ok_or_else(|| "common_layer capability entries must be strings".to_owned())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    for required in REQUIRED_COMMON_CAPABILITIES {
        if !capability_set.contains(required) {
            return Err(format!("missing common desktop capability {required}"));
        }
    }

    validate_cloudflare(&appliance, &root)?;
    validate_hot_reload(&appliance, &root)?;
    validate_compose(&compose_path, product, listen)?;

    if appliance
        .pointer("/promotion_gates/daemon_lockfile_committed")
        .and_then(Value::as_bool)
        .is_none()
    {
        return Err("promotion_gates.daemon_lockfile_committed must be explicit".to_owned());
    }

    println!("desktop appliance contract OK: {product}");
    return Ok(());
}

fn validate_cloudflare(appliance: &Value, root: &Path) -> Result<(), String> {
    let mode = string_at(appliance, "/cloudflare/mode")?;
    let ready = bool_at(appliance, "/cloudflare/public_ingress_ready")?;
    let origin_auth = string_at(appliance, "/cloudflare/origin_auth")?;
    let public_config = appliance.pointer("/orchestrator/public_config");
    let public_path = root.join(".ores-compose.public.yaml");

    match mode {
        "not-required" => {
            if !public_config.is_none_or(Value::is_null) || ready {
                return Err("not-required Cloudflare mode cannot be public-ready".to_owned());
            }
            if origin_auth != "outbound-agent-only" {
                return Err("not-required mode requires outbound-agent-only auth".to_owned());
            }
            if !appliance
                .pointer("/cloudflare/origin")
                .is_none_or(Value::is_null)
            {
                return Err("not-required Cloudflare mode requires null origin".to_owned());
            }
        }
        "gated" => {
            if !public_config.is_none_or(Value::is_null) || ready {
                return Err("gated Cloudflare mode cannot ship public-ready config".to_owned());
            }
            if origin_auth != "dedicated-remote-auth-required" {
                return Err("gated mode requires dedicated remote-auth bridge".to_owned());
            }
            if string_at(appliance, "/cloudflare/promotion_reason")?
                .trim()
                .is_empty()
            {
                return Err("gated mode requires promotion_reason".to_owned());
            }
        }
        "remotely-managed" => {
            require_string(
                appliance,
                "/orchestrator/public_config",
                ".ores-compose.public.yaml",
            )?;
            if !ready || origin_auth != "dedicated-remote-auth" {
                return Err("remotely-managed mode requires public-ready dedicated auth".to_owned());
            }
            require_string(appliance, "/cloudflare/token_source", "file")?;
        }
        other => return Err(format!("unsupported cloudflare mode {other:?}")),
    }

    if mode != "remotely-managed" && public_path.exists() {
        return Err("gated/not-required appliance may not ship public compose profile".to_owned());
    }
    return Ok(());
}

fn validate_hot_reload(appliance: &Value, root: &Path) -> Result<(), String> {
    let policy_name = string_at(appliance, "/hot_reload_policy")?;
    if policy_name != "hot-reload-policy.json" {
        return Err("hot_reload_policy must be hot-reload-policy.json".to_owned());
    }
    let policy = read_json(&root.join(policy_name))?;
    require_string(&policy, "/schema", "ores.desktop-hot-reload/v1")?;
    require_string(&policy, "/product", string_at(appliance, "/product")?)?;

    let front = string_at(&policy, "/default/front_proxy")?;
    if !matches!(front, "none" | "nginx" | "haproxy" | "caddy") {
        return Err(format!("unsupported default front proxy {front:?}"));
    }
    let drain = u64_at(&policy, "/default/route_drain_timeout_ms")?;
    if drain == 0 || drain > 86_400_000 {
        return Err("route drain timeout must be bounded".to_owned());
    }
    let old_generations = u64_at(&policy, "/default/max_old_generations")?;
    if !(1..=4).contains(&old_generations) {
        return Err("max_old_generations must be between 1 and 4".to_owned());
    }
    require_bool(
        &policy,
        "/invariants/routing_middleware_separate_from_compute",
        true,
    )?;
    require_bool(
        &policy,
        "/invariants/route_change_restarts_standalone_server",
        false,
    )?;
    require_bool(
        &policy,
        "/invariants/route_change_restarts_lambda_workers",
        false,
    )?;
    require_bool(
        &policy,
        "/invariants/arbitrary_native_middleware_in_router_process",
        false,
    )?;
    require_bool(&policy, "/invariants/bounded_old_generation_drain", true)?;
    return Ok(());
}

fn validate_compose(path: &Path, product: &str, listen: &str) -> Result<(), String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    for required in [
        "schema_version: ores.compose.v1",
        "runtime: host",
        "cargo\", \"build\", \"--release",
        "./target/release/",
        "checkout_dir: tmp/dev/",
        "repository: https://github.com/",
        "commit:",
    ] {
        if !text.contains(required) {
            return Err(format!(
                "compose contract missing required marker {required:?}"
            ));
        }
    }
    if text.contains("0.0.0.0") || text.contains("[::]") {
        return Err("compose contract contains non-loopback bind".to_owned());
    }
    if text.contains("services:\n  tunnel:") || text.contains("\n  tunnel:\n") {
        return Err("local compose may not launch a public tunnel".to_owned());
    }
    if text.contains("shell:") || text.contains("sh -c") || text.contains("bash -c") {
        return Err("compose daemon launch may not use a shell".to_owned());
    }
    let health_origin = format!("http://{listen}/");
    if text.contains("healthcheck:") && !text.contains(&health_origin) {
        return Err("daemon healthcheck must target appliance loopback origin".to_owned());
    }
    if !text.contains(&format!("project: {product}-desktop")) {
        return Err("compose project name does not match appliance product".to_owned());
    }

    let commit = text
        .lines()
        .find_map(|line| line.trim().strip_prefix("commit:"))
        .map(str::trim)
        .ok_or_else(|| "compose source commit is missing".to_owned())?;
    require_full_git_object_id(commit, "compose source commit")?;

    return Ok(());
}

fn validate_loopback_socket(value: &str) -> Result<(), String> {
    let socket = value
        .parse::<std::net::SocketAddr>()
        .map_err(|_| "host.daemon_listen must be a socket address".to_owned())?;
    if !socket.ip().is_loopback() {
        return Err("host.daemon_listen must bind to loopback".to_owned());
    }
    return Ok(());
}

fn read_json(path: &Path) -> Result<Value, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    return serde_json::from_slice(&bytes)
        .map_err(|error| format!("{} is invalid JSON: {error}", path.display()));
}

fn object_at<'a>(
    value: &'a Value,
    pointer: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    return value
        .pointer(pointer)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{pointer} must be an object"));
}

fn string_at<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, String> {
    return value
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{pointer} must be a string"));
}

fn bool_at(value: &Value, pointer: &str) -> Result<bool, String> {
    return value
        .pointer(pointer)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("{pointer} must be a boolean"));
}

fn u64_at(value: &Value, pointer: &str) -> Result<u64, String> {
    return value
        .pointer(pointer)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{pointer} must be an unsigned integer"));
}

fn require_string(value: &Value, pointer: &str, expected: &str) -> Result<(), String> {
    return require_string_value(value.pointer(pointer), expected, pointer);
}

fn require_string_value(value: Option<&Value>, expected: &str, label: &str) -> Result<(), String> {
    if value.and_then(Value::as_str) != Some(expected) {
        return Err(format!("{label} must equal {expected:?}"));
    }
    return Ok(());
}

fn require_bool(value: &Value, pointer: &str, expected: bool) -> Result<(), String> {
    if value.pointer(pointer).and_then(Value::as_bool) != Some(expected) {
        return Err(format!("{pointer} must equal {expected}"));
    }
    return Ok(());
}

fn require_full_git_object_id(value: &str, label: &str) -> Result<(), String> {
    if !matches!(value.len(), 40 | 64) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
            "{label} must be a full immutable 40/64-hex object id"
        ));
    }
    return Ok(());
}

fn require_nonempty_identifier(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(format!("{label} must be a safe identifier"));
    }
    return Ok(());
}
