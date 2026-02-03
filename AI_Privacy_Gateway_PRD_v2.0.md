# Product Requirements Document: AI Privacy Gateway

**Project Name:** AI Privacy Gateway (B2B SaaS)  
**Author:** Benjamin Silva  
**Version:** 2.0 (Enhanced with AI-Native Development Architecture)  
**Date:** February 2, 2026  
**Status:** Planning Phase  

---

## 1. Executive Summary

The AI Privacy Gateway is a high-performance reverse proxy that solves the "black box" privacy challenge in enterprise Generative AI adoption. By intercepting API requests to LLM providers (OpenAI, Anthropic, Azure OpenAI), detecting personally identifiable information (PII) in real-time, and replacing it with reversible surrogate tokens, the Gateway enables enterprises to leverage AI capabilities without exposing sensitive data to third-party model training or security breaches.

**Core Value Proposition:** Zero-trust privacy architecture that decouples LLM utility from data liability through real-time PII detection, vaulting, and rehydration—all while maintaining transparent API compatibility.

**Target Market:** Technology companies, AI startups, and SaaS platforms requiring GDPR/privacy compliance for LLM integration (initial focus: non-healthcare enterprises).

**Key Innovation:** This PRD leverages cutting-edge AI-native development practices including Model Context Protocol (MCP) integration, Claude Code Skills for domain expertise encoding, and automated SDK generation to deliver a production-ready system in 8 weeks.

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

- **Regulatory Pressure:** EU AI Act enforcement begins 2025, requiring documented PII safeguards
- **Enterprise Adoption Blockers:** 67% of enterprises cite data privacy as top barrier to LLM adoption (Gartner 2024)
- **Technical Feasibility:** Hybrid Python/Rust architectures (PyO3) now enable real-time performance at acceptable development cost
- **AI Development Maturity:** MCP and agentic coding tools enable 2-3× faster development cycles

---

## 3. Goals & Success Metrics

### 3.1 Primary Goals

**PG-1: Zero-Trust Privacy Architecture**  
Implement a "round-trip" architecture where sensitive data never leaves enterprise control in its original form. All PII is detected, vaulted with encryption, replaced with surrogates, and rehydrated post-inference.

**PG-2: Transparent Developer Experience**  
Maintain 100% API compatibility with OpenAI's format so existing client applications can switch the `base_url` parameter and immediately benefit from privacy protection without code changes.

**PG-3: Production-Grade Performance**  
Minimize latency overhead to ensure the privacy layer doesn't degrade user experience. Target sub-50ms p99 detection latency versus vanilla Presidio's 50-150ms baseline.

**PG-4: Multi-Provider Flexibility**  
Abstract provider-specific APIs (OpenAI, Anthropic, Azure OpenAI) behind a unified interface, enabling customers to switch providers without reconfiguration.

**PG-5: AI-Native Ecosystem Integration (NEW)**  
Expose gateway capabilities as MCP server to enable AI-to-AI integration, and provide auto-generated SDKs for seamless developer adoption.

### 3.2 Key Performance Indicators (KPIs)

| Metric | MVP Target | Production Target | Measurement Method |
|--------|-----------|------------------|-------------------|
| **Detection Latency** | <50ms p99 | <30ms p99 | Application performance monitoring (APM) |
| **Rust Bridge Overhead** | <5ms p99 | <2ms p99 | Langfuse distributed tracing |
| **Throughput** | 500 RPS/instance | 1,000+ RPS/instance | Load testing with k6 |
| **Accuracy (F1)** | >85% | >90% | Benchmark against labeled test dataset |
| **Infrastructure Cost** | <$300/month | <$2,000/month at 10M req/day | AWS Cost Explorer |
| **Uptime** | 99.5% | 99.9% | Uptime monitoring (UptimeRobot) |
| **API Compatibility** | 95% OpenAI endpoints | 100% coverage | Integration test suite |
| **Security (Adversarial)** | 0 prompt injections | 0 critical vulnerabilities | garak automated red-teaming |

### 3.3 Non-Goals (Out of Scope for MVP)

- ❌ HIPAA compliance and healthcare-specific entity types (future roadmap)
- ❌ On-premise deployment options (cloud-only for MVP)
- ❌ Custom LLM fine-tuning or model hosting
- ❌ Data loss prevention (DLP) beyond PII detection
- ❌ Real-time prompt injection detection (integrate Lakera Guard as optional addon)

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

**FR-05: MCP Server Exposure (NEW)**  
The system MUST expose its core capabilities as an MCP server following the Model Context Protocol specification:
- Tool definitions for PII detection, redaction, and audit
- OAuth2 authentication for secure AI-to-AI communication
- Rate limiting per MCP client
- Real-time documentation access via MCP tools

### 4.2 PII Detection Engine (Hybrid Architecture)

**FR-06: Tiered Detection Strategy**  
The system MUST implement a two-tier detection pipeline optimizing for latency:

**Tier 1 - Fast Pattern Matching (Rust):**
- Use `aho-corasick` crate for O(n) multi-pattern matching across all patterns simultaneously
- Detect structured PII: Email, Phone, SSN, Credit Cards, API Keys (AWS, OpenAI, GitHub tokens)
- Target latency: <1ms per KB of text
- Coverage: 60-70% of common PII patterns
- **Optimization:** Use `cast()` over `extract()` in PyO3 for type conversion (avoids costly error handling)
- **Optimization:** Leverage PyO3 Bound API for zero-cost Python token access

**Tier 2 - ML-Based Detection (Python/Presidio):**
- Use Microsoft Presidio Analyzer for complex entities: Person Names, Locations, Organizations
- Only invoke if Tier 1 finds no matches OR for specific entity types requiring NLP
- Target latency: <50ms per KB
- Coverage: Named entities requiring contextual understanding

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

### 4.3 Vaulting & State Management

