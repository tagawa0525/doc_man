use std::fmt::Write;

use super::client::{self, ApiError};
use super::types::{
    CreateDocumentRequest, DocumentResponse, DocumentRevisionResponse, PaginatedResponse,
    ReviseDocumentRequest, UpdateDocumentRequest,
};
use uuid::Uuid;

#[derive(Default)]
pub struct DocumentListParams {
    pub page: u32,
    pub per_page: u32,
    pub q: String,
    pub dept_codes: String,
    pub doc_kind_ids: String,
    pub fiscal_years: String,
    pub project_name: String,
    pub author_name: String,
    pub wbs_code: String,
}

pub async fn list_filtered(
    params: &DocumentListParams,
) -> Result<PaginatedResponse<DocumentResponse>, ApiError> {
    let mut url = format!(
        "/api/v1/documents?page={}&per_page={}",
        params.page, params.per_page
    );
    if !params.q.is_empty() {
        let _ = write!(url, "&q={}", super::encode_query(&params.q));
    }
    if !params.dept_codes.is_empty() {
        let _ = write!(
            url,
            "&dept_codes={}",
            super::encode_query(&params.dept_codes)
        );
    }
    if !params.doc_kind_ids.is_empty() {
        let _ = write!(url, "&doc_kind_ids={}", params.doc_kind_ids);
    }
    if !params.fiscal_years.is_empty() {
        let _ = write!(url, "&fiscal_years={}", params.fiscal_years);
    }
    if !params.project_name.is_empty() {
        let _ = write!(
            url,
            "&project_name={}",
            super::encode_query(&params.project_name)
        );
    }
    if !params.author_name.is_empty() {
        let _ = write!(
            url,
            "&author_name={}",
            super::encode_query(&params.author_name)
        );
    }
    if !params.wbs_code.is_empty() {
        let _ = write!(url, "&wbs_code={}", super::encode_query(&params.wbs_code));
    }
    client::get(&url).await
}

pub async fn list(
    page: u32,
    per_page: u32,
    q: &str,
) -> Result<PaginatedResponse<DocumentResponse>, ApiError> {
    let mut url = format!("/api/v1/documents?page={page}&per_page={per_page}");
    if !q.is_empty() {
        let _ = write!(url, "&q={}", super::encode_query(q));
    }
    client::get(&url).await
}

pub async fn list_by_project(
    project_id: Uuid,
    page: u32,
    per_page: u32,
) -> Result<PaginatedResponse<DocumentResponse>, ApiError> {
    client::get(&format!(
        "/api/v1/documents?project_id={project_id}&page={page}&per_page={per_page}"
    ))
    .await
}

pub async fn get(id: Uuid) -> Result<DocumentResponse, ApiError> {
    client::get(&format!("/api/v1/documents/{id}")).await
}

pub async fn create(req: &CreateDocumentRequest) -> Result<DocumentResponse, ApiError> {
    client::post("/api/v1/documents", req).await
}

pub async fn update(id: Uuid, req: &UpdateDocumentRequest) -> Result<DocumentResponse, ApiError> {
    client::put(&format!("/api/v1/documents/{id}"), req).await
}

pub async fn delete(id: Uuid) -> Result<(), ApiError> {
    client::delete(&format!("/api/v1/documents/{id}")).await
}

pub async fn revise(id: Uuid, req: &ReviseDocumentRequest) -> Result<DocumentResponse, ApiError> {
    client::post(&format!("/api/v1/documents/{id}/revise"), req).await
}

pub async fn list_revisions(id: Uuid) -> Result<Vec<DocumentRevisionResponse>, ApiError> {
    client::get(&format!("/api/v1/documents/{id}/revisions")).await
}
