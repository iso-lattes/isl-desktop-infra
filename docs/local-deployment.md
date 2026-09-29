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

The manifest pins an exact 40-hex desktop-daemon commit under `tmp/dev`, builds it with `cargo build --release --locked`, executes the built release binary directly, and binds the daemon only to the loopback address recorded in `appliance.json`.

The local daemon is runnable and loopback-only. Public ingress remains gated because `/v1/invoke` currently uses the same bearer as the local desktop-control boundary. A distinct remote-auth bridge must land before a public compose profile is added.

## Cloudflare boundary

A dedicated/static/public IP is not required. Cloudflare account/API credentials must never be copied to an end-user machine or committed here.

For appliances marked `cloudflare.mode = "gated"`, `appliance.json` records the intended loopback origin and hostname/token metadata, but this repository intentionally does **not** ship a runnable `.ores-compose.public.yaml`. Promotion requires both:

1. the real public origin to be started by the declared local lifecycle; and
2. a remote authentication boundary distinct from the daemon's privileged local-control bearer.

For `cloudflare.mode = "not-required"`, the product uses an outbound authenticated agent path and does not need inbound tunneling.

## Reproducibility gate

The pinned daemon revision `2d81f3ed81a7eac5096cdcfecfbc3e6d83c2c18a` contains a committed Cargo v4 lockfile, so `promotion_gates.daemon_lockfile_committed` is `true`. The compose build uses Cargo's read-only `--locked` mode and must fail rather than silently resolving a different dependency graph.

Upgrades change the immutable daemon source commit only after upstream review/CI. Mutable `latest` refs are forbidden.

## Common desktop implementation layer

This appliance consumes `ORESoftware/ores-common-desktop-infra` for generic host/security/lifecycle behavior instead of maintaining product-local copies.

The machine-readable ORES appliance currently records:

- repository: `ORESoftware/ores-common-desktop-infra`;
- checkout: `tmp/dev/ores-common-desktop-infra`;
- status: `pinned`;
- revision: `20ec084cc550c824c009d5413de84ab519081bd7`.

That merge commit has the same source tree as externally certified PR #40 head `95479e07b6b724784e639576f537303a5e4144b4`, but `promotion_gates.common_layer_ci_verified` remains `false` until the exact consumer pin completes a stepful integration proof here.

The desktop-contract workflow therefore runs two gates during migration:

1. the historical product-specific admission checks; and
2. the shared Rust `validate_appliance_contract` binary from the exact pinned common-layer checkout.

The historical Ruby validator is temporary parity evidence. Remove it only after the Rust gate has demonstrated equivalent coverage for this product; do not weaken admission simply to complete the language migration.

## Reusable isolate boundary

Iso-Lattes is not a fresh-process runtime. The daemon owns a reusable V8 isolate pool, and completion still requires proof that reuse is bounded and keyed to the intended tenant/deployment identity, with deterministic reset/sanitization between invocations, crash recovery, stale-isolate cleanup, and no state leakage across reuse boundaries.

Mutable branches or tags are not acceptable release dependencies.
