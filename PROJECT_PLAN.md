# AI Privacy Gateway — 6-Week MVP Execution Plan

**Created:** February 6, 2026  
**Co-Founders:** Benjamin Silva & Jacob Luna  
**Repo:** `github.com/bssilva06/mantle`  
**Status:** Pre-development (documentation only, zero source files)

---

## Git Branching Strategy

**Model: Trunk-based development with short-lived feature branches**

```
main (protected — always deployable)
 └── develop (integration branch — merge target for all work)
      ├── feat/proxy-core         (Person A)
      ├── feat/auth-middleware     (Person A)
      ├── feat/detection-engine    (Person B)
      ├── feat/vault-rehydration   (Person B)
      └── ...
```

### Rules

1. **Never push directly to `main` or `develop`.** All work goes through PRs.
2. Branch naming: `feat/`, `fix/`, `infra/`, `docs/` prefixes.
3. **Short branches** — merge within 2–3 days max to avoid drift.
4. Both founders rebase on `develop` daily (`git pull --rebase origin develop`).
5. One person reviews the other's PR before merge — this doubles as knowledge sharing.
6. `main` ← `develop` merge only at phase milestones (end of Week 2, 4, 6) after joint testing.
7. **First task together:** Scaffold the repo (project structure, `pyproject.toml`, `.gitignore`, Docker) on a single shared call to avoid day-1 conflicts.

---

## Work Split Philosophy

The codebase has two natural seams: **"data in" (proxy + auth + infra)** vs **"privacy logic" (detection + surrogates + vault + rehydration)**. Each person owns a seam with clear module boundaries and shared interfaces (Pydantic schemas) defined together upfront.

| Owner                         | Domain                                                                  | Key Modules                                                                                |
| ----------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| **Person A (Infra & Proxy)**  | Everything that touches the network, auth, DB, deployment               | API routes, middleware, provider adapters, database, Docker, CI/CD, observability          |
| **Person B (Privacy Engine)** | Everything that touches PII — detection, surrogates, vault, rehydration | Detection engine, Presidio, Rust core, Faker generator, Fernet vault, streaming rehydrator |

> **Decide who is A vs B based on strengths** — if one person knows Rust, they should be Person B. If one knows DevOps/infra better, they're Person A.

---

## Phase 1: Foundation (Weeks 1–2)

**Goal:** Working non-streaming reverse proxy with authentication and observability

### Day 1 — Together (pair session)

- Initialize repo structure, `.gitignore`, `pyproject.toml`, `Dockerfile`, `docker-compose.yml`
- Define shared Pydantic schemas (`ChatCompletionRequest`, `ChatCompletionResponse`, `DetectedEntity`)
- Set up `develop` branch protection rules on GitHub
- Agree on module boundaries and import contracts

### Person A (Proxy & Infra)

| #   | Branch                   | Task                                                                                  |
| --- | ------------------------ | ------------------------------------------------------------------------------------- |
| 1   | `feat/fastapi-scaffold`  | FastAPI app with uvloop, `/health` + `/ready` endpoints                               |
| 2   | `feat/provider-adapters` | `ProviderAdapter` base class + OpenAI + Anthropic adapters                            |
| 3   | `feat/database`          | PostgreSQL + Alembic migrations (tenants, api_keys, usage_logs tables + RLS policies) |
| 4   | `feat/auth-middleware`   | API key authentication middleware with SHA-256 hashing                                |
| 5   | `feat/docker`            | Multi-stage Dockerfile + docker-compose (FastAPI + Postgres + Redis)                  |
| 6   | `feat/langfuse`          | Self-hosted Langfuse setup + trace integration in request lifecycle                   |

### Person B (Privacy Engine)

