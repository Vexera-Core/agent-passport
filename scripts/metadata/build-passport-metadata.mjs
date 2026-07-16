import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join, relative, resolve } from "node:path";

const [, , sourceArg = ".passport", outArg] = process.argv;

const sourceDir = resolve(process.cwd(), sourceArg);
const outFile = resolve(
  process.cwd(),
  outArg ?? join(sourceArg, "passport-metadata.json"),
);

const requiredFiles = {
  normalizedIdentity: "normalized-identity.json",
  agentPassport: "agent-passport.json",
  proofReceipt: "proof-receipt.json",
};

async function readJsonFile(fileName) {
  const absolutePath = join(sourceDir, fileName);
  const raw = await readFile(absolutePath, "utf8");
  return {
    raw,
    json: JSON.parse(raw),
    sha256: createHash("sha256").update(raw).digest("hex"),
  };
}

function requiredString(value, path) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`Missing required string: ${path}`);
  }
  return value;
}

function buildMetadata(files) {
  const normalizedIdentity = files.normalizedIdentity.json;
  const passport = files.agentPassport.json;
  const receipt = files.proofReceipt.json;

  return {
    metadataVersion: "1.0.0",
    generatedAt: new Date().toISOString(),
    transport: {
      status: "portable",
      source: relative(process.cwd(), sourceDir) || ".",
    },
    agent: {
      id: requiredString(normalizedIdentity.identity?.id, "normalizedIdentity.identity.id"),
      name: requiredString(normalizedIdentity.identity?.name, "normalizedIdentity.identity.name"),
      version: normalizedIdentity.identity?.version ?? "0.0.0",
    },
    passport: {
      passportId: requiredString(passport.passportId, "passport.passportId"),
      identityHash: requiredString(passport.identityHash, "passport.identityHash"),
      passportHash: requiredString(receipt.passportHash, "receipt.passportHash"),
      verificationStatus: passport.verificationStatus ?? "declared",
      proofStatus: passport.proofStatus ?? "offchain",
      schemaVersion: passport.schemaVersion ?? "1.0.0",
    },
    securityGate: passport.securityGate ?? {
      status: "blocked",
      warnings: [],
      reasons: [{ code: "MISSING_SECURITY_GATE", message: "Passport has no securityGate.", path: "/" }],
    },
    receipt: {
      receiptId: requiredString(receipt.receiptId, "receipt.receiptId"),
      network: receipt.network ?? "offchain",
      status: receipt.status ?? "ready",
      createdAt: receipt.createdAt,
    },
    files: Object.fromEntries(
      Object.entries(requiredFiles).map(([key, fileName]) => [
        key,
        {
          path: fileName,
          sha256: files[key].sha256,
        },
      ]),
    ),
  };
}

async function main() {
  const files = Object.fromEntries(
    await Promise.all(
      Object.entries(requiredFiles).map(async ([key, fileName]) => [
        key,
        await readJsonFile(fileName),
      ]),
    ),
  );

  const metadata = buildMetadata(files);
  await mkdir(dirname(outFile), { recursive: true });
  await writeFile(outFile, `${JSON.stringify(metadata, null, 2)}\n`, "utf8");

  console.log(`passport metadata written: ${relative(process.cwd(), outFile)}`);
  console.log(`agent: ${metadata.agent.id}`);
  console.log(`security gate: ${metadata.securityGate.status}`);
  console.log(`passport hash: ${metadata.passport.passportHash}`);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
