# Local lifecycle contract

iso-lattes desktop intentionally mirrors the production lifecycle without sharing implementation code.

States:

- `active`: V8 isolate cell may receive invocations.
- `idle`: resident cell has no in-flight request but remains reusable for the same tenant + deployment generation.
- `frozen`: execution is suspended; no new traffic is admitted to the cell.
- `reclaiming`: host may reclaim/swap cold pages while the workload remains logically frozen.
- `draining`: no new invocations; in-flight work is allowed to finish before generation retirement.
- `stopped`: worker state is gone; immutable deployment artifacts remain available for restart.

Rules:

1. Reuse is allowed only for the same tenant and immutable deployment generation.
2. Freeze and memory reclaim are distinct operations; freezing alone is never reported as reclaimed RAM.
3. Generation activation is prepare -> validate -> stage -> health-check -> activate -> drain old generation -> retire/rollback.
4. The Rust desktop daemon is the sole machine-lifecycle writer. CLI and desktop app are clients only.
5. An Erlang supervisor may be launched as the long-lived granddaddy process for local parity with production orchestration.
6. Payloads and credentials never belong in argv, logs, or desired-state files.
