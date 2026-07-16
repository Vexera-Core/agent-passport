const lines = [
  "Agent Passport transport flow:",
  "",
  "1. agent.md or identity.json",
  "2. normalized-identity.json",
  "3. security gate",
  "4. agent-passport.json",
  "5. proof-receipt.json",
  "6. passport-metadata.json",
  "",
  "Core rule:",
  "Markdown/JSON input -> normalized JSON -> security gate -> passport -> proof receipt -> metadata JSON",
  "",
  "Metadata JSON is a portable summary.",
  "It is not live verification, not benchmark proof, and not a place for private keys or raw memory.",
  "",
  "Build metadata from passport output:",
  "pnpm metadata:build -- .passport .passport/passport-metadata.json",
];

console.log(lines.join("\n"));