**FR-10: Deterministic Token Generation**  
Detected PII MUST be replaced with consistent, reversible surrogate tokens following the pattern `{{ENTITY_TYPE_INDEX}}` (e.g., `{{PERSON_1}}`, `{{EMAIL_2}}`). The same PII value within a single request MUST always map to the same surrogate.

**FR-11: Encrypted Redis Vault**  
The mapping between surrogates and original values MUST be stored in Redis with:
- **Client-side encryption** using AES-256-GCM with unique ephemeral keys per request
- **Cryptographic key derivation** from tenant secret + request ID
- **Hash data structure**: `pii:{tenant_id}:{request_id}` → `{surrogate_id: encrypted_pii}`
- **Strict TTL**: 600 seconds (10 minutes) for streaming requests, 120 seconds for synchronous

**FR-12: Vault Availability Fail-Safe**  
If Redis is unreachable during rehydration, the system MUST:
1. Return HTTP 503 Service Unavailable to the client
2. Log the incident with request metadata (NO PII)
3. NEVER fail open by returning unrehydrated responses with `{{TOKEN}}` placeholders

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

**FR-15: Sliding Window Buffer for Token Splitting**  
To handle surrogates split across chunk boundaries (e.g., `{{PER` in chunk N, `SON_1}}` in chunk N+1), the system MUST implement:

```python
class StreamingPIIDetector:
    overlap_size: int = 100  # Characters of overlap
    buffer: str = ""
    
    def process_chunk(chunk: str) -> tuple[str, bool]:
        # Accumulate chunks
        # Emit confirmed portion, retain overlap
        # Flush on stream end signal
```

**FR-16: Flush Strategies**  
The buffer MUST flush data to the client when:
- Buffer size exceeds 2× overlap threshold (200 chars)
- Complete sentence detected (`. `, `? `, `! `)
- Timeout of 500ms since last chunk received
- Stream termination signal (`[DONE]` or `message_stop`)

**FR-17: Backpressure Handling**  
If the client consumer is slower than the upstream LLM provider, the system MUST:
- Buffer up to 1MB of data in memory
- Apply TCP backpressure to upstream connection
- Log slow consumer warnings for rate limiting consideration

### 4.5 Multi-Tenancy & Access Control

**FR-18: API Key Authentication**  
Each tenant MUST authenticate using API keys with format: `pg_<random_6>_<random_32>` (e.g., `pg_abc123_xyz789...`). The system MUST:
- Store only SHA-256 hashes of full keys
- Support key rotation without service interruption
- Include key prefix in logs for debugging (first 8 chars only)

**FR-19: Row-Level Security (RLS)**  
Tenant data isolation MUST be enforced via PostgreSQL RLS policies:

```sql
CREATE POLICY tenant_isolation ON api_requests
  USING (tenant_id::TEXT = current_setting('app.current_tenant'));
```

Middleware MUST set `app.current_tenant` session variable on every database connection.

**FR-20: Usage Tracking & Metering**  
The system MUST track per-tenant metrics for billing:
- Total tokens processed (input + output)
- Request count by provider
- Total PII entities detected
- Storage duration in Redis vault

**FR-21: Rate Limiting**  
The system MUST enforce per-tenant rate limits using token bucket algorithm:
- Free tier: 100 requests/hour
- Pro tier: 10,000 requests/hour
- Enterprise tier: Custom limits

### 4.6 SDK & MCP Integration (NEW)

**FR-22: OpenAPI Specification**  
The system MUST maintain a complete OpenAPI 3.1 specification documenting all endpoints, with:
- Detailed descriptions optimized for LLM understanding
- Example requests/responses for each endpoint
- Security scheme definitions
- Error response schemas

**FR-23: Auto-Generated SDKs**  
The system MUST provide auto-generated, type-safe client SDKs for:
- Python (via Speakeasy or Stainless)
- TypeScript/JavaScript (via Speakeasy or Stainless)
- Published to PyPI and npm with semantic versioning

