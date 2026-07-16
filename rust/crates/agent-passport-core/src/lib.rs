use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentitySummary {
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedIdentity {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub identity: IdentitySummary,
    #[serde(default)]
    pub runtime: Value,
    #[serde(default)]
    pub components: Value,
    #[serde(default)]
    pub tools: Value,
    #[serde(default)]
    pub permissions: Value,
    #[serde(default)]
    pub memory: Value,
    #[serde(default)]
    pub metadata: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityGateReason {
    pub code: String,
    pub message: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityGateResult {
    pub status: SecurityGateStatus,
    pub warnings: Vec<SecurityGateReason>,
    pub reasons: Vec<SecurityGateReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SecurityGateStatus {
    Passed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentPassport {
    pub passport_id: String,
    pub identity_hash: String,
    pub player_id: String,
    pub normalized_identity: NormalizedIdentity,
    pub security_gate: SecurityGateResult,
    pub verification_status: String,
    pub proof_status: String,
    pub schema_version: String,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProofReceipt {
    pub receipt_id: String,
    pub identity_hash: String,
    pub passport_hash: String,
    pub player_id: String,
    pub network: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PassportBuildResult {
    pub normalized_identity: NormalizedIdentity,
    pub passport: AgentPassport,
    pub proof_receipt: ProofReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlayerIdResult {
    pub player_id: String,
    pub passport_hash: String,
    pub network: String,
}

pub fn build_passport(
    identity: NormalizedIdentity,
    generated_at: impl Into<String>,
) -> PassportBuildResult {
    let generated_at = generated_at.into();
    let identity_hash = hash_canonical_json(&identity);
    let security_gate = run_security_gate(&identity);
    let player_id = issue_player_id_from_hash(&identity_hash);
    let passport_id = prefixed_hash("passport", &identity_hash);

    let passport = AgentPassport {
        passport_id,
        identity_hash: identity_hash.clone(),
        player_id: player_id.clone(),
        normalized_identity: identity.clone(),
        security_gate,
        verification_status: "declared".to_string(),
        proof_status: "offchain".to_string(),
        schema_version: default_schema_version(),
        generated_at: generated_at.clone(),
    };

    let passport_hash = hash_canonical_json(&passport);
    let proof_receipt = ProofReceipt {
        receipt_id: prefixed_hash("receipt", &passport_hash),
        identity_hash,
        passport_hash,
        player_id,
        network: "offchain-devnet".to_string(),
        status: "ready".to_string(),
        created_at: generated_at,
    };

    PassportBuildResult {
        normalized_identity: identity,
        passport,
        proof_receipt,
    }
}

pub fn issue_player_id(passport: &AgentPassport) -> PlayerIdResult {
    let passport_hash = hash_canonical_json(passport);
    PlayerIdResult {
        player_id: issue_player_id_from_hash(&passport_hash),
        passport_hash,
        network: "offchain-devnet".to_string(),
    }
}

pub fn run_security_gate(identity: &NormalizedIdentity) -> SecurityGateResult {
    let mut reasons = Vec::new();
    scan_value(
        "",
        &serde_json::to_value(identity).expect("identity serializes"),
        &mut reasons,
    );

    if matches!(identity.permissions.get("shell"), Some(Value::Bool(true))) {
        reasons.push(reason(
            "UNRESTRICTED_SHELL",
            "/permissions/shell",
            "Shell access is not allowed.",
        ));
    }
    if matches!(identity.permissions.get("filesystem"), Some(Value::String(value)) if value == "unrestricted" || value == "all")
    {
        reasons.push(reason(
            "UNRESTRICTED_FILESYSTEM",
            "/permissions/filesystem",
            "Unrestricted filesystem access is not allowed.",
        ));
    }
    if matches!(
        identity.memory.get("rawContentIncluded"),
        Some(Value::Bool(true))
    ) {
        reasons.push(reason(
            "RAW_MEMORY",
            "/memory/rawContentIncluded",
            "Raw memory upload is not allowed.",
        ));
    }
    if let Value::Array(tools) = &identity.tools {
        for (index, tool) in tools.iter().enumerate() {
            if matches!(tool.get("hidden"), Some(Value::Bool(true))) {
                reasons.push(reason(
                    "HIDDEN_TOOL",
                    &format!("/tools/{index}/hidden"),
                    "Hidden tools are not allowed.",
                ));
            }
        }
    }

    reasons.sort_by(|a, b| a.code.cmp(&b.code).then(a.path.cmp(&b.path)));
    reasons.dedup();

    SecurityGateResult {
        status: if reasons.is_empty() {
            SecurityGateStatus::Passed
        } else {
            SecurityGateStatus::Blocked
        },
        warnings: Vec::new(),
        reasons,
    }
}

pub fn hash_canonical_json<T: Serialize>(value: &T) -> String {
    let value = serde_json::to_value(value).expect("value serializes");
    let canonical = canonical_json(&value);
    let digest = Sha256::digest(canonical.as_bytes());
    format!("sha256_{}", hex::encode(digest))
}

pub fn canonical_json(value: &Value) -> String {
    match sort_value(value) {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => serde_json::to_string(&value).expect("string serializes"),
        Value::Array(items) => {
            let rendered = items
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{rendered}]")
        }
        Value::Object(map) => {
            let rendered = map
                .iter()
                .map(|(key, value)| {
                    let key = serde_json::to_string(key).expect("key serializes");
                    format!("{key}:{}", canonical_json(value))
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{rendered}}}")
        }
    }
}

fn sort_value(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(sort_value).collect()),
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                sorted.insert(key.clone(), sort_value(&map[key]));
            }
            Value::Object(sorted)
        }
        other => other.clone(),
    }
}

fn scan_value(path: &str, value: &Value, reasons: &mut Vec<SecurityGateReason>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = format!("{path}/{}", escape_pointer(key));
                let lower = key.to_ascii_lowercase();
                if lower.contains("privatekey") || lower == "private_key" {
                    reasons.push(reason(
                        "PRIVATE_KEY",
                        &child_path,
                        "Private keys are not allowed.",
                    ));
                }
                if lower.contains("seedphrase") || lower == "seed_phrase" {
                    reasons.push(reason(
                        "SEED_PHRASE",
                        &child_path,
                        "Seed phrases are not allowed.",
                    ));
                }
                if lower.contains("rawmemory") || lower == "raw_memory_upload" {
                    reasons.push(reason(
                        "RAW_MEMORY",
                        &child_path,
                        "Raw memory upload is not allowed.",
                    ));
                }
                if lower.contains("hiddeninjection") || lower == "hidden_injection" {
                    reasons.push(reason(
                        "HIDDEN_INJECTION",
                        &child_path,
                        "Hidden injection is not allowed.",
                    ));
                }
                scan_value(&child_path, child, reasons);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                scan_value(&format!("{path}/{index}"), child, reasons);
            }
        }
        _ => {}
    }
}

