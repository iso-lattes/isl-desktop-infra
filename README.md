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

This repository pins the shared desktop implementation `ORESoftware/ores-common-desktop-infra` at exact revision `7bb4ed89ab4aa4a81c5e26e36b91f58d6313cc7c`.

Generic consumer validation, loopback/secret policy, Cloudflare promotion policy, update/lifecycle rules, readiness policy, and structured-log redaction are owned by that common layer. The local appliance records the exact pin; `common_layer_ci_verified` intentionally remains false until the private cross-org certification workflow runs successfully.

See [docs/local-deployment.md](docs/local-deployment.md) and [the certification workflow](.github/workflows/common-layer-certification.yml).

