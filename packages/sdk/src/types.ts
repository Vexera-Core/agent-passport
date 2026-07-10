export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | JsonValue[] | { [key: string]: JsonValue };

export interface AgentComponent {
  id: string;
  type: string;
  description?: string;
  [key: string]: JsonValue | undefined;
}

export interface AgentTool {
  id: string;
  description?: string;
  hidden?: boolean;
  [key: string]: JsonValue | undefined;
}

export interface AgentIdentity {
  schemaVersion?: string;
  identity: {
    id: string;
    name: string;
    version?: string;
    description?: string;
  };
  runtime?: Record<string, JsonValue>;
  components?: AgentComponent[];
  tools?: AgentTool[];
  permissions?: Record<string, JsonValue>;
  memory?: Record<string, JsonValue>;
  metadata?: Record<string, JsonValue>;
  notes?: string;
}

export interface NormalizedIdentity {
  schemaVersion: "1.0.0";
  identity: {
    id: string;
    name: string;
    version: string;
    description?: string;
  };
  runtime: Record<string, JsonValue>;
  components: AgentComponent[];
  tools: AgentTool[];
  permissions: Record<string, JsonValue>;
  memory: Record<string, JsonValue>;
  metadata: Record<string, JsonValue>;
  notes?: string;
}

export interface SecurityGateReason {
  code: string;
  message: string;
  path: string;
}

export interface SecurityGateResult {
  status: "passed" | "blocked";
  warnings: SecurityGateReason[];
  reasons: SecurityGateReason[];
}

export interface AgentPassport {
  passportId: string;
  identityHash: string;
  normalizedIdentity: NormalizedIdentity;
  securityGate: SecurityGateResult;
  verificationStatus: "declared";
  proofStatus: "offchain";
  schemaVersion: "1.0.0";
  generatedAt: string;
}

export interface ProofReceipt {
  receiptId: string;
  identityHash: string;
  passportHash: string;
  network: "offchain";
  status: "ready";
  createdAt: string;
}

export interface PassportBuildResult {
  normalizedIdentity: NormalizedIdentity;
  passport: AgentPassport;
  proofReceipt: ProofReceipt;
}

export interface PassportBuildOptions {
  now?: () => Date;
}

export interface PassportVerificationResult {
  valid: boolean;
  reasons: SecurityGateReason[];
}
