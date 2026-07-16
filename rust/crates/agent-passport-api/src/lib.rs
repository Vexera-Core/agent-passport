use agent_passport_core::{
    build_passport, issue_player_id, AgentPassport, NormalizedIdentity, PassportBuildResult,
    PlayerIdResult,
};
use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPassportRequest {
    pub normalized_identity: NormalizedIdentity,
    #[serde(default = "default_dev_timestamp")]
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPassportRequest {
    pub passport: AgentPassport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyPassportResponse {
    pub valid: bool,
    pub player_id: String,
    pub verification_status: String,
    pub proof_status: String,
    pub network: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuePlayerIdRequest {
    pub passport: AgentPassport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub network: String,
    pub live_chain: bool,
}

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/passports/build", post(build_passport_handler))
        .route("/v1/passports/verify", post(verify_passport_handler))
        .route("/v1/players/issue-id", post(issue_player_id_handler))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        network: "offchain-devnet".to_string(),
        live_chain: false,
    })
}

async fn build_passport_handler(
    Json(request): Json<BuildPassportRequest>,
) -> Json<PassportBuildResult> {
    Json(build_passport(
        request.normalized_identity,
        request.generated_at,
    ))
}

async fn verify_passport_handler(Json(request): Json<VerifyPassportRequest>) -> Response {
    let mut reasons = Vec::new();
    if request.passport.verification_status != "declared" {
        reasons.push("verificationStatus must remain declared in the offchain API.".to_string());
    }
    if request.passport.proof_status != "offchain" {
        reasons.push("proofStatus must remain offchain until chain anchoring exists.".to_string());
    }
    if request.passport.security_gate.status != agent_passport_core::SecurityGateStatus::Passed {
        reasons.push("security gate must be passed.".to_string());
    }

    let valid = reasons.is_empty();
    let status = if valid {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };

    (
        status,
        Json(VerifyPassportResponse {
            valid,
            player_id: request.passport.player_id,
            verification_status: request.passport.verification_status,
            proof_status: request.passport.proof_status,
            network: "offchain-devnet".to_string(),
            reasons,
        }),
    )
        .into_response()
}

async fn issue_player_id_handler(
    Json(request): Json<IssuePlayerIdRequest>,
) -> Json<PlayerIdResult> {
    Json(issue_player_id(&request.passport))
}

fn default_dev_timestamp() -> String {
    "2026-07-16T00:00:00Z".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

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

    #[tokio::test]
    async fn health_exposes_offchain_devnet() {
        let response = router()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn builds_passport_for_frontend() {
        let body = serde_json::json!({
            "normalizedIdentity": identity(),
            "generatedAt": "2026-07-16T00:00:00Z"
        });

        let response = router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/passports/build")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
