# Product Requirements Document: AI Privacy Gateway

**Project Name:** AI Privacy Gateway (B2B SaaS)  
**Author:** Benjamin Silva  
**Version:** 3.0 (Revised Architecture — Feedback-Driven)  
**Date:** February 6, 2026  
**Status:** Planning Phase  

---

## 1. Executive Summary

The AI Privacy Gateway is a high-performance reverse proxy that solves the "black box" privacy challenge in enterprise Generative AI adoption. By intercepting API requests to LLM providers (OpenAI, Anthropic, Azure OpenAI), detecting personally identifiable information (PII) in real-time, and replacing it with reversible surrogate data, the Gateway enables enterprises to leverage AI capabilities without exposing sensitive data to third-party model training or security breaches.

**Core Value Proposition:** Zero-trust privacy architecture that decouples LLM utility from data liability through real-time PII detection, vaulting, and rehydration — all while maintaining transparent API compatibility.

**Target Market:** Technology companies, AI startups, and SaaS platforms requiring GDPR/privacy compliance for LLM integration (initial focus: non-healthcare enterprises).

**Key Architectural Decisions (v3.0):**
- **Faker-based realistic surrogates** instead of abstract placeholder tokens — LLMs treat synthetic names/emails as natural text, dramatically improving rehydration accuracy
- **Parallel dual-tier PII detection** — Rust regex and Presidio NER always run concurrently, eliminating the conditional logic gap that could miss co-occurring PII
- **Fernet encryption** for the ephemeral vault with built-in TTL enforcement, migrating to AES-256-GCM only if compliance or scale demands it
- **Adaptive sliding window buffer** for streaming rehydration with dictionary-based matching against active Faker values
- **Defense-in-depth PII-safe logging** via structlog with `ProcessorFormatter` capturing all third-party library output
- **6-week focused MVP** shipping proxy + detection + vault + streaming + auth + deployment, with MCP server, SDK generation, admin dashboard, and marketing deferred to post-validation phases

---

## 2. Problem Statement

### 2.1 The Enterprise AI Privacy Dilemma

**The Conflict:** Enterprises want to leverage powerful LLMs (GPT-4, Claude Sonnet) but are blocked by data sovereignty regulations (GDPR Article 25, CCPA) and internal security policies prohibiting transmission of PII to third-party APIs.

**The Risk:** Unstructured data in user prompts is inherently difficult to sanitize. Once sensitive information crosses the enterprise boundary:
- It may be ingested into model training pipelines (despite vendor no-train policies)
- It becomes vulnerable to vendor security breaches
- It creates compliance violations with potential fines exceeding €20M or 4% of global revenue (GDPR)

**Current Market Gap:** Existing solutions operate in silos:
- **LLM Gateways** (LiteLLM, Portkey): Focus on routing/observability without privacy detection
- **PII Detection Tools** (Presidio, Private AI): Provide detection but lack LLM-specific integration
- **No strong open-source solution** combines both capabilities with streaming support and production-ready performance

### 2.2 Why Now?

- **Regulatory Pressure:** EU AI Act enforcement began 2025, requiring documented PII safeguards
- **Enterprise Adoption Blockers:** 67% of enterprises cite data privacy as top barrier to LLM adoption (Gartner 2024)
- **Technical Feasibility:** Hybrid Python/Rust architectures (PyO3) now enable real-time performance at acceptable development cost
- **Market Timing:** No open-source product combines LLM gateway routing with real-time streaming PII detection today

---

## 3. Goals & Success Metrics

### 3.1 Primary Goals

**PG-1: Zero-Trust Privacy Architecture**  
Implement a "round-trip" architecture where sensitive data never leaves enterprise control in its original form. All PII is detected, replaced with realistic synthetic data, vaulted with encryption, and rehydrated post-inference.

**PG-2: Transparent Developer Experience**  
Maintain 100% API compatibility with OpenAI's format so existing client applications can switch the `base_url` parameter and immediately benefit from privacy protection without code changes.

**PG-3: Production-Grade Performance**  
Minimize latency overhead to ensure the privacy layer doesn't degrade user experience. Target sub-50ms p99 detection latency with parallel dual-tier detection.

**PG-4: Multi-Provider Flexibility**  
Abstract provider-specific APIs (OpenAI, Anthropic, Azure OpenAI) behind a unified interface, enabling customers to switch providers without reconfiguration.

### 3.2 Key Performance Indicators (KPIs)

| Metric | MVP Target (Week 6) | Production Target | Measurement Method |
|--------|---------------------|-------------------|-------------------|
| **Detection Latency** | <50ms p99 | <30ms p99 | Langfuse distributed tracing |
| **Rust Bridge Overhead** | <5ms p99 | <2ms p99 | Langfuse spans |
| **Rehydration Success Rate** | >85% | >95% | Custom metric: matched/total surrogates |
| **Throughput** | 500 RPS/instance | 1,000+ RPS/instance | Load testing with k6 |
| **Detection Accuracy (F1)** | >85% | >90% | Benchmark against labeled test dataset |
| **Infrastructure Cost** | <$300/month | <$2,000/month at 10M req/day | AWS Cost Explorer |
| **Uptime** | 99.5% | 99.9% | Uptime monitoring |
| **API Compatibility** | 95% OpenAI endpoints | 100% coverage | Integration test suite |

### 3.3 Non-Goals (Out of Scope for MVP)

- ❌ HIPAA compliance and healthcare-specific entity types (future roadmap)
- ❌ On-premise deployment options (cloud-only for MVP)
- ❌ Custom LLM fine-tuning or model hosting
- ❌ Data loss prevention (DLP) beyond PII detection
- ❌ Real-time prompt injection detection (integrate Lakera Guard as optional addon)
- ❌ MCP server exposure (deferred to Phase 2)
- ❌ Auto-generated SDKs (deferred to Phase 2 — users use standard OpenAI SDK with changed `base_url`)
- ❌ Admin dashboard UI (deferred to Phase 2 — CLI/config for MVP)
- ❌ Stripe billing integration (deferred to Phase 2 — manual onboarding for MVP beta)

---

## 4. Functional Requirements

### 4.1 Reverse Proxy Core

**FR-01: TLS Termination & Inspection**  
The system MUST terminate TLS connections to inspect request/response payloads for PII detection. All upstream connections to LLM providers MUST use TLS 1.2+ with certificate validation.

**FR-02: HTTP/2 Support**  
The system MUST support HTTP/2 to handle multiplexed connections efficiently, reducing connection overhead for high-concurrency scenarios.

