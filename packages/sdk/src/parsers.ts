import { parse as parseYaml } from "yaml";
import type { AgentIdentity, JsonValue, NormalizedIdentity } from "./types.js";

const FRONTMATTER = /^---\s*\r?\n([\s\S]*?)\r?\n---\s*(?:\r?\n|$)/;
const JSON_BLOCK = /```json(?:\s+agent-passport)?\s*\r?\n([\s\S]*?)\r?\n```/g;

function assertIdentity(value: unknown): asserts value is AgentIdentity {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError("Agent identity must be an object.");
  }

  const identity = (value as { identity?: unknown }).identity;
  if (!identity || typeof identity !== "object" || Array.isArray(identity)) {
    throw new TypeError("Agent identity requires an identity object.");
  }

  const record = identity as { id?: unknown; name?: unknown };
  if (typeof record.id !== "string" || !record.id.trim()) {
    throw new TypeError("Agent identity requires identity.id.");
  }
  if (typeof record.name !== "string" || !record.name.trim()) {
    throw new TypeError("Agent identity requires identity.name.");
  }
}

export function parseIdentityJson(input: string): AgentIdentity {
  const value: unknown = JSON.parse(input);
  assertIdentity(value);
  return value;
}

export function parseIdentityMarkdown(input: string): AgentIdentity {
  const frontmatter = input.match(FRONTMATTER);
  if (frontmatter) {
    const value: unknown = parseYaml(frontmatter[1] ?? "");
    assertIdentity(value);
    const notes = input.slice(frontmatter[0].length).trim();
    return notes ? { ...value, notes } : value;
  }

  const matches = [...input.matchAll(JSON_BLOCK)];
  if (matches.length !== 1) {
    throw new TypeError("Markdown requires exactly one JSON code block.");
  }

  const value = parseIdentityJson(matches[0]?.[1] ?? "");
  const notes = input.replace(matches[0]?.[0] ?? "", "").trim();
  return notes ? { ...value, notes } : value;
}

function sortRecord(record: Record<string, JsonValue> | undefined): Record<string, JsonValue> {
  return Object.fromEntries(Object.entries(record ?? {}).sort(([a], [b]) => a.localeCompare(b)));
}

export function normalizeIdentity(source: AgentIdentity): NormalizedIdentity {
  assertIdentity(source);
  const normalized: NormalizedIdentity = {
    schemaVersion: "1.0.0",
    identity: {
      id: source.identity.id.trim(),
      name: source.identity.name.trim(),
      version: source.identity.version?.trim() || "0.0.0",
      ...(source.identity.description?.trim()
        ? { description: source.identity.description.trim() }
        : {}),
    },
    runtime: sortRecord(source.runtime),
    components: [...(source.components ?? [])].sort((a, b) => a.id.localeCompare(b.id)),
    tools: [...(source.tools ?? [])].sort((a, b) => a.id.localeCompare(b.id)),
    permissions: sortRecord(source.permissions),
    memory: sortRecord(source.memory),
    metadata: sortRecord(source.metadata),
    ...(source.notes?.trim() ? { notes: source.notes.trim() } : {}),
  };
  return normalized;
}