**FR-24: MCP Server Generation**  
The system MUST expose its API as an MCP server that:
- Auto-generates from OpenAPI specification
- Provides tool descriptions optimized for AI agents
- Handles OAuth2 authentication
- Respects model-specific tool limits (e.g., Cursor's 40-tool cap)

---

## 5. Non-Functional Requirements (NFRs)

### 5.1 Security

**NFR-01: Zero PII in Logs**  
Application logs, error messages, and telemetry MUST NEVER contain:
- Raw PII values
- Surrogate token mappings
- Encryption keys or Redis keys containing PII references

Logs MAY contain: Entity types, counts, confidence scores, request IDs.

**NFR-02: Encryption Standards**  
- **In Transit:** TLS 1.2+ for all connections (client ↔ proxy ↔ LLM provider ↔ Redis)
- **At Rest:** AES-256-GCM for PII in Redis vault
- **In Memory:** Use Rust's `zeroize` crate to clear sensitive data from memory after use

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
- Detected entity types and counts
- Provider selection and response status
- Authentication events (key usage, failures)

**NFR-06: Automated Security Testing (NEW)**  
The system MUST integrate automated security tooling:
- **garak:** Vulnerability scanner for prompt injection, jailbreaks (CI/CD integration)
- **promptfoo:** Local evaluation framework for security regression testing
- **Lakera Guard (optional):** Runtime firewall for production deployments (<50ms latency)

### 5.2 Performance

**NFR-07: Async Runtime Optimization**  
The application MUST use `uvloop` instead of standard Python `asyncio` to achieve 2-4× performance improvement for I/O-bound operations.

**NFR-08: GIL Release for CPU-Bound Operations**  
Rust modules MUST use PyO3's `py.allow_threads()` to release the Global Interpreter Lock during:
- Regex pattern matching (`aho-corasick`)
- String manipulation and token replacement
- Cryptographic operations

**NFR-09: PyO3 Bridge Optimization (NEW)**  
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

### 5.3 Reliability

**NFR-12: Graceful Degradation**  
Component failure modes:
- **Redis unavailable:** Return 503, block all traffic (fail-secure)
- **Presidio ML unavailable:** Fall back to Tier 1 regex-only detection with warning
- **Upstream LLM timeout:** Return timeout to client after 60 seconds

**NFR-13: Health Checks**  
The system MUST expose `/health` and `/ready` endpoints:
- `/health`: Liveness probe (HTTP 200 if process alive)
- `/ready`: Readiness probe (HTTP 200 if Redis + DB reachable)

**NFR-14: Circuit Breaker**  
Implement circuit breaker pattern for upstream LLM providers:
- Open circuit after 5 consecutive failures
- Half-open after 30-second cooldown
- Close after 3 successful requests

### 5.4 Observability (NEW)

**NFR-15: Distributed Tracing with Langfuse**  
The system MUST implement OpenTelemetry-based distributed tracing:
- Trace every request through: Ingress → PII Detection (Rust) → Vaulting (Redis) → Proxy → Rehydration
- Nested spans for cross-language boundaries (Python ↔ Rust)
- Self-hosted Langfuse instance (open-source, MIT license)
- No sensitive data in traces (entity types/counts only)

**NFR-16: Metrics & Monitoring**  
The system MUST expose Prometheus-compatible metrics:
- Request count, latency (p50, p95, p99)
- PII detection rate, entity type distribution
- Redis vault hit/miss ratio
- Upstream provider latency and error rates
- Rust bridge overhead (p50, p95, p99)

**NFR-17: Cost Tracking**  
Langfuse MUST track per-request costs:
- Token usage (input/output)
- Provider API costs
- Infrastructure overhead
- Monthly burn rate projections

**NFR-18: Error Tracking**  
Integrate with Sentry or similar for:
- Automatic error aggregation
- Source maps for stack traces
- Release tracking and deployment correlation

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
| **NLP Engine** | spaCy | ≥3.7.0 | Required by Presidio |
| **Performance Core** | Rust + PyO3 | 0.27+ | 3-50× speedup for regex/string ops |
| **Build Tool** | Maturin | ≥1.0.0 | Zero-config Python wheel building |
| **Cache/Vault** | Redis | ≥7.0 | Sub-millisecond latency, TTL support |
| **Database** | PostgreSQL | ≥15 | Row-level security, JSON support |
| **ORM** | SQLAlchemy | ≥2.0.0 | Async support, type safety |
| **Migrations** | Alembic | ≥1.13.0 | Database version control |
| **Encryption** | cryptography | ≥42.0.0 | Industry-standard crypto primitives |
| **Observability** | Langfuse | Latest | Open-source tracing, self-hosted |
| **Security Testing** | garak | Latest | Automated vulnerability scanning |
| **Prompt Testing** | promptfoo | Latest | Local evaluation framework |
| **Container** | Docker | Latest | Reproducible builds |
| **Orchestration** | AWS ECS/EKS | - | Managed Kubernetes or serverless containers |

### 6.2 Rust Crates (via PyO3)

```toml
[dependencies]
pyo3 = { version = "0.27", features = ["extension-module"] }
aho-corasick = "1.1"       # Multi-pattern string search
regex = "1.10"              # Fallback for complex patterns
bstr = "1.9"                # Fast byte string operations
rayon = "1.10"              # Data parallelism
zeroize = "1.7"             # Secure memory clearing
```

### 6.3 AI Development Tooling (NEW)

**Primary Development Stack:**
- **Cursor** ($20/month) - AI-first IDE for daily coding
- **Claude Code** ($20/month) - Terminal agent for autonomous feature development
- **Total Investment:** $40/month

**MCP Ecosystem Integration:**
- **AWS Documentation MCP Server** - Real-time API reference access
- **PostgreSQL MCP Server** - Schema inspection and SQL generation
- **Redis Cloud MCP Server** - Natural language database management
- **Docker MCP Toolkit** - Containerized MCP server execution

**SDK Generation:**
- **Speakeasy** or **Stainless** - Auto-generate Python/TypeScript SDKs and MCP servers from OpenAPI

**Observability & Testing:**
- **Langfuse** - Self-hosted distributed tracing
- **garak** - Automated security testing
- **promptfoo** - Local prompt evaluation

### 6.4 System Architecture Diagram

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
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Ingress    │→ │ PII Detect   │→ │    Vault     │  │
│  │  Middleware  │  │ (Rust/Python)│  │ (Redis AES)  │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│         │                   │                  │         │
│         └───────────────────┴──────────────────┘         │
│                             │                            │
│                   ┌─────────▼─────────┐                  │
│                   │  Provider Router  │                  │
│                   │  (OpenAI/Anthropic)│                 │
│                   └─────────┬─────────┘                  │
└─────────────────────────────┼─────────────────────────────┘
                              │ HTTPS
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
       ┌────────────┐  ┌────────────┐  ┌────────────┐
       │   OpenAI   │  │ Anthropic  │  │Azure OpenAI│
       └────────────┘  └────────────┘  └────────────┘
              │               │               │
              └───────────────┴───────────────┘
                             │
                    ┌────────▼────────┐
                    │  MCP Gateway    │ (Optional)
                    │  OAuth2, Rate   │
                    │  Limiting       │
                    └─────────────────┘
                             │
                    ┌────────▼────────┐
                    │   AI Agents     │
                    │ (Claude, GPT)   │
                    └─────────────────┘
                             │
                    ┌────────▼────────┐
                    │   Langfuse      │
                    │ (Tracing &      │
                    │  Observability) │
                    └─────────────────┘
```

### 6.5 Data Flow: Request Pipeline

**Phase 1: Ingestion**
1. Client sends `POST /v1/chat/completions` with PII-containing prompt
2. API key middleware validates tenant, sets `request.state.tenant_id`
3. Request body parsed and validated via Pydantic
4. **Langfuse creates trace span:** `request_ingestion`

**Phase 2: Detection & Vaulting**
5. **Tier 1 (Rust):** `rust_scanner.detect_patterns(text)` → finds emails, SSNs in <1ms
   - **Langfuse span:** `rust_pattern_matching` (tracks latency)
6. **Tier 2 (Python):** If needed, `presidio.analyze(text)` → finds names, locations in ~50ms
   - **Langfuse span:** `presidio_ml_detection`
7. **Vault:** For each entity:
   - Generate surrogate: `{{PERSON_1}}`, `{{EMAIL_2}}`
   - Encrypt original value with `Fernet(ephemeral_key)`
   - Store in Redis: `HSET pii:{tenant}:{req_id} {surrogate} {encrypted_value}`
   - **Langfuse span:** `redis_vault_storage`
   - Replace in request body

**Phase 3: Upstream Proxy**
8. Transform request to provider format via `ProviderAdapter`
9. Send sanitized request to LLM provider (OpenAI, Anthropic, etc.)
   - **Langfuse span:** `upstream_llm_call`
10. Receive streaming SSE response

**Phase 4: Rehydration**
11. Buffer incoming chunks via `StreamingPIIDetector`
    - **Langfuse span:** `stream_buffering`
12. Detect surrogate tokens (`{{PERSON_1}}`)
13. Retrieve from Redis: `HGET pii:{tenant}:{req_id} {{PERSON_1}}`
    - **Langfuse span:** `redis_vault_retrieval`
14. Decrypt and replace surrogate with original PII
15. Flush rehydrated chunks to client
16. On stream end:
    - Delete Redis hash
    - Zero encryption key in memory (Rust `zeroize`)
    - **Langfuse finalizes trace** with total latency, cost, token count

### 6.6 Database Schema (Core Tables)

**Tenants**
```sql
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    tier VARCHAR(20) NOT NULL CHECK (tier IN ('free', 'pro', 'enterprise')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    settings JSONB DEFAULT '{}'::jsonb,
    mcp_enabled BOOLEAN DEFAULT FALSE,
    mcp_oauth_client_id VARCHAR(255)
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
    revoked_at TIMESTAMPTZ,
    scope VARCHAR(50) DEFAULT 'api' CHECK (scope IN ('api', 'mcp'))
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
    latency_ms INTEGER,
    rust_bridge_latency_ms INTEGER,
    langfuse_trace_id VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW()
) PARTITION BY RANGE (created_at);

-- Enable RLS
ALTER TABLE usage_logs ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON usage_logs
  USING (tenant_id::TEXT = current_setting('app.current_tenant', true));
```

---

## 7. Implementation Roadmap

### Phase 1: Foundation & MCP Setup (Weeks 1-2)
**Goal:** Working non-streaming proxy with AI development tooling configured

**Tasks:**
- [ ] Install and configure Cursor + Claude Code
- [ ] Set up project-wide `CLAUDE.md` with coding standards
- [ ] Initialize MCP servers (AWS Docs, PostgreSQL, Redis Cloud)
- [ ] Set up FastAPI project with uvloop configuration
- [ ] Implement `/v1/chat/completions` endpoint (non-streaming)
- [ ] Create `ProviderAdapter` interface and OpenAI implementation
- [ ] PostgreSQL database setup with Alembic migrations
- [ ] Redis connection with health check endpoint
- [ ] Basic API key authentication middleware
- [ ] Docker containerization with multi-stage build
- [ ] Deploy Langfuse (self-hosted) for distributed tracing
- [ ] Initialize promptfoo for baseline evaluation

**Deliverable:** Proxy that forwards requests to OpenAI with full observability

**Success Criteria:** Can authenticate, route requests, return responses with <10ms overhead, visible traces in Langfuse

**AI Tooling Focus:**
- **Primary:** Cursor for rapid FastAPI development
- **Secondary:** Claude Code for infrastructure setup (Docker, GitHub Actions)
- **MCP:** AWS Documentation MCP for cloud-native best practices

---

### Phase 2: Core Privacy Logic & Skills (Weeks 3-4)
**Goal:** PII detection and streaming support with Claude Code Skills

**Tasks:**
- [ ] Integrate Microsoft Presidio analyzer
- [ ] Implement surrogate token generation (`{{ENTITY_TYPE_INDEX}}`)
- [ ] Build encrypted Redis vault with AES-256-GCM
- [ ] Create `StreamingPIIDetector` with sliding window buffer
- [ ] Implement SSE response handling with `sse-starlette`
- [ ] Build rehydration logic with buffer flushing strategies
- [ ] Add comprehensive unit tests for edge cases (split tokens, overlapping entities)
- [ ] **Create Claude Code Skill: PII Redaction Audit**
  - Automated testing comparing raw vs redacted streams
  - Validation of zero PII leakage
- [ ] Integrate garak for initial security scanning
- [ ] Set up Langfuse spans for each pipeline stage

**Deliverable:** End-to-end privacy protection with streaming and automated auditing

**Success Criteria:** Can detect PII, vault it, stream sanitized responses, rehydrate correctly, pass garak security tests

**AI Tooling Focus:**
- **Primary:** Claude Code for complex streaming logic
- **Secondary:** Cursor for testing and debugging
- **MCP:** PostgreSQL MCP for schema generation, Redis MCP for vault configuration

**Claude Code Skills:**
```markdown
# /skills/pii-redaction-audit/SKILL.md

## Purpose
Audit the AI Privacy Gateway's PII detection accuracy

## When to Use
- After implementing new PII entity types
- Before production deployment
- During CI/CD pipeline

## Workflow
1. Generate synthetic test data with known PII
2. Process through gateway endpoint
3. Compare input vs output (should be 0% PII in output)
4. Report precision, recall, F1 score
5. Flag any leaked entities for investigation
```

---

### Phase 3: Rust Performance Core & Optimization (Weeks 5-6)
**Goal:** Optimize latency-critical paths with Rust, target <5ms bridge overhead

**Tasks:**
- [ ] Initialize Maturin project structure (`maturin init`)
- [ ] Implement Rust pattern matcher using `aho-corasick`
- [ ] Port token replacement logic to Rust with `bstr` for fast string ops
- [ ] Add PyO3 bindings with GIL release (`py.allow_threads()`)
- [ ] **Implement PyO3 micro-optimizations:**
  - Use `cast()` over `extract()` for type conversion
  - Leverage Bound API for zero-cost token access
- [ ] Build manylinux wheels for production deployment
- [ ] Benchmark Rust vs Python implementations
- [ ] Replace Python regex calls with Rust module
- [ ] **Create Claude Code Skill: Performance Budgeting**
  - Analyze flame graphs from `py-spy`
  - Verify <5ms p99 Rust bridge latency
  - Alert on performance regressions
- [ ] Add Langfuse spans for Rust module calls

**Deliverable:** Hybrid Python/Rust architecture with measurable performance gains

**Success Criteria:** Achieve <50ms p99 detection latency, <5ms p99 Rust bridge overhead, 3-10× speedup on regex operations

**AI Tooling Focus:**
- **Primary:** Claude Code for cross-language refactoring
- **Secondary:** Cursor for Rust syntax debugging
- **MCP:** None (focus on local optimization)

**Claude Code Skills:**
```markdown
# /skills/performance-budgeting/SKILL.md

## Purpose
Ensure Python-Rust bridge stays within latency budget

## Performance Targets
- Rust bridge overhead: <5ms p99
- Total PII detection: <50ms p99
- End-to-end request: <100ms p99

## Workflow
1. Run profiler: `py-spy record -o profile.svg -- python main.py`
2. Analyze flame graph for bottlenecks
3. Check Langfuse traces for bridge latency
4. If budget exceeded, identify hot path
5. Optimize using `cast()`, Bound API, or parallelism
6. Re-profile and verify improvement
```

---

### Phase 4: B2B SaaS Features & SDK Generation (Weeks 7-8)
**Goal:** Production-ready multi-tenant platform with auto-generated SDKs

**Tasks:**
- [ ] Implement API key generation with SHA-256 hashing
- [ ] Build usage tracking and metering system
- [ ] Add rate limiting with token bucket algorithm
- [ ] Create admin dashboard for tenant management
- [ ] Set up Prometheus metrics and Grafana dashboards
- [ ] Implement circuit breaker for upstream providers
- [ ] **Generate OpenAPI 3.1 specification with LLM-optimized descriptions**
- [ ] **Auto-generate SDKs using Speakeasy or Stainless:**
  - Python SDK (publish to PyPI)
  - TypeScript SDK (publish to npm)
- [ ] **Generate MCP server from OpenAPI spec**
  - OAuth2 authentication
  - Rate limiting per MCP client
  - Tool descriptions optimized for Claude/GPT
- [ ] Deploy to AWS ECS Fargate with auto-scaling
- [ ] Configure CloudWatch alarms and on-call rotation
- [ ] **Run comprehensive garak security suite**
- [ ] **Create Claude Code Skill: Security Baseline**

**Deliverable:** Production-ready B2B SaaS with monitoring, SDKs, and MCP server

**Success Criteria:** Support 10+ tenants, 99.5% uptime, <$300/month infrastructure cost, SDKs published, MCP server functional

**AI Tooling Focus:**
- **Primary:** Claude Code for infrastructure automation
- **Secondary:** Cursor for dashboard UI development
- **MCP:** AWS Documentation MCP for IaC generation, Docker MCP for containerization

**Claude Code Skills:**
```markdown
# /skills/security-baseline/SKILL.md

## Purpose
Enforce security coding standards in the Privacy Gateway

## Critical Rules
1. NEVER log PII values (only entity types/counts)
2. ALWAYS encrypt before storing in Redis
3. ALWAYS use RLS session variable for queries
4. NEVER hard-code credentials (use environment variables)
5. ALWAYS zero sensitive memory with `zeroize`

## Workflow
1. Before committing code, scan for violations
2. Check for: `print(pii)`, `logger.info(email)`, etc.
3. Verify .env files not in git
4. Run garak security tests
5. Block commit if violations found
```

---

### Phase 5: Provider Expansion & MCP Gateway (Weeks 9-10)
**Goal:** Multi-provider support with centralized MCP orchestration

**Tasks:**
- [ ] Implement `AnthropicAdapter` with message transformation
- [ ] Add Azure OpenAI adapter with custom endpoint support
- [ ] Build provider selection logic (tenant configuration)
- [ ] Create provider-specific error handling
- [ ] Add integration tests for each provider
- [ ] **Deploy optional MCP Gateway for centralized control:**
  - OAuth2 authentication
  - Rate limiting
  - Tool discovery and governance
- [ ] Update documentation with provider setup guides
- [ ] Publish marketing materials (landing page, demo video)

**Deliverable:** Support for OpenAI, Anthropic, and Azure OpenAI with MCP ecosystem integration

**Success Criteria:** Customers can seamlessly switch providers via configuration, AI agents can discover and use gateway via MCP

**AI Tooling Focus:**
- **Primary:** Claude Code for provider adapter implementation
- **Secondary:** Cursor for integration testing
- **MCP:** Test with external AI agents accessing gateway via MCP

---

## 8. Risks & Mitigation Strategies

| Risk | Impact | Probability | Mitigation Strategy |
|------|--------|-------------|---------------------|
| **Token Splitting in Streams** | High | High | Implement robust sliding window buffer with 100-char overlap, word-boundary detection, and comprehensive test suite with edge cases |
| **Latency Budget Exceeded** | High | Medium | Offload CPU-heavy operations to Rust, use uvloop, benchmark continuously via Langfuse, enforce <5ms bridge target with Performance Budgeting Skill |
| **False Positives (Over-detection)** | Medium | Medium | Tune confidence thresholds per entity type, allow customer override, implement feedback loop via Langfuse annotations |
| **Redis Single Point of Failure** | Critical | Low | Deploy Redis Cluster with replication, implement circuit breaker, fail-secure (block traffic if vault unavailable) |
| **Prompt Injection Attacks** | High | Medium | Integrate garak in CI/CD, optional Lakera Guard for production (<50ms latency), implement input sanitization |
| **Encryption Key Management** | Critical | Low | Use AWS KMS for master keys, rotate ephemeral keys per request, implement cryptographic erasure with Rust `zeroize`, audit key access |
| **Deployment Complexity (Hybrid Binary)** | Medium | Medium | Use Maturin for automated wheel building, manylinux2014 compatibility, test on multiple platforms, provide Docker images |
| **AI Tooling Learning Curve** | Medium | Medium | Week 1 onboarding plan, progressive adoption (Cursor first, then Claude Code), community Discord for support |
| **MCP Server Discovery** | Low | Medium | Publish to MCP registry, provide clear documentation, demo video showing AI-to-AI integration |
| **SDK Adoption** | Medium | Low | Publish to PyPI/npm with semantic versioning, comprehensive docs, example code in multiple languages |
| **Competitor Response** | Medium | Medium | Focus on open-source community, emphasize performance/cost advantage, build integrations with complementary tools |
| **Regulatory Changes (GDPR/AI Act)** | Medium | Low | Monitor regulatory updates, maintain compliance documentation, build audit logging from day one |

---

## 9. Success Criteria & Launch Readiness

### MVP Launch Criteria (End of Week 8)

**Technical Readiness:**
- ✅ <50ms p99 latency overhead
- ✅ <5ms p99 Rust bridge latency (tracked via Langfuse)
- ✅ 500+ RPS per instance sustained
- ✅ >85% F1 score on PII detection benchmark
- ✅ Zero PII in logs validated via PII Redaction Audit Skill
- ✅ 99.5% uptime over 1-week test period
- ✅ 0 critical vulnerabilities from garak scanning
- ✅ Langfuse tracing operational with <1% overhead

**Product Readiness:**
- ✅ OpenAPI documentation published with LLM-optimized descriptions
- ✅ Python SDK published to PyPI
- ✅ TypeScript SDK published to npm
- ✅ MCP server functional and documented
- ✅ Quickstart guide with cURL and SDK examples
- ✅ Admin dashboard for tenant management
- ✅ Billing integration (Stripe) functional
- ✅ Customer onboarding flow tested

**Business Readiness:**
- ✅ Pricing tiers defined (Free, Pro, Enterprise)
- ✅ Landing page with demo video
- ✅ 5 beta customers signed up
- ✅ Support channel established (Discord/Slack)
- ✅ MCP registry listing published

### Post-Launch Metrics (First 3 Months)

- **Adoption:** 50 active tenants, 1M API requests/month
- **Performance:** Maintain <50ms p99 latency at scale
- **SDK Downloads:** 500+ PyPI downloads, 300+ npm downloads
- **MCP Integration:** 10+ AI agents using gateway via MCP
- **Revenue:** $2,500 MRR (5 Pro @ $49, 3 Enterprise @ $299)
- **Reliability:** 99.9% uptime, <5 critical incidents
- **Customer Satisfaction:** NPS > 40, <24hr support response
- **Security:** 0 PII leakage incidents, 0 successful prompt injections

---

## 10. Open Questions & Decisions Needed

**Technical Decisions:**
1. **Presidio vs Custom ML Model:** Start with Presidio for MVP, evaluate fine-tuned transformers if accuracy <85%
2. **Redis Cluster Sizing:** Single instance for MVP (<100K req/day), cluster at 1M+ req/day
3. **Kubernetes vs Serverless:** Start with ECS Fargate for simplicity, migrate to EKS if custom scaling needed
4. **Logging Backend:** CloudWatch for MVP, evaluate Loki/Grafana if costs exceed $100/month
5. **Speakeasy vs Stainless:** Speakeasy for dependency-free CLI (preferred), Stainless for SOC 2 compliance if needed
6. **MCP Gateway Deployment:** Optional for MVP, required at scale (>100 AI agent clients)

**Product Decisions:**
1. **Free Tier Limits:** 10K tokens/month or 100 requests/hour? (Pending competitive analysis)
2. **Provider Priority:** OpenAI → Anthropic → Azure, or add Azure earlier for enterprise appeal?
3. **Self-Hosted Option:** Offer Docker Compose for enterprise on-prem? (Future roadmap)
4. **MCP Server Visibility:** Public registry listing or invite-only beta?

**Business Decisions:**
1. **Pricing Model:** Usage-based ($/1K tokens) vs seat-based ($/user/month)?
2. **Target ICP:** AI startups (high volume, low ACV) vs enterprises (low volume, high ACV)?
3. **Go-to-Market:** Product Hunt launch vs direct sales outreach?
4. **AI Tooling Subscription:** Include Cursor/Claude Code cost in pricing or expect customers to provide?

---

## 11. Appendices

### A. Competitive Landscape

| Solution | Type | Strengths | Weaknesses | Pricing | MCP Support |
|----------|------|-----------|------------|---------|-------------|
| **Private AI** | Commercial SaaS | High accuracy (92% F1), healthcare focus | Expensive ($500K+ quotes), batch-only | Enterprise | ❌ No |
| **Protecto** | Commercial SaaS | GDPR/HIPAA certified, audit logging | No streaming, slow (200ms+ latency) | Enterprise | ❌ No |
| **LiteLLM** | Open Source | 100+ providers, great observability | Zero PII detection | Free/Pro $500 | ❌ No |
| **Portkey** | Commercial Gateway | Multi-provider, cost tracking | No native PII detection | $99-$999 | ⚠️ Limited |
| **Presidio** | Open Source Library | Extensible, well-maintained | Not a gateway, needs integration | Free | ❌ No |
| **AI Privacy Gateway (Ours)** | Open Source SaaS | Streaming PII detection, Rust performance, MCP native | New entrant | Free/Pro/Enterprise | ✅ Full |

**Our Differentiation:** 
1. Only open-source solution combining LiteLLM-style routing with Presidio-level detection
2. Optimized for streaming workloads with hybrid Rust performance (<50ms p99)
3. **AI-native ecosystem integration** via MCP server and auto-generated SDKs
4. **Built with AI tools** (Cursor, Claude Code, MCP) - "dogfooding" the future of development

### B. Reference Architecture

**GitHub Projects to Study:**
- [PyO3/maturin](https://github.com/PyO3/maturin) - Build tooling
- [microsoft/presidio](https://github.com/microsoft/presidio) - PII detection
- [sysid/sse-starlette](https://github.com/sysid/sse-starlette) - SSE streaming
- [anthropics/claude-code](https://github.com/anthropics/claude-code) - AI coding agent

**MCP Resources:**
- [Model Context Protocol Specification](https://modelcontextprotocol.io)
- [AWS MCP Servers](https://github.com/aws/mcp-servers)
- [Anthropic MCP Documentation](https://docs.anthropic.com/mcp)

**Benchmark Datasets:**
- [presidio-research](https://github.com/microsoft/presidio-research) - PII detection test sets
- [CredData](https://github.com/duo-labs/creddata) - Credentials and API key patterns

**AI Development Tools:**
- [Cursor Documentation](https://cursor.sh/docs)
- [Claude Code Quickstart](https://code.claude.com/docs)
- [Langfuse Tracing Guide](https://langfuse.com/docs/tracing)
- [garak Security Scanner](https://github.com/leondz/garak)

### C. Glossary

**Core Concepts:**
- **PII (Personally Identifiable Information):** Data that can identify an individual (name, email, SSN, etc.)
- **Surrogate Token:** Placeholder like `{{PERSON_1}}` replacing PII in sanitized requests
- **Vaulting:** Secure storage of PII mapping with encryption and TTL
- **Rehydration:** Replacing surrogate tokens with original PII in LLM responses
- **SSE (Server-Sent Events):** HTTP streaming standard for real-time data push
- **RLS (Row-Level Security):** PostgreSQL feature for multi-tenant data isolation
- **PyO3:** Rust bindings for Python, enabling hybrid architectures
- **GIL (Global Interpreter Lock):** Python's execution bottleneck, released via Rust

**AI Development:**
- **MCP (Model Context Protocol):** Standard for connecting AI agents to external tools and data sources
- **Claude Code Skills:** Modular instruction sets teaching AI agents domain-specific workflows
- **Langfuse:** Open-source observability platform for LLM applications with distributed tracing
- **garak:** Automated vulnerability scanner for LLM security (prompt injection, jailbreaks)
- **promptfoo:** Local evaluation framework for testing prompts and detecting security regressions
- **Speakeasy/Stainless:** SDK generators that create type-safe client libraries from OpenAPI specs
- **Bounded API (PyO3):** Zero-cost access pattern for Python objects in Rust

**Security:**
- **Cryptographic Erasure:** Permanently deleting encryption keys to render encrypted data unrecoverable
- **Lakera Guard:** Production firewall for real-time prompt injection detection (<50ms latency)
- **Zeroize:** Rust crate for securely clearing sensitive data from memory

---

## 12. AI Development Tooling Guide

### 12.1 Recommended Tool Stack

**Primary Tools (Required):**
- **Cursor Pro** - $20/month - AI-first IDE with sub-100ms autocomplete
- **Claude Pro** - $20/month - Powers Claude Code terminal agent
- **Total:** $40/month

**Development Infrastructure (Free):**
- **Langfuse** - Self-hosted observability (open-source)
- **garak** - Security scanning (open-source)
- **promptfoo** - Prompt testing (open-source)

**MCP Ecosystem (Free):**
- AWS Documentation MCP Server
- PostgreSQL MCP Server
- Redis Cloud MCP Server
- Docker MCP Toolkit

**Optional (Future):**
- **Lakera Guard** - $500+/month - Production prompt injection firewall
- **Speakeasy** - Free tier → $99/month - SDK generation
- **MCP Gateway** - Deploy as needed for >100 AI agent clients

### 12.2 Week 1 Onboarding Plan

**Day 1: Installation**
```bash
# Install Cursor
# Download from cursor.sh

# Install Claude Code
brew install claude-code

# Verify installations
claude --version
```

**Day 2: Project Configuration**
Create `CLAUDE.md` in project root:

```markdown
# AI Privacy Gateway - Claude Code Configuration

## Project Overview
Hybrid Python/Rust B2B SaaS for PII detection in LLM proxy requests.
Target: <50ms p99 latency, <5ms Rust bridge overhead.

## Tech Stack
- Python 3.11+ (FastAPI, Presidio, SQLAlchemy)
- Rust (PyO3 for performance-critical regex via aho-corasick)
- PostgreSQL with RLS, Redis for ephemeral vault
- Deployed on AWS ECS Fargate

## Build Commands
- Python tests: `pytest tests/ -v`
- Rust build: `cd rust_modules && maturin develop`
- Full build: `make build`
- Linting: `ruff check . && cargo clippy`
- Security scan: `garak --model localhost:8000`

## Performance Budgets
- Rust bridge: <5ms p99 (CRITICAL)
- PII detection: <50ms p99
- End-to-end: <100ms p99

## Coding Standards
- Use Pydantic v2 for all data models
- All async functions use `asyncio.to_thread` for CPU-bound work
- Rust functions MUST use `py.allow_threads()` to release GIL
- PyO3 optimizations: `cast()` over `extract()`, use Bound API
- NEVER log PII values, only entity types and counts

## Common Workflows

### Adding New PII Entity Type
1. Add pattern to `FAST_PATTERNS` dict in `config.yaml`
2. Update Rust regex matcher in `rust_modules/src/patterns.rs`
3. Add test cases in `tests/test_pii_detection.py`
4. Run PII Redaction Audit Skill
5. Update documentation

### Creating API Endpoints
1. Define Pydantic request/response models
2. Add route to `app/routes/`
3. Implement with tenant isolation (RLS session variable)
4. Add Langfuse tracing spans
5. Add integration test
6. Update OpenAPI docs

### Optimizing Performance
1. Profile with `py-spy record`
2. Check Langfuse traces for bottlenecks
3. If Rust bridge >5ms, use Performance Budgeting Skill
4. Optimize hot path (parallelism, better algorithms)
5. Re-profile and verify

## Critical Security Rules
- NEVER store plaintext PII in Redis
- ALWAYS encrypt before vault storage (AES-256-GCM)
- ALWAYS use RLS session variable: `SET app.current_tenant`
- Rust modules must handle Unicode edge cases
- Test streaming endpoints with mock SSE data
- Run garak before every deployment
```

**Day 3: MCP Server Setup**
Configure Claude Desktop with MCP servers:

```json
// ~/Library/Application Support/Claude/claude_desktop_config.json
{
  "mcpServers": {
    "aws-docs": {
      "command": "npx",
      "args": ["-y", "@aws/mcp-server-docs"]
    },
    "postgresql": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres"],
      "env": {
        "DATABASE_URL": "postgresql://localhost/gateway_dev"
      }
    },
    "redis": {
      "command": "docker",
      "args": ["run", "-i", "redis-mcp-server"]
    }
  }
}
```

**Day 4-5: Practice Tasks**
Build familiarity with the tooling:
1. Use Cursor to create a simple FastAPI endpoint
2. Use Claude Code to add database migrations
3. Test MCP integration by asking Claude to query PostgreSQL schema
4. Practice with Langfuse tracing on local endpoint

### 12.3 Daily Development Workflow

**Morning (Planning):**
1. Review Claude Code's overnight work (if autonomous task assigned)
2. Check Langfuse dashboard for performance regressions
3. Review garak security scan results from CI/CD

**Daytime (Active Coding):**
1. Use **Cursor** for interactive development:
   - FastAPI route implementation
   - Pydantic model creation
   - Quick debugging with inline suggestions
   - Rust syntax fixes with `@file` mentions

2. Use **Claude Code** for complex tasks:
   - Multi-file refactoring (Python ↔ Rust)
   - Database schema migrations
   - Infrastructure automation (Docker, CI/CD)
   - Cross-language debugging

**Evening (Autonomous Tasks):**
1. Assign overnight tasks to Claude Code:
   - "Implement the streaming SSE endpoint from the PRD"
   - "Add comprehensive tests for PII detection edge cases"
   - "Generate OpenAPI spec with LLM-optimized descriptions"

2. Review the next morning, iterate if needed

### 12.4 Tool Selection Matrix

| Task Type | Best Tool | Rationale |
|-----------|-----------|-----------|
| Write FastAPI endpoint | **Cursor** | Autocomplete faster than describing to agent |
| Refactor auth across 10 files | **Claude Code** | Autonomous multi-file coordination |
| Debug Rust compiler error | **Cursor** | Inline suggestions from error messages |
| Build PyO3 integration | **Claude Code** | Complex cross-language task requiring planning |
| Add type hints to code | **Cursor Composer** | Visual diff for review |
| Database migration | **Claude Code + PostgreSQL MCP** | Can inspect schema and generate migration |
| Performance optimization | **Claude Code + Langfuse** | Analyze traces, identify bottleneck, fix |
| Security audit | **garak + Claude Code** | Run automated tests, fix vulnerabilities |
| Generate SDKs | **Speakeasy + Claude Code** | Auto-generate from OpenAPI, review output |

### 12.5 Expected Productivity Gains

**Conservative Estimates:**
- **Routine coding (CRUD, tests):** 2× faster with Cursor autocomplete
- **Complex refactoring:** 3× faster with Claude Code autonomous work
- **Infrastructure setup:** 5× faster with MCP-assisted automation
- **Security testing:** 10× faster with garak automation vs manual penetration testing

**Time Savings Over 8 Weeks:**
- **Without AI tools:** ~320 hours (40 hrs/week × 8 weeks)
- **With AI tools:** ~160-200 hours (2-3× productivity gain)
- **Time saved:** 120-160 hours
- **Value at $50/hour:** $6,000-$8,000
- **Tool cost:** $80 (2 months × $40/month)
- **ROI:** 75-100× return on investment

---

## Document Control

**Version History:**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | Feb 2, 2026 | Benjamin Silva | Initial MVP draft |
| 1.1 | Feb 2, 2026 | Benjamin Silva | Added NFRs, expanded architecture, detailed roadmap |
| **2.0** | **Feb 2, 2026** | **Benjamin Silva** | **Added MCP ecosystem, Claude Code Skills, Langfuse observability, SDK generation, comprehensive AI tooling guide, PyO3 optimizations, security automation (garak, promptfoo), updated architecture diagrams** |

**Review & Approval:**

- [ ] Technical Review: _______________ (Date: _______)
- [ ] Product Review: _______________ (Date: _______)
- [ ] Security Review: _______________ (Date: _______)
- [ ] AI Tooling Validation: _______________ (Date: _______)

**Next Review Date:** March 1, 2026 (post-Phase 1 completion)

**Key Changes in v2.0:**
- Added Model Context Protocol (MCP) integration for AI-native development
- Introduced Claude Code Skills for domain expertise encoding
- Integrated Langfuse for distributed tracing with <5ms bridge latency targets
- Added automated security testing with garak and promptfoo
- Specified SDK generation via Speakeasy/Stainless
- Enhanced PyO3 optimization section (cast vs extract, Bound API)
- Comprehensive AI development tooling guide (Cursor + Claude Code)
- Updated roadmap with MCP server setup and Skills creation
- Added MCP Gateway for centralized AI agent orchestration
- Expanded KPIs to include Rust bridge latency and security metrics
