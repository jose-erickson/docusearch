# SuccessFactors Employee Central Microservice

A Rust microservice for reading SAP SuccessFactors Employee Central data via the OData v2 API, built with security as a first-class concern.

## Features

- RESTful HTTP API backed by `axum`
- SuccessFactors OData v2 client using `reqwest` + `rustls`
- Entities: User, PerPersonal, EmpJob, EmpEmployment, PerEmail, PerPhone
- Safe pagination (`top` / `skip`) with enforced bounds
- Structured JSON logs, CORS, graceful shutdown

## Security Controls

| Concern | Control |
|---|---|
| Credential exposure | `secrecy::SecretString` wraps password and API key; zeroed on drop; redacted in `Debug` output |
| Transport security | HTTPS enforced on `SF_API_URL` (HTTP only for localhost); TLS 1.2+ minimum on client; `rustls` TLS stack (no OpenSSL) |
| Client auth to upstream | HTTP Basic over TLS; cookie store disabled; redirects disabled (prevents credential leak to unexpected hosts) |
| Service authentication | `X-API-Key` header required on all `/api/*` routes; constant-time comparison (`subtle::ConstantTimeEq`) prevents timing side-channels; minimum 32-char key |
| OData injection | User-supplied IDs have `'` doubled (OData escape) and are percent-encoded before URL embedding; charset allowlist on path params |
| Input validation | Pagination clamped to `MAX_PAGE_SIZE=500` / `MAX_SKIP=100_000`; user IDs limited to 128 chars, alphanumeric + `-_.@` |
| Rate limiting | Per-IP token bucket (`governor`) with bounded in-memory map to prevent memory exhaustion |
| Request size | Body size capped via `RequestBodyLimitLayer` |
| Response size | Upstream responses capped at 10 MB to prevent memory exhaustion |
| Request timeout | Server-side and client-side timeouts configurable |
| Error disclosure | Upstream error bodies/URLs never forwarded; all 4xx/5xx responses sanitized; details only in server logs |
| Security headers | `X-Content-Type-Options`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, HSTS, strict CSP, Permissions-Policy |
| CORS | Allowlist-based; `*` not supported; empty list = no cross-origin |
| Graceful shutdown | SIGINT/SIGTERM drain in-flight requests |
| Build hardening | `panic=abort`, LTO, symbol stripping in release profile |

## Threat Model Notes

- **Intended deployment:** behind a TLS-terminating reverse proxy (e.g. nginx, Envoy, ALB) inside a trusted network. The service binds to `127.0.0.1` by default.
- **Out of scope:** request signing, per-user RBAC, mTLS, and audit logging of returned PII. Add these if required by your compliance regime (GDPR, CCPA, etc.). HR data is sensitive — treat all returned records as personal data.
- **Secrets:** `.env` is for local development only. In production, inject via your secret manager. Never commit `.env`.

## Prerequisites

- Rust 1.75+
- SuccessFactors account with OData API access
- A 32+ char service API key: `openssl rand -base64 48`

## Configuration

Copy and edit:

```bash
cp .env.example .env
```

All required variables are documented in `.env.example`.

## Build & Run

```bash
cargo build --release
cargo run --release
```

## API Endpoints

All `/api/*` routes require the `X-API-Key` header.

| Endpoint | Auth | Description |
|---|---|---|
| `GET /health` | none | Liveness check |
| `GET /api/test-connection` | api key | Verify SF connectivity |
| `GET /api/users` | api key | List users |
| `GET /api/users/:user_id` | api key | Fetch one user |
| `GET /api/employees/personal` | api key | List personal info |
| `GET /api/employees/jobs` | api key | List job info |
| `GET /api/employees/employment` | api key | List employment info |
| `GET /api/employees/emails` | api key | List emails |
| `GET /api/employees/phones` | api key | List phones |

Query params: `top` (1..=500, default 50), `skip` (0..=100000, default 0).

### Example

```bash
curl -H "X-API-Key: $SERVICE_API_KEY" \
  "https://your-host/api/users?top=25&skip=0"
```

## Project Layout

```
Cargo.toml
.env.example
src/
├── main.rs         # server bootstrap, middleware stack, shutdown
├── config.rs       # env parsing, validation, secret handling
├── client.rs       # SuccessFactors OData client (Basic auth, TLS, caps)
├── auth.rs         # API key middleware (constant-time compare)
├── rate_limit.rs   # per-IP token bucket
├── handlers.rs     # route handlers, input validation, error mapping
└── models.rs       # DTOs and error envelope
```

## Testing

```bash
cargo test
```

Unit tests cover OData escaping, user ID validation, and pagination bounds.

## Operational Recommendations

- Run behind a TLS-terminating proxy; do not expose port 3000 directly.
- Rotate `SERVICE_API_KEY` and the SuccessFactors password on a schedule.
- Ship JSON logs to a SIEM and alert on `rate_limit`, `auth`, and `upstream_error` events.
- Run `cargo audit` in CI to detect vulnerable dependencies.
- Keep dependencies current; rebuild and redeploy on each advisory.

## License

MIT