fn issue_player_id_from_hash(hash: &str) -> String {
    prefixed_hash("player", hash)
}

fn prefixed_hash(prefix: &str, input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    format!("{prefix}_sha256_{}", &hex::encode(digest)[..32])
}

fn reason(code: &str, path: &str, message: &str) -> SecurityGateReason {
    SecurityGateReason {
        code: code.to_string(),
        message: message.to_string(),
        path: if path.is_empty() {
            "/".to_string()
        } else {
            path.to_string()
        },
    }
}

fn escape_pointer(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

fn default_schema_version() -> String {
    "1.0.0".to_string()
}

fn default_version() -> String {
    "0.0.0".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> NormalizedIdentity {
        serde_json::from_value(serde_json::json!({
            "schemaVersion": "1.0.0",
            "identity": {
                "id": "example.agent",
                "name": "Example Agent",
                "version": "1.0.0"
            },
            "permissions": {
                "shell": false,
                "filesystem": "none"
            },
            "memory": {
                "rawContentIncluded": false
            },
            "tools": []
        }))
        .unwrap()
    }

    #[test]
    fn hashes_canonical_json_stably() {
        let left = serde_json::json!({ "b": 2, "a": 1 });
        let right = serde_json::json!({ "a": 1, "b": 2 });

        assert_eq!(hash_canonical_json(&left), hash_canonical_json(&right));
    }

    #[test]
    fn builds_offchain_passport_and_receipt() {
        let result = build_passport(identity(), "2026-07-16T00:00:00Z");

        assert!(result.passport.passport_id.starts_with("passport_sha256_"));
        assert!(result.passport.player_id.starts_with("player_sha256_"));
        assert_eq!(result.passport.verification_status, "declared");
        assert_eq!(result.proof_receipt.network, "offchain-devnet");
        assert_eq!(result.proof_receipt.status, "ready");
    }

    #[test]
    fn blocks_private_keys_and_raw_memory() {
        let mut identity = identity();
        identity.metadata = serde_json::json!({ "privateKey": "do-not-echo" });
        identity.memory = serde_json::json!({ "rawContentIncluded": true });

        let gate = run_security_gate(&identity);

        assert_eq!(gate.status, SecurityGateStatus::Blocked);
        let rendered = serde_json::to_string(&gate).unwrap();
        assert!(rendered.contains("PRIVATE_KEY"));
        assert!(rendered.contains("RAW_MEMORY"));
        assert!(!rendered.contains("do-not-echo"));
    }

    #[test]
    fn issues_player_id_from_passport_hash() {
        let result = build_passport(identity(), "2026-07-16T00:00:00Z");
        let player = issue_player_id(&result.passport);

        assert!(player.player_id.starts_with("player_sha256_"));
        assert_eq!(player.network, "offchain-devnet");
    }
}
