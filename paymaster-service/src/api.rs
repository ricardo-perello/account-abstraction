use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::signature_service::{SignatureService, SponsorshipRequest, SponsorshipResponse, Metrics};

pub async fn sign_sponsorship(
    State(signature_service): State<Arc<SignatureService>>,
    Json(request): Json<SponsorshipRequest>,
) -> Result<Json<SponsorshipResponse>, (StatusCode, String)> {
    signature_service
        .sign_sponsorship(request)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn get_metrics(
    State(signature_service): State<Arc<SignatureService>>,
) -> Json<Metrics> {
    Json(signature_service.get_metrics().await)
}
