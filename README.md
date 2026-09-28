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

The common layer is now pinned as a candidate at `73ebe4b3caa7478c45f64a2eb983838a91ff9732`. The appliance remains candidate-only until the pinned common revision passes its own CI/conformance gates and its Zed package artifact is promoted; product repositories must not fall back to a mutable branch ref.

