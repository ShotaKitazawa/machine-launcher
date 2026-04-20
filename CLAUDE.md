# CLAUDE.md

## Commands

All commands must be run via `mise run <task>`. Do not invoke `cargo`, `trunk`, or other tools directly.
All `mise run` tasks may be executed without human confirmation.

## Development

```
mise run dev
```

- Backend: http://localhost:8080 (hot-reload via systemfd + cargo-watch)
- Frontend: http://localhost:3000 (hot-reload via trunk serve, proxies /api to :8080)

`dev-backend` always passes `--disable-oidc`, so OIDC auth is bypassed automatically.
The backend requires a config file — default is `backend/example.toml`.
Override with `CONFIG=path/to/config.toml mise run dev`.

`example.toml` uses `${VAR}` env-var substitution. Required vars for production: `URL`, `AUTH0_CLIENT_ID`, `OIDC_SUB`, `IPMI_USERNAME`, `IPMI_PASSWORD`.

## OpenAPI workflow

1. Edit `openapi.yaml`
2. `mise run generate` (regenerates from utoipa annotations in backend source)
3. Update handler implementation if needed

**Never edit `openapi.yaml` directly** — it is fully generated from utoipa annotations.

## Authentication (OIDC + PKCE)

The frontend implements PKCE (Authorization Code + PKCE) entirely in the browser:
- Generates code_verifier / code_challenge and nonce client-side (sessionStorage)
- Exchanges the authorization code for tokens directly with the IdP token endpoint
- Sends the ID token as `Authorization: Bearer` on each API request

The backend only verifies the incoming ID token signature (via `openidconnect` crate). There is no server-side nonce store.

### Authorization

Allowed users are configured via `allowed_subs` in `[oidc]` config, or via `OIDC_ALLOWED_SUBS` env var (comma-separated). Config takes precedence. If neither is set, all authenticated users are allowed.

### `--disable-oidc` flag

When started with `--disable-oidc`:
- All API requests bypass auth middleware
- `GET /api/oidc-config` returns `{"enabled": false}`
- Frontend hides login/logout UI and treats the session as already authenticated

**Never use `--disable-oidc` in production.**

## Notes

- `OidcState` enum in `lib.rs` carries all OIDC-related state; `Disabled` variant means auth is off
- `jmespath` crate was intentionally removed — authorization is sub-based only
- Backend binary selection: `cargo run --bin machine-launcher` (not `generate_openapi`)
