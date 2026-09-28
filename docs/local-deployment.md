# ISO Lattes local desktop deployment

This repository uses `ORESoftware/ores-compose` for the declared laptop/desktop lifecycle.

## Local orchestration

```sh
ores-compose check .ores-compose.yaml
ores-compose plan .ores-compose.yaml
ores-compose up .ores-compose.yaml
```

Stop it from another terminal with:

```sh
ores-compose down .ores-compose.yaml
```

The manifest pins an exact 40-hex desktop-daemon commit under `tmp/dev`, builds it before startup, executes the built release binary directly, and binds the daemon only to the loopback address recorded in `appliance.json`.

The local daemon is runnable and loopback-only. Public ingress remains gated because /v1/invoke currently uses the same bearer as the local desktop-control boundary. A distinct remote-auth bridge must land before a public compose profile is added.

## Cloudflare boundary

A dedicated/static/public IP is not required. Cloudflare account/API credentials must never be copied to an end-user machine or committed here.

For appliances marked `cloudflare.mode = "gated"`, `appliance.json` records the intended loopback origin and hostname/token metadata, but this repository intentionally does **not** ship a runnable `.ores-compose.public.yaml`. Promotion requires both:

1. the real public origin to be started by the declared local lifecycle; and
2. a remote authentication boundary distinct from the daemon's privileged local-control bearer.

For `cloudflare.mode = "not-required"`, the product uses an outbound authenticated agent path and does not need inbound tunneling.

## Reproducibility gate

The daemon source is commit-pinned, but the pinned daemon repository does not currently commit a `Cargo.lock`. Therefore `promotion_gates.daemon_lockfile_committed` remains false and this candidate must not be described as fully transitive-dependency reproducible.

Before stable promotion, commit the daemon lockfile, change the build to `cargo build --locked --release`, and make CI enforce it.

## Upgrade model

Upgrades change the immutable daemon source commit only after upstream review/CI. Mutable `latest` refs are forbidden.

## Common desktop implementation layer

Generic host/security/lifecycle behavior is owned by `ORESoftware/ores-common-desktop-infra`.

This appliance now pins the shared implementation at:

```text
repository = ORESoftware/ores-common-desktop-infra
revision   = 7bb4ed89ab4aa4a81c5e26e36b91f58d6313cc7c
checkout   = tmp/dev/ores-common-desktop-infra
status     = pinned
```

The common implementation supplies the shared Rust consumer checker, loopback policy, secret-file policy, Cloudflare promotion checks, exact-revision update policy, process lifecycle, health/readiness policy, and structured-log redaction.

For an authenticated local checkout, validate this repository through the common code with:

```sh
git clone https://github.com/ORESoftware/ores-common-desktop-infra.git \
  tmp/dev/ores-common-desktop-infra
git -C tmp/dev/ores-common-desktop-infra checkout 7bb4ed89ab4aa4a81c5e26e36b91f58d6313cc7c

ORES_COMMON_DESKTOP_EXPECTED_REVISION=7bb4ed89ab4aa4a81c5e26e36b91f58d6313cc7c \
  cargo run --locked \
  --manifest-path tmp/dev/ores-common-desktop-infra/daemon/rust/Cargo.toml \
  --bin ores-common-desktop-consumer-check
```

Cross-organization GitHub Actions use `.github/workflows/common-layer-certification.yml`. Because the common repository is private, that workflow requires the approved read-only fleet credential boundary (`FLEET_GITHUB_READ_TOKEN` or `TEST_FLEET_READ_TOKEN`). Missing credentials fail closed.

`promotion_gates.common_layer_pinned` is true because the exact source revision is now recorded. `promotion_gates.common_layer_ci_verified` remains false until that certification workflow executes successfully. Stable promotion is not allowed while CI evidence is false.

Product-specific ports, workers, daemon/runtime topology, native contracts, and business behavior remain local to this repository; generic policy should migrate into the pinned common layer rather than be reimplemented here.

