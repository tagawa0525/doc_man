use serde::Serialize;
use uuid::Uuid;

pub mod approval_step;
pub mod department;
pub mod discipline;
pub mod distribution;
pub mod document;
pub mod document_kind;
pub mod document_register;
pub mod employee;
pub mod project;
pub mod tag;

/// `{ id, code, name }` の共通Brief型
#[derive(Debug, Serialize)]
pub struct DepartmentBrief {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}

/// `{ id, code, name }` の共通Brief型（文書種別用）
#[derive(Debug, Serialize)]
pub struct DocKindBrief {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}
