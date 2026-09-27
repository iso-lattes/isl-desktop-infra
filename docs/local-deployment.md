# Local desktop deployment

This repository follows the ORES desktop-appliance contract.

## Local orchestration

`ores-compose` owns the local process lifecycle. The compose manifest pins the desktop daemon to an immutable Git commit and materializes it below `tmp/dev`.

```sh
ores-compose check .ores-compose.yaml
ores-compose plan .ores-compose.yaml
ores-compose up .ores-compose.yaml
```

Stop from another terminal with:

```sh
ores-compose down .ores-compose.yaml
```

The daemon remains loopback-bound and is the privileged local control boundary. Ambient product credentials/tokens must come from the desktop bootstrap or OS secret store; they are never committed to this repo.

## Connectivity

Public mode uses a remotely managed Cloudflare Tunnel:

```sh
export ISL_CF_TUNNEL_TOKEN_FILE=/path/to/os-protected/tunnel-token
ores-compose up .ores-compose.public.yaml
```

The product control plane—not the laptop—creates the tunnel, DNS route, hostname, and remote ingress config. The device only receives the per-tunnel run token. No public/static/dedicated IP, router port-forward, or inbound firewall rule is required.

The Cloudflare origin is `http://127.0.0.1:8761`. Remote ingress should be constrained to `/v1/invoke`; do not expose the rest of the loopback daemon surface.

The token file must live outside the repository and should be stored behind the OS credential boundary (Keychain/DPAPI/Secret Service or equivalent).

## Upgrade model

Upgrades change the immutable `source.commit` in the compose manifest after upstream CI is green. Do not use mutable `latest` refs for desktop production/candidate channels.
