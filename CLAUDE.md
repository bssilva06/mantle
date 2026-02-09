# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**AI Privacy Gateway** — a high-performance reverse proxy that intercepts API requests to LLM providers (OpenAI, Anthropic, Azure OpenAI), detects PII in real-time, replaces it with Faker-based realistic surrogates, vaults the originals with Fernet encryption in Redis, and rehydrates responses before returning them to the client.

- **Co-Founders:** Benjamin Silva & Jacob Luna
- **Repo:** `github.com/bssilva06/mantle`
- **Status:** Pre-development (documentation only — PRD v3.0 and PROJECT_PLAN.md exist, zero source files)

## Architecture

### Request Pipeline (5 phases)
1. **Ingestion** — Client sends `POST /v1/chat/completions` → API key auth middleware → Pydantic validation
2. **Parallel Detection** — Tier 1 (Rust `aho-corasick` regex, <1ms) and Tier 2 (Presidio NER via `en_core_web_md`, 5-20ms) run concurrently via `asyncio.gather`, results merged/deduplicated by span position
3. **Faker Surrogate Generation & Vaulting** — PII replaced with locale-appropriate Faker values (request-scoped consistency, collision avoidance), originals encrypted with Fernet and stored in Redis with TTL
4. **Upstream Proxy** — System prompt prepended for surrogate preservation, request transformed via `ProviderAdapter`, sent to LLM
5. **Streaming Rehydration** — `StreamingFakerRehydrator` with `aho-corasick` automaton + adaptive sliding window buffer, multi-strategy deanonymization (exact → case-insensitive → fuzzy → n-gram)

### Tech Stack
- **API:** FastAPI + uvloop, httpx (HTTP/2), sse-starlette
- **Language:** Python 3.11+ with Rust (PyO3/Maturin) for performance-critical paths
- **PII Detection:** Microsoft Presidio + spaCy `en_core_web_md` (Tier 2), Rust `aho-corasick` (Tier 1)
- **Surrogates:** Faker library with `fuzzywuzzy` + `python-Levenshtein` for matching
- **Vault:** Redis (Fernet encryption, TTL enforcement)
- **Database:** PostgreSQL + SQLAlchemy 2.0 + Alembic (RLS for multi-tenancy)
- **Logging:** structlog with `ProcessorFormatter` + `foreign_pre_chain` for PII scrubbing across all loggers
- **Observability:** Self-hosted Langfuse for distributed tracing, Prometheus metrics
- **Security Testing:** garak (vulnerability scanning), promptfoo (prompt evaluation)
- **Containers:** Docker, docker-compose (LocalStack for local AWS emulation)

### Key Module Boundaries (two-person split)
- **Person A (Infra & Proxy):** API routes, middleware, provider adapters, database, Docker, CI/CD, SSE streaming, observability
- **Person B (Privacy Engine):** Detection engine, Presidio integration, Rust core, Faker generator, Fernet vault, streaming rehydrator

## Build & Development Commands

No build system exists yet. When scaffolded, the project will use:
- `pyproject.toml` for Python packaging
- `maturin` for building Rust/PyO3 extensions
- `docker-compose` for local development (FastAPI + PostgreSQL + Redis + LocalStack)
- `alembic` for database migrations
- GitHub Actions CI: `ruff check`, `pytest`, `mypy` on every PR to `develop`

## Git Workflow

- **Trunk-based** with short-lived feature branches off `develop`
- Branch naming: `feat/`, `fix/`, `infra/`, `docs/` prefixes
- Never push directly to `main` or `develop` — all work through PRs
- `main` ← `develop` merge only at phase milestones (end of Weeks 2, 4, 6)
- Rebase on `develop` daily: `git pull --rebase origin develop`

## Critical Design Decisions

- **Faker surrogates over abstract placeholders** — `{{TYPE_N}}` tokens cause LLMs to paraphrase; Faker names/emails are treated as natural text, improving rehydration accuracy
- **Always-parallel detection** — Both tiers run concurrently on every request; never skip Tier 2 conditionally. Co-occurring structured + unstructured PII is the norm
- **Fernet over AES-256-GCM** — For ephemeral data (<10min TTL), Fernet's impossible-to-misuse API eliminates nonce-reuse risk. Migration path to AES-256-GCM documented in PRD Appendix C
- **Fail-secure on Redis unavailability** — Return HTTP 503, never fail open with Faker surrogates visible to client
- **Adaptive streaming timeout** — `max(50ms, 3 × EWMA_inter_chunk_interval)` instead of fixed 500ms
- **AWS endpoint URL pattern** — All AWS clients use `AWS_ENDPOINT_URL` env var so same code works against LocalStack (dev) and real AWS (prod)
- **Shared Pydantic schemas are contracts** — Changes to shared models (`ChatCompletionRequest`, `ChatCompletionResponse`, `DetectedEntity`) require both founders' sign-off

## Performance Targets (MVP)

- Detection latency: <50ms p99
- Rust bridge overhead: <5ms p99
- Rehydration success rate: >85%
- Throughput: 500+ RPS/instance
