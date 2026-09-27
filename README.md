# ISO Lattes Desktop Infra

Single-host desired-state and local deployment authority for ISO Lattes on developer- or end-user-owned desktops/laptops.

## Local deployment

The repository uses `ORESoftware/ores-compose` as its local orchestrator. The compose file pins the desktop daemon to an immutable Git commit under `tmp/dev`.

```sh
ores-compose check .ores-compose.yaml
ores-compose plan .ores-compose.yaml
ores-compose up .ores-compose.yaml
```

For public mode, the product control plane provisions a remotely managed Cloudflare Tunnel and DNS route; the device receives only a per-tunnel run token stored outside the repository. No dedicated/static public IP or router port-forward is required.

See [docs/local-deployment.md](docs/local-deployment.md) and [appliance.json](appliance.json).
