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

This appliance is required to consume `ORESoftware/ores-common-desktop-infra` for generic host/security/lifecycle behavior instead of maintaining product-local copies.

The machine-readable ORES appliance currently records:

- repository: `ORESoftware/ores-common-desktop-infra`;
- checkout: `tmp/dev/ores-common-desktop-infra`;
- status: `awaiting-repository`;
- revision: `null`.

That is a fail-closed migration state. Stable promotion is blocked while `promotion_gates.common_layer_pinned` is false.

Once the common repository is available, migration must be atomic:

1. pin an exact 40-hex common-layer commit;
2. change status to `pinned`;
3. set `common_layer_pinned=true`;
4. invoke/import the shared validators and lifecycle helpers from that exact checkout;
5. delete product-local copies of code now owned by the common layer;
6. keep only product-specific ports, daemon/runtime topology, workers, and native contracts here.

Mutable branches or tags are not acceptable release dependencies.

