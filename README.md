# ISO Lattes Desktop Infra

Single-host desired-state and local deployment authority for developer- or end-user-owned desktops/laptops.

## ORES Compose local deployment

The audited local lifecycle is declared in `.ores-compose.yaml`:

```sh
ores-compose check .ores-compose.yaml
ores-compose plan .ores-compose.yaml
ores-compose up .ores-compose.yaml
```

Public ingress is intentionally gated until /v1/invoke has a remote-auth credential separate from the local desktop-control bearer.

The daemon source is exact-commit pinned, loopback-only, and executed from the built release binary. Stable promotion remains blocked until the daemon repository commits a Cargo lockfile and the build switches to `--locked`.

See [docs/local-deployment.md](docs/local-deployment.md) and [appliance.json](appliance.json) for the audited boundary and promotion gates.

## Shared desktop infra dependency

Generic desktop lifecycle/security behavior is moving to `ORESoftware/ores-common-desktop-infra`. This repo declares that dependency in its ORES appliance metadata and blocks stable promotion until an exact common-layer commit is pinned.

Current state is intentionally `awaiting-repository` with a null revision because GitHub does not yet expose that repository through the connected installation. Product-local behavior remains candidate-only until the common layer can be consumed by exact SHA.


## Hot-reload routing and middleware

This product consumes the shared ORES generation model with **native atomic routes + supervised external middleware process generations** as its default. Routing/middleware is a separate lifecycle and memory/failure boundary from standalone servers and lambda/actor workers, so route or middleware updates do not restart unrelated compute.

`hot-reload-policy.json` declares the product policy. The edge may optionally use nginx, HAProxy, or Caddy. nginx uses validated worker-generation reloads; HAProxy prefers Runtime API changes and falls back to master-worker reload for structural changes; Caddy uses its transactional Admin API. Proxy-managed application routes are opt-in and limited to declarative routing/middleware. Arbitrary middleware code stays in BEAM, Wasm, or a separately supervised process generation.

Long-lived WebSockets/streams are bounded by a hard generation drain timeout so repeated reloads cannot accumulate old generations indefinitely.
