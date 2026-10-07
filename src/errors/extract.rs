use axum::Json;
use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;

use super::AppError;

/// Wrapper di [`Json`] che fa rientrare la deserializzazione nella pipeline
/// degli errori: un corpo non valido produce un [`AppError::Validation`]
/// (400 + JSON uniforme) invece del testo semplice di axum.
pub struct AppJson<T>(pub T);

impl<S, T> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| AppError::Validation(rejection.body_text()))?;
        Ok(Self(value))
    }
}
