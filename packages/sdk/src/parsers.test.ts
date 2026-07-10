import { describe, expect, it } from "vitest";
import {
  normalizeIdentity,
  parseIdentityJson,
  parseIdentityMarkdown,
} from "./index.js";

const identity = {
  identity: { id: "demo.agent", name: "Demo Agent" },
  components: [{ id: "review", type: "capability" }],
  tools: [],
};

describe("identity parsers", () => {
  it("parses JSON identity", () => {
    expect(parseIdentityJson(JSON.stringify(identity))).toMatchObject(identity);
  });

  it("parses YAML frontmatter and preserves body as notes", () => {
    const markdown = `---
identity:
  id: demo.agent
  name: Demo Agent
tools: []
---
# Demo Agent

Human operating notes.`;

    expect(parseIdentityMarkdown(markdown)).toMatchObject({
      identity: identity.identity,
      tools: [],
      notes: "# Demo Agent\n\nHuman operating notes.",
    });
  });

  it("parses one agent-passport JSON block and preserves prose", () => {
    const markdown = `# Demo Agent

Human operating notes.

\`\`\`json agent-passport
${JSON.stringify(identity)}
\`\`\``;

    expect(parseIdentityMarkdown(markdown)).toMatchObject({
      ...identity,
      notes: "# Demo Agent\n\nHuman operating notes.",
    });
  });

  it("normalizes defaults and stable list order", () => {
    const normalized = normalizeIdentity({
      ...identity,
      components: [
        { id: "z", type: "capability" },
        { id: "a", type: "capability" },
      ],
    });

    expect(normalized.schemaVersion).toBe("1.0.0");
    expect(normalized.identity.version).toBe("0.0.0");
    expect(normalized.components.map((item) => item.id)).toEqual(["a", "z"]);
    expect(normalized.permissions).toEqual({});
    expect(normalized.memory).toEqual({});
  });
});
