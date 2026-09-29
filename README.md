# ISO Lattes Desktop Infra

Single-host desired-state and local deployment authority for developer- or end-user-owned desktops/laptops.

## ORES Compose local deployment

The audited local lifecycle is declared in `.ores-compose.yaml`:

```sh
ores-compose check .ores-compose.yaml
ores-compose plan .ores-compose.yaml
ores-compose up .ores-compose.yaml
```

Public ingress is intentionally gated until `/v1/invoke` has a remote-auth credential separate from the local desktop-control bearer.

The daemon source is exact-commit pinned, loopback-only, built with the committed dependency lockfile via `cargo build --release --locked`, and executed from the built release binary. Stable promotion remains blocked on the still-open public-ingress/authentication and common-layer certification gates.

See [docs/local-deployment.md](docs/local-deployment.md) and [appliance.json](appliance.json) for the audited boundary and promotion gates.

## Shared desktop infra dependency

Generic desktop lifecycle/security behavior lives in `ORESoftware/ores-common-desktop-infra`. This repo declares that dependency in its ORES appliance metadata and pins an immutable revision rather than following a mutable branch.

The common platform is pinned at merge commit `20ec084cc550c824c009d5413de84ab519081bd7`. Its source tree is byte-identical to the externally certified PR #40 head `95479e07b6b724784e639576f537303a5e4144b4`; the product gate `common_layer_ci_verified` nevertheless remains `false` until this exact consumer pin receives its own stepful integration proof.

The desktop-contract workflow dual-runs the product's historical admission checks and the shared Rust `validate_appliance_contract` binary from the pinned common revision. The historical Ruby gate is transitional parity evidence, not the long-term authority; it should be removed only after the Rust validator has demonstrated equivalent product coverage.

## Runtime semantics

Iso-Lattes preserves reusable V8 isolate semantics. Reuse must be bounded, tenant/deployment scoped, observable, and paired with deterministic reset/sanitization before another invocation. This infra migration does not weaken or replace those runtime-specific lifecycle requirements.

## Hot-reload routing and middleware

This product consumes the shared ORES generation model with **native atomic routes + supervised external middleware process generations** as its default. Routing/middleware is a separate lifecycle and memory/failure boundary from standalone servers and isolate workers, so route or middleware updates do not restart unrelated compute.

`hot-reload-policy.json` declares the product policy. The edge may optionally use nginx, HAProxy, or Caddy. nginx uses validated worker-generation reloads; HAProxy prefers Runtime API changes and falls back to master-worker reload for structural changes; Caddy uses its transactional Admin API. Proxy-managed application routes are opt-in and limited to declarative routing/middleware. Arbitrary middleware code stays in BEAM, Wasm, or a separately supervised process generation.

Long-lived WebSockets/streams are bounded by a hard generation drain timeout so repeated reloads cannot accumulate old generations indefinitely.