| #   | Branch               | Task                                                                                                           |
| --- | -------------------- | -------------------------------------------------------------------------------------------------------------- |
| 1   | `feat/logging`       | structlog setup with `scrub_pii` processor, `ProcessorFormatter`, `foreign_pre_chain`, third-party suppression |
| 2   | `feat/redis-client`  | Redis async client, connection pooling, health check                                                           |
| 3   | `feat/chat-endpoint` | `/v1/chat/completions` non-streaming endpoint (wires Person A's adapters to request flow)                      |
| 4   | `feat/schemas`       | Shared Pydantic models, exception hierarchy, config module (`pydantic-settings`)                               |
| 5   | —                    | Begin research/prototyping Presidio integration locally (not merged yet)                                       |

### Phase 1 Milestone

PR `develop` → `main`. **Joint demo:** authenticate a tenant, proxy a request to OpenAI, get a response, see it traced in Langfuse with zero PII in logs.

---

## Phase 2: Core Privacy Logic (Weeks 3–4)

**Goal:** Parallel PII detection, Faker surrogates, streaming rehydration

### Person A (Streaming & Integration)

| #   | Branch                   | Task                                                                                |
| --- | ------------------------ | ----------------------------------------------------------------------------------- |
| 1   | `feat/sse-streaming`     | SSE support with `sse-starlette`, handle OpenAI/Anthropic stream format differences |
| 2   | `feat/system-prompt`     | System prompt injection for surrogate preservation (configurable per-tenant)        |
| 3   | `feat/rehydrate-header`  | `X-Privacy-Gateway-Rehydrate: false` header support                                 |
| 4   | `feat/usage-tracking`    | Per-tenant usage metering (tokens, requests, entities detected)                     |
| 5   | `feat/integration-tests` | End-to-end tests: full request → detect → surrogate → proxy → rehydrate pipeline    |
| 6   | `feat/garak`             | garak security scanning integration in CI                                           |

### Person B (Detection + Vault + Rehydration)

| #   | Branch                   | Task                                                                                      |
| --- | ------------------------ | ----------------------------------------------------------------------------------------- |
| 1   | `feat/presidio-detector` | Presidio Analyzer with NER-only spaCy pipeline (`en_core_web_md`)                         |
| 2   | `feat/detection-engine`  | Parallel dual-tier orchestrator (`asyncio.gather`), result merging + deduplication        |
| 3   | `feat/faker-surrogates`  | Faker generator with request-scoped consistency + collision avoidance                     |
| 4   | `feat/fernet-vault`      | Encrypted Redis vault (Fernet, TTL enforcement, key derivation, cryptographic erasure)    |
| 5   | `feat/stream-rehydrator` | `StreamingFakerRehydrator` with adaptive sliding window buffer                            |
| 6   | `feat/deanonymization`   | Multi-strategy matching (exact → case-insensitive → fuzzy → n-gram) + rehydration metrics |

### Phase 2 Milestone

PR `develop` → `main`. **Joint demo:** send a prompt with PII, see Faker surrogates in Langfuse trace, stream back a rehydrated response with >85% surrogate match rate.

---

## Phase 3: Rust Performance + Production (Weeks 5–6)

**Goal:** Optimize latency-critical paths with Rust, harden for production

### Person A (Production Hardening & Deploy)

| #   | Branch                     | Task                                                                                  |
| --- | -------------------------- | ------------------------------------------------------------------------------------- |
| 1   | `feat/rate-limiting`       | Token bucket rate limiter (free/pro/enterprise tiers) via Redis                       |
| 2   | `feat/circuit-breaker`     | Circuit breaker for upstream providers (5 failures → open → 30s cooldown)             |
| 3   | `feat/prometheus`          | Prometheus metrics endpoint (latency percentiles, detection rates, rehydration stats) |
| 4   | `feat/ecs-deploy`          | AWS ECS Fargate deployment config, auto-scaling, CloudWatch alarms                    |
| 5   | `feat/docker-compose-prod` | Production-ready Docker Compose for self-hosted users                                 |
| 6   | `docs/readme`              | README with quick-start guide (<5 min to first masked request), cURL examples         |
| 7   | `feat/garak-full`          | Comprehensive garak security suite run                                                |

### Person B (Rust Core)

| #   | Branch                  | Task                                                                 |
| --- | ----------------------- | -------------------------------------------------------------------- |
| 1   | `feat/maturin-init`     | Maturin project structure under `rust/` directory                    |
| 2   | `feat/rust-detector`    | Rust `aho-corasick` pattern matcher for Tier 1 PII detection         |
| 3   | `feat/rust-rehydrator`  | Rust `aho-corasick` streaming matcher for surrogate rehydration      |
| 4   | `feat/rust-replacer`    | Token replacement logic in Rust with `bstr`                          |
| 5   | `feat/pyo3-bindings`    | PyO3 bindings with GIL release, `cast()` over `extract()`, Bound API |
| 6   | `feat/rust-benchmarks`  | Benchmark Rust vs Python, verify <5ms p99 bridge latency             |
| 7   | `feat/manylinux-wheels` | Build manylinux wheels for distribution                              |

### Phase 3 Milestone

PR `develop` → `main`, tag `v0.1.0`. Full production deployment. <50ms p99 detection, 500+ RPS, README published, Show HN ready.

---

## Key Considerations

1. **Conflict hotspot: shared Pydantic schemas** — Both people will touch shared types. Define all Pydantic models together on Day 1 and treat schema changes as requiring both people's sign-off.
2. **Rust experience:** If neither founder knows Rust, consider doing Phase 2 with a pure-Python Tier 1 detector (regex) first, then layer in Rust in Phase 3 — the PRD supports this since it's a performance optimization, not a functional requirement.
3. **Daily syncs:** 15-min standup to flag blockers and coordinate merges. End-of-week joint demo on Friday to catch integration issues early before they compound.

---

## Earlier Chat Context

### Project Ideas Discussion

Before settling on the AI Privacy Gateway, we discussed unique developer tool project ideas. Top picks for impact:

| Project                         | Why It Stands Out                                                                     |
| ------------------------------- | ------------------------------------------------------------------------------------- |
| **Error Message Search Engine** | Every developer hits this pain daily; cutting debugging time by even 20% is massive   |
| **Config File Translator**      | Migration fatigue is real; solves a concrete, recurring problem with a clear audience |
| **API Contract Drift Detector** | Catches bugs before users do; valuable for any team with APIs, few good tools exist   |

### Copilot vs Cursor

- **GitHub Copilot** is built natively into VS Code by GitHub/Microsoft. Full extension marketplace compatibility, always current with VS Code updates, free tier + $10/mo Pro.
- **Cursor** is a fork of VS Code — a separate app. Similar AI agent capabilities, $20/mo Pro, can lag behind VS Code updates.
- Functionally converging — neither has a decisive edge. Main difference is Copilot lives inside official VS Code, Cursor requires switching editors.
