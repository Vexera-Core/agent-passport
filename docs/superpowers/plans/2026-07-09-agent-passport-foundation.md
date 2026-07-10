# Agent Passport Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship an npm-ready TypeScript workspace that converts JSON or Markdown agent declarations into deterministic, safety-checked Agent Passport documents.

**Architecture:** JSON Schemas define the protocol. A pure SDK owns parsing, normalization, validation, hashing, and Markdown rendering; CLI, OpenAPI, and MCP adapters depend inward on it. MCP remains text/JSON only and exposes no shell, arbitrary filesystem, network, secret, or live-agent capability.

**Tech Stack:** Node.js 20+, TypeScript 5.8+, pnpm 9, Ajv 8, Commander 13, MCP TypeScript SDK, tsup, Vitest, OpenAPI 3.1, GitHub Actions.

## Global Constraints

- JSON is source of truth; Markdown contains exactly one `json agent-passport` fenced JSON block.
- Public packages: `open-agent-passport`, `open-agent-passport-schema`, `open-agent-passport-cli`, `open-agent-passport-mcp`.
- No private keys, raw memory, hidden injection, shell execution in MCP, arbitrary filesystem access, or fake live integrations.
- Local validation never claims benchmark proof, hosted availability, publication, signing, or reputation.
- Existing staged user files remain untouched and enter phase `-001` only when user commits.
- Agent creates no commits; user commits after each checkpoint.

---

### Task 1: Package workspace

**Files:**
- Create: `package.json`, `pnpm-workspace.yaml`, `pnpm-lock.yaml`, `.npmrc`, `tsconfig.base.json`
- Create: `packages/schema/package.json`, `packages/schema/tsconfig.json`
- Create: `packages/sdk/package.json`, `packages/sdk/tsconfig.json`
- Create: `packages/cli/package.json`, `packages/cli/tsconfig.json`
- Create: `packages/mcp-server/package.json`, `packages/mcp-server/tsconfig.json`
- Modify: `.gitignore`

**Interfaces:**
- Produces workspace package names and inward dependency graph used by all later tasks.

- [ ] Create private root workspace metadata with `build`, `typecheck`, `test`, `lint`, and `check` scripts.
- [ ] Create publication-ready package manifests with `files`, `exports`, `bin`, `engines`, repository metadata, and `publishConfig.access=public`.
- [ ] Run `pnpm install`; expect lockfile generation and exit 0.
- [ ] Run `pnpm typecheck`; expect exit 0 with packages that have no source yet.
- [ ] Stop for user commit `-001--add-package-workspace`.

### Task 2: JSON Schemas

**Files:**
- Create: `packages/schema/schemas/agent-source.schema.json`
- Create: `packages/schema/schemas/agent-passport.schema.json`
- Create: `packages/schema/schemas/verification-receipt.schema.json`
- Create: `packages/schema/src/index.ts`
- Create: `packages/schema/src/schema.test.ts`
- Create: `examples/agent.json`, `examples/agent.md`, `examples/unsafe-agent.json`

**Interfaces:**
- Produces: `agentSourceSchema`, `agentPassportSchema`, `verificationReceiptSchema` JSON values.
- Defines stable finding shape `{ code, severity, path, message }` and protocol version `1.0.0`.

- [ ] Write schema tests that require valid fixtures and reject missing identity, raw memory, private-key fields, shell permission, and unrestricted filesystem permission.
- [ ] Run `pnpm --filter open-agent-passport-schema test`; expect RED because schema exports do not exist.
- [ ] Add JSON Schemas and direct JSON imports in `src/index.ts`.
- [ ] Run package test; expect all schema cases PASS.
- [ ] Stop for user commit `-002--add-json-schemas`.

### Task 3: SDK and CLI

**Files:**
- Create: `packages/sdk/src/types.ts`, `errors.ts`, `markdown.ts`, `canonicalize.ts`, `safety.ts`, `validator.ts`, `passport.ts`, `index.ts`
- Create: `packages/sdk/src/*.test.ts`
- Create: `packages/cli/src/cli.ts`, `packages/cli/src/cli.test.ts`

**Interfaces:**
- Produces: `parseAgentSource(text, format)`, `validateAgentSource(value)`, `buildPassport(value, options)`, `hashCanonical(value)`, `renderPassportMarkdown(passport)`.
- CLI produces `init`, `validate`, `build`, `hash`, `render` commands and stable exit codes 0, 2, 3, 4, 1.

- [ ] Write failing SDK tests for JSON/Markdown parity, duplicate block rejection, deterministic hashes, safety codes, redaction, and deterministic clock injection.
- [ ] Run SDK tests; expect RED from missing exports.
- [ ] Implement minimal pure SDK and run tests; expect PASS.
- [ ] Write failing CLI integration tests using temporary directories and spawned Node process.
- [ ] Implement CLI adapter limited to explicit paths; run CLI tests; expect PASS.
- [ ] Stop for user commit `-003--add-sdk-cli`.

### Task 4: OpenAPI contract

**Files:**
- Create: `openapi/openapi.json`, `openapi/openapi.test.ts`

**Interfaces:**
- Produces operation IDs `passportValidate`, `passportBuild`, `passportRender` with shared error envelope and embedded protocol schemas.

- [ ] Write failing contract test requiring OpenAPI 3.1, three paths, operation IDs, JSON/Markdown request variants, and no server URL.
- [ ] Run contract test; expect RED because contract is absent.
- [ ] Add contract and run test; expect PASS.
- [ ] Stop for user commit `-004--add-openapi-contract`.

### Task 5: MCP server scaffold

**Files:**
- Create: `packages/mcp-server/src/tools.ts`, `server.ts`, `index.ts`, `tools.test.ts`

**Interfaces:**
- Produces tools `passport_validate`, `passport_build`, `passport_hash`, `passport_render_markdown`.
- Each handler consumes structured JSON or Markdown text and returns structured SDK results.

- [ ] Write failing tests for tool names, descriptions, input schemas, valid calls, structured errors, and absence of path/command/url/private-key arguments.
- [ ] Run MCP tests; expect RED from missing tool registry.
- [ ] Implement tool registry and stdio server using MCP SDK.
- [ ] Run MCP tests; expect PASS.
- [ ] Stop for user commit `-005--add-mcp-server-scaffold`.

### Task 6: README, CI, package smoke tests

**Files:**
- Replace: `README.md`
- Create: `packages/*/README.md`, `SECURITY.md`, `CONTRIBUTING.md`
- Create: `.github/workflows/ci.yml`, `.github/actions/agent-passport/action.yml`
- Create: `tests/package-smoke.test.ts`, `examples/node/index.ts`, `examples/nextjs/route.ts`

**Interfaces:**
- Documents npm/local installation without claiming packages are published.
- CI executes the exact existing root scripts on Node.js 20, 22, and 24.

- [ ] Write smoke tests that pack each package and inspect required files/metadata.
- [ ] Run smoke tests; expect RED until README/package artifacts exist.
- [ ] Write BenchArena-style npm README with factual badges, Mermaid flows, JSON/Markdown, Node, Next.js, CLI, MCP, OpenAPI, trust model, current/planned table, and publishing destination.
- [ ] Add package READMEs, security/contributing docs, reusable action, and CI matrix.
- [ ] Run `pnpm check`; expect lint, typecheck, tests, and build PASS.
- [ ] Run `pnpm pack:check`; expect four package dry-runs PASS and no secret/private files.
- [ ] Review requirements against design spec and inspect `git diff --check`.
- [ ] Stop for user commit `-006--add-ci-tests`.