**FR-03: OpenAI-Compatible Canonical Format**  
The system MUST use OpenAI's `/v1/chat/completions` format as the internal canonical standard. Requests to other providers (e.g., Anthropic's Messages API) MUST be transformed via provider-specific adapters that handle:
- Different authentication headers (`Authorization: Bearer` vs `x-api-key`)
- System message placement (messages array vs separate `system` parameter)
- Required vs optional parameters (`max_tokens` required for Anthropic)

**FR-04: Request/Response Passthrough**  
For non-PII content, the system MUST act as a transparent proxy with minimal modification:
- Preserve all HTTP headers except authentication (replaced with customer's provider API key)
- Maintain original request body structure where possible
- Return identical HTTP status codes from upstream provider

### 4.2 PII Detection Engine (Parallel Dual-Tier Architecture)

**FR-05: Parallel Detection Strategy**  
The system MUST implement a two-tier detection pipeline that **always runs both tiers concurrently** on every request. Results are merged and deduplicated by span position, with conflicts resolved by highest confidence score.

**Rationale:** Co-occurrence of structured and unstructured PII is the norm. "Send the report to John Smith at john@acme.com" — regex catches the email, but skipping NER leaks "John Smith." The presence of structured PII is a positive signal that unstructured PII exists in the same text. Presidio's own internal architecture runs all recognizers unconditionally for this reason.

**Tier 1 — Fast Pattern Matching (Rust):**
- Use `aho-corasick` crate for O(n) multi-pattern matching across all patterns simultaneously
- Detect structured PII: Email, Phone, SSN, Credit Cards, API Keys (AWS, OpenAI, GitHub tokens)
- Target latency: <1ms per KB of text
- Coverage: 60-70% of common PII patterns
- **Optimization:** Use `cast()` over `extract()` in PyO3 for type conversion
- **Optimization:** Leverage PyO3 Bound API for zero-cost Python token access

**Tier 2 — ML-Based Detection (Python/Presidio):**
- Use Microsoft Presidio Analyzer for complex entities: Person Names, Locations, Organizations
- **Always invoked in parallel with Tier 1** — never conditionally skipped
- Target latency: <20ms per typical request (50-200 words)
- Coverage: Named entities requiring contextual understanding
- **Optimization:** Disable unused spaCy pipeline components (parser, tagger, lemmatizer) — load NER component only
- **Optimization:** Use `en_core_web_md` (43MB) instead of `en_core_web_lg` (560MB) for faster cold starts with comparable NER accuracy; evaluate `en_core_web_trf` only if F1 <85%

**Total detection latency:** `max(Tier1, Tier2)` ≈ 5-20ms, dominated by NER. This is well under the 50ms p99 target.

**FR-06: Result Merging & Deduplication**  
When both tiers return overlapping spans for the same text region:
- Prefer the result with the higher confidence score
- If confidence is equal, prefer the more specific entity type (e.g., EMAIL over PERSON)
- Discard fully contained subspans (e.g., if Tier 1 finds "john@acme.com" and Tier 2 finds "john", keep only the email)

**FR-07: Configurable Pattern Library**  
The system MUST support custom regex patterns defined in YAML configuration:

```yaml
custom_patterns:
  EMPLOYEE_ID: '\\bEMP-\\d{6}\\b'
  INTERNAL_IP: '\\b10\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\b'
  PROJECT_CODE: '\\b[A-Z]{3}-\\d{4}\\b'
```

**FR-08: Entity Confidence Scoring**  
Each detected entity MUST include a confidence score (0.0-1.0). The system MUST allow per-tenant configuration of minimum confidence thresholds (default: 0.75).

**FR-09: Custom Entity Recognizers**  
Enterprise customers MUST be able to upload custom spaCy models or regex patterns to detect domain-specific entities (e.g., internal employee IDs, proprietary product codes).

### 4.3 Surrogate Generation (Faker-Based Realistic Surrogates)

**FR-10: Faker-Based Surrogate Generation**  
Detected PII MUST be replaced with **realistic synthetic data** generated by the Faker library, preserving the entity type and linguistic naturalness:

| Entity Type | Original | Surrogate Example |
|-------------|----------|-------------------|
| PERSON | John Smith | Maria Lynch |
| EMAIL | john@acme.com | maria.lynch@example.net |
| PHONE | (555) 123-4567 | (555) 987-6543 |
| LOCATION | 123 Main St, Denver | 456 Oak Ave, Portland |
| SSN | 123-45-6789 | 987-65-4321 |
| CREDIT_CARD | 4111-1111-1111-1111 | 4532-8721-0039-6654 |

**Rationale:** Abstract placeholders like `{{PERSON_1}}` cause LLMs to paraphrase ("that person"), reformat, merge entities ("tell them"), or meta-comment on the placeholder format. Faker surrogates are treated as ordinary proper nouns by the LLM, dramatically improving rehydration accuracy. LangChain's `PresidioReversibleAnonymizer` validates this approach in production.

**Consistency Rule:** The same PII value within a single request MUST always map to the same Faker surrogate. A request-scoped mapping dictionary ensures "John Smith" always becomes "Maria Lynch" throughout the conversation turn.

**Collision Avoidance:** Faker surrogates MUST NOT collide with existing text in the request. If a generated surrogate already appears in the input text, regenerate with a different seed.

**FR-11: Encrypted Vault Storage**  
The mapping between Faker surrogates and original values MUST be stored in Redis with:
- **Client-side encryption** using Fernet (AES-128-CBC + HMAC-SHA256 with built-in TTL enforcement)
- **Cryptographic key derivation** from tenant secret + request ID
- **Hash data structure**: `pii:{tenant_id}:{request_id}` → `{surrogate_hash: encrypted_original}`
- **Strict TTL**: 600 seconds (10 minutes) for streaming requests, 120 seconds for synchronous
- **Fernet TTL enforcement**: `fernet.decrypt(token, ttl=600)` provides defense-in-depth — even if Redis fails to evict a key, Fernet refuses to decrypt expired data

**Rationale for Fernet over AES-256-GCM:** For ephemeral data with <10 minute TTLs, Fernet's impossible-to-misuse API eliminates the nonce-reuse risk inherent in AES-GCM (catastrophic at scale in multi-instance proxies). AES-128 remains NIST-approved and computationally infeasible to brute-force. Fernet also provides built-in `MultiFernet` for key rotation. Migration path to AES-256-GCM is documented for future compliance requirements.

**FR-12: Vault Availability Fail-Safe**  
If Redis is unreachable during rehydration, the system MUST:
1. Return HTTP 503 Service Unavailable to the client
2. Log the incident with request metadata (NO PII)
3. NEVER fail open by returning unrehydrated responses with Faker surrogates visible to the client

**FR-13: Cryptographic Erasure**  
After successful rehydration OR TTL expiration, the system MUST permanently delete:
- The Redis hash containing encrypted PII
- The ephemeral encryption key from memory (using Rust's `zeroize` crate)

### 4.4 Streaming & Rehydration

**FR-14: Server-Sent Events (SSE) Support**  
The system MUST support streaming responses via SSE for all providers, handling format differences:
- OpenAI: `data: {...}\n\ndata: [DONE]\n\n`
- Anthropic: `event: message_start\ndata: {...}\n\n` with `event: message_stop` termination
- Azure OpenAI: Identical to OpenAI format

**FR-15: Adaptive Sliding Window Buffer for Faker Rehydration**  
Since Faker surrogates are natural-language strings (not structured `{{tokens}}`), the system MUST implement a sliding window buffer with dictionary-based matching:

```python
class StreamingFakerRehydrator:
    """
    Uses aho-corasick automaton loaded with all active Faker surrogates
    for the current request. Sliding window catches names split across
    SSE chunk boundaries.
    """
    overlap_size: int  # max(len(surrogate) for surrogate in active_map) + margin
    buffer: str = ""
    automaton: AhoCorasick  # Built from request's Faker surrogate values
    
    def process_chunk(self, chunk: str) -> str:
        self.buffer += chunk
        # Run aho-corasick on full buffer
        # Emit confirmed portion (buffer minus overlap window)
        # Retain overlap for next chunk
```

**Key differences from v2.0's approach:**
- Overlap size is **dynamic**, calculated from the longest active Faker surrogate for the request (typically 15-30 chars for names)
- Matching uses `aho-corasick` multi-pattern search (Rust, O(n)) instead of regex
- No structured delimiter to trigger buffering — the sliding window is always active during rehydration

**FR-16: Flush Strategies**  
The buffer MUST flush data to the client when:
- Buffer size exceeds 2× overlap threshold
- Complete sentence detected (`. `, `? `, `! `)
- **Adaptive timeout**: `max(50ms, 3 × EWMA_inter_chunk_interval)` — adapts to model speed instead of fixed 500ms
- Stream termination signal (`[DONE]` or `message_stop`)

**FR-17: Multi-Strategy Deanonymization**  
For each Faker surrogate in the response, the system MUST attempt matching using progressively tolerant strategies:

1. **Exact match** — direct string replacement (handles ~80% of cases)
2. **Case-insensitive match** — catches LLM capitalization changes ("maria lynch" → "Maria Lynch")
3. **Fuzzy match** — Levenshtein distance ≤ 3 (catches minor typos/reformatting)
4. **Combined exact + fuzzy** — exact first, fuzzy fallback
5. **N-gram fuzzy match** — `fuzzywuzzy` with threshold 85 (catches partial name references like "Lynch" when full surrogate was "Maria Lynch")

**FR-18: Rehydration Metrics**  
The system MUST track per-request rehydration metrics:
- `surrogates_sent`: count of Faker surrogates in the sanitized request
- `surrogates_matched`: count successfully matched and replaced in the response
- `rehydration_success_rate`: `matched / sent` — alert if drops below 85%
- `match_strategy_used`: which deanonymization strategy succeeded (for tuning)
- `unmatched_surrogates`: log surrogate values that were expected but not found (for debugging)

**FR-19: System Prompt Injection for Surrogate Preservation**  
When forwarding sanitized requests to LLM providers, the system MUST prepend a system-level instruction:

```
All names, emails, locations, and other identifying information in this conversation 
are real and should be referenced exactly as written. Do not paraphrase, abbreviate, 
or replace any proper nouns.
```

This instruction improves the likelihood that the LLM preserves Faker names verbatim in its response. The instruction is configurable and can be disabled per-tenant.

**FR-20: Backpressure Handling**  
If the client consumer is slower than the upstream LLM provider, the system MUST:
- Buffer up to 1MB of data in memory
- Apply TCP backpressure to upstream connection
- Log slow consumer warnings for rate limiting consideration

**FR-21: Configurable Response-Side Rehydration**  
The system MUST support a per-request `X-Privacy-Gateway-Rehydrate: false` header that disables response-side rehydration entirely, passing the LLM response through with Faker surrogates intact. This eliminates all streaming buffer latency for use cases where the client handles deanonymization itself.

### 4.5 Multi-Tenancy & Access Control

**FR-22: API Key Authentication**  
Each tenant MUST authenticate using API keys with format: `pg_<random_6>_<random_32>` (e.g., `pg_abc123_xyz789...`). The system MUST:
- Store only SHA-256 hashes of full keys
- Support key rotation without service interruption
- Include key prefix in logs for debugging (first 8 chars only)

**FR-23: Row-Level Security (RLS)**  
Tenant data isolation MUST be enforced via PostgreSQL RLS policies:

```sql
CREATE POLICY tenant_isolation ON api_requests
  USING (tenant_id::TEXT = current_setting('app.current_tenant'));
```

Middleware MUST set `app.current_tenant` session variable on every database connection.

**FR-24: Usage Tracking & Metering**  
The system MUST track per-tenant metrics for billing:
- Total tokens processed (input + output)
- Request count by provider
- Total PII entities detected
- Rehydration success rate
- Storage duration in Redis vault

**FR-25: Rate Limiting**  
The system MUST enforce per-tenant rate limits using token bucket algorithm:
- Free tier: 100 requests/hour
- Pro tier: 10,000 requests/hour
- Enterprise tier: Custom limits

---

## 5. Non-Functional Requirements (NFRs)

### 5.1 Security

**NFR-01: Zero PII in Logs (Defense-in-Depth)**  
Application logs, error messages, and telemetry MUST NEVER contain raw PII values, surrogate token mappings, or encryption keys.

**Implementation:** Four-layer defense-in-depth architecture:

- **Layer 1 — Code discipline:** Never pass PII to logger calls. Use opaque request IDs. Implement `__repr__` on all data classes to exclude PII fields.
- **Layer 2 — structlog processor chain:** All application logging via structlog with a `scrub_pii` processor that applies regex patterns (emails, SSNs, credit cards, phone numbers, IPs, JWTs, API keys) and allowlist key redaction (email, phone, ssn, password, name, address, request_body, response_body).
- **Layer 3 — `ProcessorFormatter` with `foreign_pre_chain`:** All stdlib logging handlers use structlog's `ProcessorFormatter`, which routes third-party library output (httpx, SQLAlchemy, Presidio, spaCy) through the same `scrub_pii` processor via `foreign_pre_chain`.
- **Layer 4 — Third-party logger suppression:** httpx and httpcore set to WARNING, SQLAlchemy set to WARNING with `propagate=False`, Presidio analyzers set to ERROR, spaCy set to WARNING.

```python
import structlog, logging

def scrub_pii(logger, method_name, event_dict):
    """Regex + key allowlist scrubbing on all log output."""
    # Apply PII regex patterns to string values
    # Redact known-sensitive keys entirely
    return event_dict

formatter = structlog.stdlib.ProcessorFormatter(
    processors=[scrub_pii, structlog.processors.JSONRenderer()],
    foreign_pre_chain=[scrub_pii],  # Catches httpx, SQLAlchemy, Presidio, spaCy
)
handler = logging.StreamHandler()
handler.setFormatter(formatter)
logging.getLogger().handlers = [handler]
```

Logs MAY contain: Entity types, counts, confidence scores, request IDs, tenant IDs (prefix only).

**NFR-02: Encryption Standards**  
- **In Transit:** TLS 1.2+ for all connections (client ↔ proxy ↔ LLM provider ↔ Redis)
- **At Rest:** Fernet (AES-128-CBC + HMAC-SHA256) for PII in Redis vault, with built-in TTL enforcement
- **In Memory:** Use Rust's `zeroize` crate to clear sensitive data from memory after use
- **Migration Path:** Document AES-256-GCM migration for compliance requirements (256-bit keys, AAD support, cross-language interop)

**NFR-03: Authentication Security**  
- API keys MUST use cryptographically secure random generation (`secrets.token_urlsafe`)
- Hash storage MUST use SHA-256 with per-tenant salt
- Support for future OAuth2/OIDC integration (Enterprise tier)

**NFR-04: Vulnerability Prevention**  
The system MUST implement protections against:
- **HTTP Request Smuggling:** Reject requests with both `Content-Length` and `Transfer-Encoding`
- **SSRF:** Allowlist upstream LLM provider domains, block internal IP ranges (10.x, 192.168.x, 127.x)
- **Header Injection:** Sanitize all input, reject carriage return/line feed characters
- **Memory Exhaustion:** Set strict request body limits (10MB max), enforce streaming for large payloads

**NFR-05: Audit Logging**  
The system MUST maintain immutable audit logs containing:
- Timestamp, tenant ID, request ID
- Detected entity types and counts (never values)
- Provider selection and response status
- Rehydration success rate per request
- Authentication events (key usage, failures)

**NFR-06: Automated Security Testing**  
The system MUST integrate automated security tooling:
- **garak:** Vulnerability scanner for prompt injection, jailbreaks (CI/CD integration)
- **promptfoo:** Local evaluation framework for security regression testing

### 5.2 Performance

**NFR-07: Async Runtime Optimization**  
The application MUST use `uvloop` instead of standard Python `asyncio` to achieve 2-4× performance improvement for I/O-bound operations.

**NFR-08: GIL Release for CPU-Bound Operations**  
Rust modules MUST use PyO3's `py.allow_threads()` to release the Global Interpreter Lock during:
- Regex pattern matching (`aho-corasick`)
- String manipulation and token replacement
- `aho-corasick` streaming search for Faker surrogate matching in response rehydration

**NFR-09: PyO3 Bridge Optimization**  
The Python-Rust bridge MUST implement micro-optimizations:
- Use `cast()` method over `extract()` for type conversion (avoids polymorphic overhead)
- Leverage Bound API for zero-cost Python token access
- Target: <5ms p99 latency for bridge operations

**NFR-10: Connection Pooling**  
The system MUST maintain persistent connection pools:
- **HTTP Client:** 100 max connections, 20 keepalive connections per provider
- **Redis:** 50 connections per instance
- **PostgreSQL:** 20 connections via SQLAlchemy async pool

**NFR-11: Memory Efficiency**  
- Maximum memory per instance: 2GB
- Streaming chunks: 4KB default size
- Buffer limits: 1MB per concurrent request
- **spaCy model:** Use `en_core_web_md` (43MB) over `en_core_web_lg` (560MB) to reduce Docker image size and cold start time

### 5.3 Reliability

**NFR-12: Graceful Degradation**  
Component failure modes:
- **Redis unavailable:** Return 503, block all traffic (fail-secure)
- **Presidio ML unavailable:** Fall back to Tier 1 regex-only detection with warning header `X-Privacy-Gateway-Degraded: tier2-unavailable`
- **Upstream LLM timeout:** Return timeout to client after 60 seconds

**NFR-13: Health Checks**  
The system MUST expose `/health` and `/ready` endpoints:
- `/health`: Liveness probe (HTTP 200 if process alive)
- `/ready`: Readiness probe (HTTP 200 if Redis + DB + Presidio reachable)

**NFR-14: Circuit Breaker**  
Implement circuit breaker pattern for upstream LLM providers:
- Open circuit after 5 consecutive failures
- Half-open after 30-second cooldown
- Close after 3 successful requests

### 5.4 Observability

**NFR-15: Distributed Tracing with Langfuse**  
The system MUST implement OpenTelemetry-based distributed tracing:
- Trace every request through: Ingress → PII Detection (Rust + Presidio parallel) → Vaulting (Redis) → Proxy → Rehydration
- Nested spans for cross-language boundaries (Python ↔ Rust)
- Self-hosted Langfuse instance (open-source, MIT license)
- No sensitive data in traces (entity types/counts only)
- **Rehydration metrics** tracked per trace: success rate, match strategy distribution

**NFR-16: Metrics & Monitoring**  
The system MUST expose Prometheus-compatible metrics:
- Request count, latency (p50, p95, p99)
- PII detection rate, entity type distribution
- Redis vault hit/miss ratio
- Upstream provider latency and error rates
- Rust bridge overhead (p50, p95, p99)
- **Rehydration success rate** (rolling average, per-tenant)
- **Match strategy distribution** (exact vs fuzzy vs n-gram)

---

## 6. Technical Architecture

### 6.1 Technology Stack

| Layer | Technology | Version | Rationale |
|-------|-----------|---------|-----------|
| **API Framework** | FastAPI | ≥0.115.0 | Best-in-class async support, automatic OpenAPI docs |
| **Runtime** | Python | 3.11+ | Stable async features, pattern matching |
| **Event Loop** | uvloop | ≥0.19.0 | 2-4× performance vs standard asyncio |
| **HTTP Client** | httpx | ≥0.27.0 | Async, HTTP/2, clean API |
| **SSE Library** | sse-starlette | ≥3.2.0 | W3C compliant, FastAPI native |
| **PII Detection** | Presidio | ≥2.2.0 | Industry standard, extensible |
| **NLP Engine** | spaCy (`en_core_web_md`) | ≥3.7.0 | NER-only pipeline, 43MB model |
| **Surrogate Generation** | Faker | ≥24.0.0 | Locale-aware realistic synthetic data |
| **Fuzzy Matching** | fuzzywuzzy + python-Levenshtein | Latest | Multi-strategy deanonymization |
| **Performance Core** | Rust + PyO3 | 0.27+ | 3-50× speedup for regex/string ops |
| **Streaming Match** | aho-corasick (Rust) | 1.1+ | O(n) multi-pattern streaming search |
| **Build Tool** | Maturin | ≥1.0.0 | Zero-config Python wheel building |
| **Cache/Vault** | Redis | ≥7.0 | Sub-millisecond latency, TTL support |
| **Encryption** | Fernet (cryptography lib) | ≥42.0.0 | Built-in TTL, impossible-to-misuse API |
| **Database** | PostgreSQL | ≥15 | Row-level security, JSON support |
| **ORM** | SQLAlchemy | ≥2.0.0 | Async support, type safety |
| **Migrations** | Alembic | ≥1.13.0 | Database version control |
| **Logging** | structlog | ≥24.0.0 | Processor chain, `foreign_pre_chain` for third-party capture |
| **Observability** | Langfuse | Latest | Open-source tracing, self-hosted |
| **Security Testing** | garak | Latest | Automated vulnerability scanning |
| **Prompt Testing** | promptfoo | Latest | Local evaluation framework |
| **Container** | Docker | Latest | Reproducible builds |
| **Orchestration** | AWS ECS Fargate | - | Managed serverless containers |
| **Local AWS Emulation** | LocalStack | ≥4.13.0 | Free local AWS emulation for development (S3, KMS, etc.) |

**Development vs Production AWS Configuration:**  
All AWS service clients (S3, KMS, SQS, etc.) MUST use a configurable endpoint URL so the same codebase works against both LocalStack (development) and real AWS (production). Configuration via environment variable:

```python
# AWS_ENDPOINT_URL=http://localhost:4566  → LocalStack (dev)
# AWS_ENDPOINT_URL=None/unset             → Real AWS (production)
endpoint_url = os.getenv("AWS_ENDPOINT_URL", None)
s3 = boto3.client("s3", endpoint_url=endpoint_url)
```

### 6.2 Rust Crates (via PyO3)

```toml
[dependencies]
pyo3 = { version = "0.27", features = ["extension-module"] }
aho-corasick = "1.1"       # Multi-pattern string search (detection + rehydration)
regex = "1.10"              # Fallback for complex patterns
bstr = "1.9"                # Fast byte string operations
rayon = "1.10"              # Data parallelism
zeroize = "1.7"             # Secure memory clearing
```

### 6.3 System Architecture Diagram

```
┌─────────────┐
│   Client    │
│ Application │
└──────┬──────┘
       │ HTTPS (TLS)
       ▼
┌─────────────────────────────────────────────────────────┐
│              AI Privacy Gateway (FastAPI)               │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────────────────────────┐ │
│  │   Ingress    │→ │ PII Detection (PARALLEL)         │ │
│  │  Middleware  │  │ ┌──────────┐  ┌───────────────┐  │ │
│  │  (Auth,Rate) │  │ │Tier 1    │  │Tier 2         │  │ │
│  │             │  │ │Rust Regex │  │Presidio NER   │  │ │
│  │             │  │ │<1ms      │  │5-20ms         │  │ │
│  │             │  │ └──────────┘  └───────────────┘  │ │
│  │             │  │        ↓ merge + dedup ↓          │ │
│  └──────────────┘  └──────────────────────────────────┘ │
│         │                         │                      │
│         │              ┌──────────▼──────────┐           │
│         │              │  Faker Surrogate    │           │
│         │              │  Generator          │           │
│         │              └──────────┬──────────┘           │
│         │              ┌──────────▼──────────┐           │
│         │              │  Fernet Vault       │           │
│         │              │  (Redis + TTL)      │           │
│         │              └──────────┬──────────┘           │
│         │              ┌──────────▼──────────┐           │
│         │              │  Provider Router    │           │
│         │              │  + System Prompt    │           │
│         │              └──────────┬──────────┘           │
│         │              ┌──────────▼──────────┐           │
│         │              │  Streaming          │           │
│         │              │  Rehydrator         │           │
│         │              │  (aho-corasick +    │           │
│         │              │   adaptive window)  │           │
│         │              └──────────┬──────────┘           │
└─────────────────────────┼────────────────────────────────┘
                          │ HTTPS
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
   ┌────────────┐  ┌────────────┐  ┌────────────┐
   │   OpenAI   │  │ Anthropic  │  │Azure OpenAI│
   └────────────┘  └────────────┘  └────────────┘
```

### 6.4 Data Flow: Request Pipeline

**Phase 1: Ingestion**
1. Client sends `POST /v1/chat/completions` with PII-containing prompt
2. API key middleware validates tenant, sets `request.state.tenant_id`
3. Request body parsed and validated via Pydantic
4. **Langfuse creates trace span:** `request_ingestion`

**Phase 2: Parallel Detection**
5. **Tier 1 (Rust) and Tier 2 (Presidio) execute concurrently** via `asyncio.gather`:
   - Tier 1: `rust_scanner.detect_patterns(text)` → emails, SSNs in <1ms
   - Tier 2: `presidio.analyze(text, entities=["PERSON","LOCATION","ORG"])` → names, locations in 5-20ms
   - **Langfuse spans:** `rust_pattern_matching` + `presidio_ml_detection` (parallel)
6. **Merge results:** Deduplicate overlapping spans, prefer highest confidence

**Phase 3: Faker Surrogate Generation & Vaulting**
7. For each detected entity:
   - Generate locale-appropriate Faker surrogate (consistent per request)
   - Verify no collision with existing text
   - Encrypt original value with `Fernet(ephemeral_key)`
   - Store in Redis: `HSET pii:{tenant}:{req_id} {surrogate_hash} {encrypted_original}`
   - **Langfuse span:** `faker_generation_and_vaulting`
   - Replace PII with Faker surrogate in request body

**Phase 4: Upstream Proxy**
8. Prepend system prompt instruction for surrogate preservation
9. Transform request to provider format via `ProviderAdapter`
10. Send sanitized request to LLM provider
    - **Langfuse span:** `upstream_llm_call`
11. Receive streaming SSE response

**Phase 5: Streaming Rehydration**
12. Build `aho-corasick` automaton from request's active Faker surrogates
13. Buffer incoming chunks via `StreamingFakerRehydrator` with adaptive sliding window
    - **Langfuse span:** `stream_rehydration`
14. Multi-strategy deanonymization (exact → case-insensitive → fuzzy → n-gram)
15. Replace Faker surrogates with original PII, flush to client
16. On stream end:
    - Record rehydration metrics (success rate, match strategies used)
    - Delete Redis hash
    - Zero encryption key in memory (Rust `zeroize`)
    - **Langfuse finalizes trace** with total latency, cost, token count, rehydration stats

### 6.5 Database Schema (Core Tables)

**Tenants**
```sql
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    tier VARCHAR(20) NOT NULL CHECK (tier IN ('free', 'pro', 'enterprise')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    settings JSONB DEFAULT '{}'::jsonb
);
```

**API Keys**
```sql
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    key_hash VARCHAR(64) NOT NULL UNIQUE,
    key_prefix VARCHAR(16) NOT NULL,
    name VARCHAR(255),
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);
CREATE INDEX idx_api_keys_hash ON api_keys(key_hash);
```

**Usage Metrics**
```sql
CREATE TABLE usage_logs (
    id BIGSERIAL PRIMARY KEY,
    tenant_id UUID REFERENCES tenants(id),
    request_id UUID NOT NULL,
    provider VARCHAR(50),
    input_tokens INTEGER,
    output_tokens INTEGER,
    entities_detected INTEGER,
    rehydration_success_rate DECIMAL(5,4),
    latency_ms INTEGER,
    rust_bridge_latency_ms INTEGER,
    langfuse_trace_id VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW()
) PARTITION BY RANGE (created_at);

ALTER TABLE usage_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON usage_logs
  USING (tenant_id::TEXT = current_setting('app.current_tenant', true));
```

---

## 7. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
**Goal:** Working non-streaming reverse proxy with authentication and observability

**Tasks:**
- [ ] Set up FastAPI project with uvloop configuration
- [ ] Implement `/v1/chat/completions` endpoint (non-streaming)
- [ ] Create `ProviderAdapter` interface and OpenAI implementation
- [ ] Implement Anthropic adapter with message transformation
- [ ] PostgreSQL database setup with Alembic migrations
- [ ] Redis connection with health check endpoint
- [ ] Basic API key authentication middleware with SHA-256 hashing
- [ ] Row-level security (RLS) policies and middleware
- [ ] Docker containerization with multi-stage build
- [ ] Deploy self-hosted Langfuse for distributed tracing
- [ ] Set up structlog with `ProcessorFormatter` and `foreign_pre_chain` PII scrubbing
- [ ] Suppress verbose third-party loggers (httpx, SQLAlchemy)

**Deliverable:** Proxy that authenticates tenants, forwards requests to OpenAI/Anthropic, returns responses with full tracing and PII-safe logging

**Success Criteria:** Can authenticate, route requests, return responses with <10ms overhead, zero PII in any log output, visible traces in Langfuse

---

### Phase 2: Core Privacy Logic (Weeks 3-4)
**Goal:** Parallel PII detection, Faker surrogates, streaming rehydration

**Tasks:**
- [ ] Integrate Microsoft Presidio analyzer with NER-only spaCy pipeline (`en_core_web_md`)
- [ ] Implement parallel dual-tier detection (`asyncio.gather` for Rust + Presidio)
- [ ] Build result merging and deduplication logic
- [ ] Implement Faker-based surrogate generation with collision avoidance
- [ ] Build encrypted Redis vault with Fernet and built-in TTL enforcement
- [ ] Create `StreamingFakerRehydrator` with adaptive sliding window buffer
- [ ] Implement multi-strategy deanonymization (exact → case-insensitive → fuzzy → n-gram)
- [ ] Build rehydration metrics tracking (success rate, match strategy distribution)
- [ ] Implement system prompt injection for surrogate preservation
- [ ] Implement SSE response handling with `sse-starlette`
- [ ] Add `X-Privacy-Gateway-Rehydrate: false` header support
- [ ] Comprehensive unit tests for edge cases (split names, overlapping entities, collision avoidance)
- [ ] Integrate garak for initial security scanning

**Deliverable:** End-to-end privacy protection with streaming, Faker surrogates, and rehydration metrics

**Success Criteria:** Can detect PII (both tiers in parallel), replace with Faker surrogates, vault securely, stream sanitized responses, rehydrate with >85% success rate, pass garak tests

---

### Phase 3: Rust Performance Core (Weeks 5-6)
**Goal:** Optimize latency-critical paths with Rust, harden for production

**Tasks:**
- [ ] Initialize Maturin project structure (`maturin init`)
- [ ] Implement Rust pattern matcher using `aho-corasick` for Tier 1 detection
- [ ] Implement Rust `aho-corasick` streaming matcher for Faker surrogate rehydration
- [ ] Port token replacement logic to Rust with `bstr` for fast string ops
- [ ] Add PyO3 bindings with GIL release (`py.allow_threads()`)
- [ ] Implement PyO3 micro-optimizations: `cast()` over `extract()`, Bound API
- [ ] Build manylinux wheels for production deployment
- [ ] Benchmark Rust vs Python implementations, verify <5ms p99 bridge latency
- [ ] Implement rate limiting with token bucket algorithm
- [ ] Implement usage tracking and metering system
- [ ] Set up Prometheus metrics endpoint
- [ ] Implement circuit breaker for upstream providers
- [ ] Deploy to AWS ECS Fargate with auto-scaling
- [ ] Configure CloudWatch alarms
- [ ] Run comprehensive garak security suite
- [ ] Write README with quick-start guide (<5 minutes to first masked request)
- [ ] Docker Compose for self-hosted deployment

**Deliverable:** Production-ready privacy proxy with Rust performance, monitoring, and deployment

**Success Criteria:** <50ms p99 detection latency, <5ms p99 Rust bridge overhead, 500+ RPS per instance, >85% rehydration success rate, complete deployment pipeline, README published

---

### Phase 2+ Roadmap (Post-Validation)

The following features are explicitly deferred from MVP. They will be prioritized based on user feedback and demand signals from beta customers.

#### Phase 4: Developer Experience (Weeks 7-10)
**Trigger:** 10+ active tenants or 3+ requests for SDK/dashboard

- [ ] Generate OpenAPI 3.1 specification with LLM-optimized descriptions
- [ ] Auto-generate Python SDK (Speakeasy or Stainless) → publish to PyPI
- [ ] Auto-generate TypeScript SDK → publish to npm
- [ ] Build admin dashboard for tenant management (usage, key management, configuration)
- [ ] Stripe billing integration (Free/Pro/Enterprise tiers)
- [ ] Customer onboarding flow

#### Phase 5: Ecosystem Integration (Weeks 11-14)
**Trigger:** Demand from AI agent builders or 5+ enterprise inquiries

- [ ] MCP server exposing gateway capabilities
- [ ] OAuth2 authentication for AI-to-AI communication
- [ ] MCP Gateway for centralized orchestration (if >100 agent clients)
- [ ] Rate limiting per MCP client
- [ ] Tool descriptions optimized for Claude/GPT

#### Phase 6: Advanced Privacy (Weeks 15+)
**Trigger:** F1 score plateaus below 90% or customer demand for specific entity types

- [ ] Fine-tuned NER models for domain-specific entities
- [ ] Evaluate LLM-based fine-tuning for surrogate preservation (Protecto's approach)
- [ ] Custom Faker providers for industry-specific synthetic data
- [ ] HIPAA compliance and healthcare entity types
- [ ] On-premise deployment option (Docker Compose + Helm chart)
- [ ] AES-256-GCM migration (if compliance requires 256-bit keys)

#### Phase 7: Scale & Polish
- [ ] Landing page with demo video
- [ ] Grafana dashboards
- [ ] Azure OpenAI adapter
- [ ] Provider selection routing (cost optimization, latency-based)
- [ ] Multi-region deployment
- [ ] SOC 2 compliance preparation

---

## 8. Risks & Mitigation Strategies

| Risk | Impact | Probability | Mitigation Strategy |
|------|--------|-------------|---------------------|
| **LLM Paraphrases Faker Surrogates** | High | Medium | Multi-strategy deanonymization (5 levels), system prompt injection, rehydration metrics alerting, configurable opt-out of response-side rehydration |
| **Faker Collisions with Real Text** | Medium | Low | Collision detection before replacement, regeneration with different seed, request-scoped consistency mapping |
| **Parallel Detection Latency** | Medium | Low | spaCy NER-only pipeline (5-20ms), `en_core_web_md` model, async parallel execution — total is `max(T1, T2)` not `T1 + T2` |
| **Fernet → AES-256-GCM Migration** | Low | Low | Clean encryption abstraction layer, documented migration path, only triggered by explicit compliance requirement |
| **PII Leakage in Third-Party Logs** | High | Medium | Four-layer defense-in-depth logging architecture, structlog `foreign_pre_chain`, PII canary testing |
| **Streaming Buffer Adds Latency** | Medium | Medium | Adaptive timeout (`3 × EWMA_ITL`), configurable rehydration opt-out, sentence-boundary flushing |
| **Redis Single Point of Failure** | Critical | Low | Fernet TTL as defense-in-depth, Redis Cluster with replication, fail-secure (503 on unavailability) |
| **Scope Creep Delays MVP** | High | High | Ruthlessly enforced 6-week MVP scope, deferred features gated on user demand signals, explicit phase triggers |
| **Latency Budget Exceeded** | High | Medium | Offload CPU-heavy operations to Rust, use uvloop, benchmark continuously via Langfuse, enforce <5ms bridge target |
| **False Positives (Over-detection)** | Medium | Medium | Tune confidence thresholds per entity type, allow customer override, implement feedback loop via Langfuse annotations |
| **Deployment Complexity (Hybrid Binary)** | Medium | Medium | Maturin for automated wheel building, manylinux2014 compatibility, Docker images provided |

---

## 9. Success Criteria & Launch Readiness

### MVP Launch Criteria (End of Week 6)

**Technical Readiness:**
- ✅ <50ms p99 latency overhead (parallel dual-tier detection)
- ✅ <5ms p99 Rust bridge latency (tracked via Langfuse)
- ✅ 500+ RPS per instance sustained
- ✅ >85% F1 score on PII detection benchmark
- ✅ >85% rehydration success rate (Faker surrogates matched in LLM responses)
- ✅ Zero PII in logs validated via PII canary testing
- ✅ 99.5% uptime over 1-week test period
- ✅ 0 critical vulnerabilities from garak scanning

**Product Readiness:**
- ✅ OpenAI-compatible API with `base_url` swap (zero client code changes)
- ✅ Quick-start README with cURL examples (<5 minutes to first masked request)
- ✅ Docker Compose for self-hosted deployment
- ✅ Configuration via YAML / environment variables
- ✅ API key management via CLI
- ✅ Usage metrics visible via Langfuse traces

**Business Readiness:**
- ✅ Published on GitHub (open-source core)
- ✅ "Show HN" post prepared
- ✅ 3-5 beta users identified for feedback
- ✅ Support channel established (GitHub Issues + Discord)

### Post-Launch Metrics (First 3 Months)

- **Adoption:** 50 GitHub stars, 10 active self-hosted deployments, 3 beta SaaS tenants
- **Performance:** Maintain <50ms p99 latency at scale
- **Rehydration:** >85% average success rate across all tenants
- **Reliability:** 99.9% uptime, <5 critical incidents
- **Security:** 0 PII leakage incidents
- **Feedback Signal:** 3+ requests for SDKs/dashboard → trigger Phase 4

---

## 10. Open Questions & Decisions Needed

**Technical Decisions:**
1. **Faker Locale Strategy:** Default to `en_US`, or infer locale from detected PII language? (Start simple, expand later)
2. **Fuzzy Match Threshold Tuning:** Levenshtein max_l_dist=3 and fuzzywuzzy threshold=85 are defaults from LangChain — need to validate against real LLM output patterns
3. **System Prompt Injection Opt-Out:** Some tenants may not want system prompt modification — make configurable from day 1
4. **Redis Cluster Sizing:** Single instance for MVP (<100K req/day), cluster at 1M+ req/day
5. **spaCy Model Selection:** Start with `en_core_web_md` (43MB), upgrade to `_lg` (560MB) only if F1 < 85%

**Product Decisions:**
1. **Open-Source Scope:** Core proxy fully open-source? Or open-core with SaaS features proprietary?
2. **Free Tier Limits:** 10K tokens/month or 100 requests/hour?
3. **Provider Priority:** OpenAI + Anthropic for MVP, Azure based on demand
4. **Self-Hosted vs SaaS First:** GitHub + Docker Compose first (build community), SaaS tier after validation

**Business Decisions:**
1. **Pricing Model:** Usage-based ($/1K tokens) vs seat-based ($/user/month)?
2. **Target ICP:** AI startups (high volume, low ACV) vs enterprises (low volume, high ACV)?
3. **Go-to-Market:** Show HN → GitHub community → Enterprise outreach?

---

## 11. Appendices

### A. Competitive Landscape

| Solution | Type | Strengths | Weaknesses | Pricing |
|----------|------|-----------|------------|---------|
| **Private AI** | Commercial SaaS | High accuracy (92% F1), healthcare focus | Expensive ($500K+ quotes), batch-only | Enterprise |
| **Protecto** | Commercial SaaS | GDPR/HIPAA certified, fine-tuned model recognition | No streaming, slow (200ms+ latency) | Enterprise |
| **LiteLLM** | Open Source | 100+ providers, great observability | Zero PII detection | Free/Pro $500 |
| **Portkey** | Commercial Gateway | Multi-provider, cost tracking | No native PII detection | $99-$999 |
| **Presidio** | Open Source Library | Extensible, well-maintained | Not a gateway, needs integration | Free |
| **LangChain Anonymizer** | Open Source Library | Faker surrogates, multi-strategy matching | Not a gateway, single-request only | Free |
| **Rehydra** | Open Source SDK | XML-tag fuzzy matching, lightweight | Client-side only, no streaming | Free |
| **AI Privacy Gateway (Ours)** | Open Source SaaS | Streaming PII detection, Faker surrogates, Rust performance, parallel detection | New entrant | Free/Pro/Enterprise |

**Our Differentiation:**
1. Only open-source solution combining LiteLLM-style routing with Presidio-level detection
2. Faker-based surrogates for superior rehydration accuracy (LangChain approach, gateway form factor)
3. Parallel dual-tier detection eliminating the conditional logic gap in competing architectures
4. Optimized for streaming workloads with Rust `aho-corasick` rehydration (<50ms p99)

### B. Surrogate Token Strategy Comparison

| Strategy | Faithfulness | LLM Output Quality | Implementation Complexity | Used By |
|----------|-------------|-------------------|--------------------------|---------|
| **Faker surrogates + fuzzy matching ✅** | Good | Highest — natural language preserved | Medium | LangChain, AI Privacy Gateway |
| Fine-tuned model recognition | Best | Medium — requires training data | High — per-model fine-tuning | Protecto |
| XML-tag fuzzy matching | Medium | Medium — tags can confuse reasoning | Medium | Rehydra |
| Abstract placeholders `{{TYPE_N}}` | Poor | Lowest — paraphrasing, meta-comments | Low | Most academic approaches |

### C. AES-256-GCM Migration Guide

If compliance requirements mandate 256-bit encryption keys, migrate from Fernet to AES-256-GCM:

1. Add `VaultEncryptor` abstraction interface with `encrypt(plaintext, ttl)` and `decrypt(ciphertext, ttl)` methods
2. Implement `FernetEncryptor` (current) and `AesGcmEncryptor` as concrete classes
3. AES-GCM implementation must: generate random 12-byte nonce per encryption, prepend nonce to ciphertext, implement manual TTL checking via embedded timestamp, use `os.urandom(12)` for nonce generation (never sequential)
4. Key considerations: nonce reuse is catastrophic in GCM — verify randomness under load testing with multiple instances. Storage efficiency improves ~60% (38 bytes vs ~100 bytes per encrypted value).

### D. Reference Architecture

**GitHub Projects:**
- [LangChain PresidioReversibleAnonymizer](https://github.com/langchain-ai/langchain/tree/master/libs/experimental) — Faker surrogate + multi-strategy matching reference
- [PyO3/maturin](https://github.com/PyO3/maturin) — Build tooling
- [microsoft/presidio](https://github.com/microsoft/presidio) — PII detection
- [sysid/sse-starlette](https://github.com/sysid/sse-starlette) — SSE streaming
- [rehydra-ai/rehydra-sdk](https://github.com/rehydra-ai/rehydra-sdk) — XML-tag fuzzy matching reference
- [BurntSushi/aho-corasick](https://github.com/BurntSushi/aho-corasick) — Streaming multi-pattern search

**Benchmark Datasets:**
- [presidio-research](https://github.com/microsoft/presidio-research) — PII detection test sets
- [CredData](https://github.com/duo-labs/creddata) — Credentials and API key patterns

### E. Glossary

**Core Concepts:**
- **PII (Personally Identifiable Information):** Data that can identify an individual (name, email, SSN, etc.)
- **Faker Surrogate:** Realistic synthetic data (e.g., "Maria Lynch") replacing detected PII — LLMs treat as ordinary text
- **Vaulting:** Secure storage of PII-to-surrogate mapping with Fernet encryption and TTL
- **Rehydration:** Replacing Faker surrogates in LLM responses with original PII values
- **Deanonymization Strategy:** Progressive matching techniques (exact → fuzzy → n-gram) for surrogate recovery
- **SSE (Server-Sent Events):** HTTP streaming standard for real-time data push
- **RLS (Row-Level Security):** PostgreSQL feature for multi-tenant data isolation
- **PyO3:** Rust bindings for Python, enabling hybrid architectures
- **aho-corasick:** Multi-pattern string search algorithm with O(n) time complexity, used for both detection and streaming rehydration
- **Fernet:** Symmetric encryption scheme (AES-128-CBC + HMAC-SHA256) with built-in TTL enforcement and key rotation

---

## Document Control

**Version History:**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | Feb 2, 2026 | Benjamin Silva | Initial MVP draft |
| 1.1 | Feb 2, 2026 | Benjamin Silva | Added NFRs, expanded architecture, detailed roadmap |
| 2.0 | Feb 2, 2026 | Benjamin Silva | Added MCP ecosystem, Claude Code Skills, Langfuse observability, SDK generation, AI tooling guide, PyO3 optimizations, security automation |
| **3.0** | **Feb 6, 2026** | **Benjamin Silva** | **Architectural revision based on technical review: Faker surrogates replacing abstract placeholders, parallel dual-tier detection, Fernet encryption with AES-256-GCM migration path, adaptive sliding window with aho-corasick rehydration, structlog defense-in-depth logging, 6-week focused MVP scope with demand-gated future phases, rehydration success metrics** |

**Review & Approval:**

- [ ] Technical Review: _______________ (Date: _______)
- [ ] Product Review: _______________ (Date: _______)
- [ ] Security Review: _______________ (Date: _______)

**Next Review Date:** March 1, 2026 (post-Phase 1 completion)

**Key Changes in v3.0:**
- Replaced abstract `{{TYPE_N}}` placeholders with Faker-based realistic surrogates
- Changed PII detection from conditional (Tier 2 only if Tier 1 empty) to always-parallel execution
- Resolved encryption inconsistency: specified Fernet with documented AES-256-GCM migration path
- Replaced fixed 500ms streaming timeout with adaptive `3 × EWMA_ITL` approach
- Added multi-strategy deanonymization (5 levels: exact → case-insensitive → Levenshtein → combined → n-gram)
- Added rehydration success rate as a first-class KPI with alerting
- Implemented defense-in-depth PII-safe logging via structlog `ProcessorFormatter` with `foreign_pre_chain`
- Added system prompt injection for surrogate preservation (configurable)
- Added configurable response-side rehydration opt-out (`X-Privacy-Gateway-Rehydrate: false`)
- Reduced MVP scope from 8 weeks to 6 weeks; cut MCP server, SDK generation, admin dashboard, Stripe, landing page
- Added explicit phase triggers for deferred features (gated on user demand signals)
- Used `en_core_web_md` (43MB) instead of `en_core_web_lg` (560MB) for faster cold starts
- Removed AI Development Tooling Guide section (moved to separate internal document)
