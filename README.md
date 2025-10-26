<p align="center">
	<img src="tiger-logo.png" alt="Tigrinho logo" width="720" />
</p>

# tigrinho-rs

Prototype slot-like engine and demo built in Rust with a provably-fair RNG.

Important security/legal caveats:
- This is a technical prototype for educational purposes only. Do NOT use for real-money gambling.
- No cryptographic construction here is audited. Do not deploy publicly without professional review.
- Keep the server seed strictly secret and rotate regularly. Never log it; only publish its hash.
- RTP tuning and compliance require regulation-specific processes; this repo does not cover that.

Workspace layout:
- tigrinho_shared: Shared API/request/response types (serde)
- tigrinho_core: Core engine: RNG, reels, paytable, payouts, RTP sim
- tigrinho_server: Axum HTTP server with sqlite audit log
- tigrinho_cli: Admin CLI to rotate seed, view logs, export CSV
- tigrinho_wasm: Yew frontend (WASM) to demo spins and verification

Provably fair overview:
1) Server picks random secret server_seed and publishes server_seed_hash = SHA256(server_seed)
2) Client provides client_seed and nonce per spin.
3) Outcome RNG uses HMAC-SHA256(server_seed, client_seed || ":" || nonce) and derives random floats.
4) Anyone can later verify the outcome by recomputing with revealed server_seed.

Quick start (server locally):
- Build core/server/cli: `cargo build`
- Run DB migrations on first start automatically.
- Start server: `cargo run -p tigrinho_server`
- Spin example: POST /spin with JSON {"client_seed":"abc","bet":1.0,"lines":10}

WASM (optional):
- Install trunk: `cargo install trunk`
- Add wasm toolchain: `rustup target add wasm32-unknown-unknown`
- Run: `cd tigrinho_wasm && trunk serve`

Testing:
- `cargo test` runs unit tests and RTP simulation smoke tests.

Endpoints (server):
- GET /verify -> { server_seed_hash }
- POST /spin -> { server_seed_hash, nonce, reels, payout }
- POST /admin/set-params (Authorization: Bearer <API_KEY>) -> 204; body { rtp_target, paytable[] }

Run server (Windows PowerShell):
```
$env:API_KEY = "dev-key"
$env:DATABASE_URL = "sqlite://tigrinho.db"
cargo run -p tigrinho_server
```

CLI usage:
- Rotate seed: `cargo run -p tigrinho_cli -- rotate-seed NEW_SERVER_SEED`
- View logs: `cargo run -p tigrinho_cli -- view-logs 20`
- Export CSV: `cargo run -p tigrinho_cli -- export-csv spins.csv`

Provably fair verification:
- After rotating the server seed, publish the old server_seed and its hash so users can verify past spins.
- Verification formula: HMAC-SHA256(key=server_seed, msg=client_seed||":"||nonce). Convert bytes to floats as in `tigrinho_core::derive_floats`.

## observations and troubleshooting (Windows)

- SQLite database URL/path:
	- If you get `(code: 14) unable to open database file`, it usually means the file path is invalid or not writable.
	- Try one of these from the repo root:
		- Relative file: `$env:DATABASE_URL = "sqlite://./tigrinho.db"`
		- Absolute file (use forward slashes and three slashes after scheme):
			`$env:DATABASE_URL = "sqlite:///C:/Users/<you>/path/to/tigrinho.db"`
	- Ensure the directory exists and you have write permissions.
	- For quick, non-persistent testing: `$env:DATABASE_URL = "sqlite::memory:"` (no logs persisted across runs).

- Run server and call endpoints in separate terminals:
	- Start the server in one terminal. In another, call endpoints, e.g.:
		- `Invoke-WebRequest http://127.0.0.1:8080/verify | Select-Object -ExpandProperty Content`
	- Stopping the server with Ctrl+C will terminate it; avoid running requests in the same foreground run terminal.

- Known warnings (harmless for the demo):
	- The WASM canvas demo uses `CanvasRenderingContext2d::set_fill_style`, which is deprecated but fine for this prototype.
	- You may see "unused import" warnings; they can be cleaned up with `cargo fix` but are non-blocking.

- CI and WASM:
	- The wasm crate is not part of default-members to keep CI lean. To build it locally: `cargo build -p tigrinho_wasm --target wasm32-unknown-unknown` or run `trunk serve` in its crate.

## quick test (PowerShell)

Run the server (Terminal A) using an in-memory SQLite DB (no files created):

```pwsh
$env:API_KEY = "dev-key"
$env:DATABASE_URL = "sqlite::memory:"
cargo run -p tigrinho_server
```

From another terminal (Terminal B), call the API:

Verify current server seed hash:
```pwsh
Invoke-RestMethod http://127.0.0.1:8080/verify
```

Spin once with a client seed (bet=1.0, lines=1):
```pwsh
$body = @{ client_seed = "demo-seed"; bet = 1.0; lines = 1 } | ConvertTo-Json
Invoke-RestMethod -Method POST -Uri http://127.0.0.1:8080/spin -ContentType "application/json" -Body $body
```

Optionally update RTP/paytable (requires API key via Bearer token):
```pwsh
$paytable = @(
	@{ symbol = 0; count = 3; payout_multiplier = 5.0 },
	@{ symbol = 1; count = 3; payout_multiplier = 4.0 },
	@{ symbol = 2; count = 3; payout_multiplier = 3.0 },
	@{ symbol = 3; count = 3; payout_multiplier = 2.0 },
	@{ symbol = 4; count = 3; payout_multiplier = 10.0 }
)
$body = @{ rtp_target = 0.95; paytable = $paytable } | ConvertTo-Json -Depth 5
Invoke-RestMethod -Method POST -Uri http://127.0.0.1:8080/admin/set-params -Headers @{ Authorization = "Bearer dev-key" } -ContentType "application/json" -Body $body
```

Notes:
- Using `sqlite::memory:` means spins are not persisted. For persistence, set `DATABASE_URL` to a file path (see troubleshooting above).
- If you change the paytable, it applies to subsequent spins.

