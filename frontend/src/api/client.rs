use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{de::DeserializeOwned, Serialize};

use super::types::ErrorResponse;

const AUTH_TOKEN_KEY: &str = "auth_token";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub status: u16,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

fn get_token() -> Option<String> {
    LocalStorage::get::<String>(AUTH_TOKEN_KEY).ok()
}

pub fn set_token(token: &str) {
    let _ = LocalStorage::set(AUTH_TOKEN_KEY, token.to_string());
}

pub fn clear_token() {
    LocalStorage::delete(AUTH_TOKEN_KEY);
}

pub fn has_token() -> bool {
    get_token().is_some()
}

fn handle_unauthorized() {
    clear_token();
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/login");
    }
}

async fn parse_error(resp: gloo_net::http::Response) -> ApiError {
    let status = resp.status();
    match resp.json::<ErrorResponse>().await {
        Ok(err_resp) => ApiError {
            code: err_resp.error.code,
            message: err_resp.error.message,
            status,
        },
        Err(_) => ApiError {
            code: "UNKNOWN".to_string(),
            message: format!("HTTP {status}"),
            status,
        },
    }
}

pub async fn get<T: DeserializeOwned>(url: &str) -> Result<T, ApiError> {
    let mut req = Request::get(url);
    if let Some(token) = get_token() {
        req = req.header("Authorization", &format!("Bearer {token}"));
    }

    let resp = req.send().await.map_err(|e| ApiError {
        code: "NETWORK".to_string(),
        message: e.to_string(),
        status: 0,
    })?;

    if resp.status() == 401 {
        handle_unauthorized();
        return Err(ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: "Unauthorized".to_string(),
            status: 401,
        });
    }

    if !resp.ok() {
        return Err(parse_error(resp).await);
    }

    resp.json::<T>().await.map_err(|e| ApiError {
        code: "PARSE".to_string(),
        message: e.to_string(),
        status: 0,
    })
}

pub async fn post<T: DeserializeOwned, B: Serialize>(url: &str, body: &B) -> Result<T, ApiError> {
    let mut req = Request::post(url);
    if let Some(token) = get_token() {
        req = req.header("Authorization", &format!("Bearer {token}"));
    }

    let resp = req
        .header("Content-Type", "application/json")
        .json(body)
        .map_err(|e| ApiError {
            code: "SERIALIZE".to_string(),
            message: e.to_string(),
            status: 0,
        })?
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NETWORK".to_string(),
            message: e.to_string(),
            status: 0,
        })?;

    if resp.status() == 401 {
        handle_unauthorized();
        return Err(ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: "Unauthorized".to_string(),
            status: 401,
        });
    }

    if !resp.ok() {
        return Err(parse_error(resp).await);
    }

    resp.json::<T>().await.map_err(|e| ApiError {
        code: "PARSE".to_string(),
        message: e.to_string(),
        status: 0,
    })
}

pub async fn put<T: DeserializeOwned, B: Serialize>(url: &str, body: &B) -> Result<T, ApiError> {
    let mut req = Request::put(url);
    if let Some(token) = get_token() {
        req = req.header("Authorization", &format!("Bearer {token}"));
    }

    let resp = req
        .header("Content-Type", "application/json")
        .json(body)
        .map_err(|e| ApiError {
            code: "SERIALIZE".to_string(),
            message: e.to_string(),
            status: 0,
        })?
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NETWORK".to_string(),
            message: e.to_string(),
            status: 0,
        })?;

    if resp.status() == 401 {
        handle_unauthorized();
        return Err(ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: "Unauthorized".to_string(),
            status: 401,
        });
    }

    if !resp.ok() {
        return Err(parse_error(resp).await);
    }

    resp.json::<T>().await.map_err(|e| ApiError {
        code: "PARSE".to_string(),
        message: e.to_string(),
        status: 0,
    })
}

pub async fn delete(url: &str) -> Result<(), ApiError> {
    let mut req = Request::delete(url);
    if let Some(token) = get_token() {
        req = req.header("Authorization", &format!("Bearer {token}"));
    }

    let resp = req.send().await.map_err(|e| ApiError {
        code: "NETWORK".to_string(),
        message: e.to_string(),
        status: 0,
    })?;

    if resp.status() == 401 {
        handle_unauthorized();
        return Err(ApiError {
            code: "UNAUTHORIZED".to_string(),
            message: "Unauthorized".to_string(),
            status: 401,
        });
    }

    if !resp.ok() {
        return Err(parse_error(resp).await);
    }

    Ok(())
}

pub async fn post_no_response<B: Serialize>(url: &str, body: &B) -> Result<(), ApiError> {
    let _ = post::<serde_json::Value, B>(url, body).await?;
    Ok(())
}
