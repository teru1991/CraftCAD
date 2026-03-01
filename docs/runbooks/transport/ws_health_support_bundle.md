# WS health & support bundle

## One-command collection

### Bash

```bash
bash scripts/collect_support_bundle.sh http://127.0.0.1:8080
```

### PowerShell

```powershell
powershell -ExecutionPolicy Bypass -File scripts/collect_support_bundle.ps1 -BaseUrl http://127.0.0.1:8080
```

## Artifacts

- `healthz.json`: status/reasons
- `support_bundle.json`: metrics/events/rules snapshot (no secrets)
- `metrics.prom`: Prometheus scrape output
