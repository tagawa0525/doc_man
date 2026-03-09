use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{DepartmentBrief, DocKindBrief};

/// DB行型（document_registers + document_kinds + departments JOIN）
#[derive(Debug, Clone)]
pub struct DocumentRegisterRow {
    pub id: Uuid,
    pub register_code: String,
    pub file_server_root: String,
    pub new_doc_sub_path: Option<String>,
    pub doc_number_pattern: Option<String>,
    pub doc_kind_id: Uuid,
    pub doc_kind_code: String,
    pub doc_kind_name: String,
    pub dept_id: Uuid,
    pub dept_code: String,
    pub dept_name: String,
}

/// レスポンス型
#[derive(Debug, Serialize)]
pub struct DocumentRegisterResponse {
    pub id: Uuid,
    pub register_code: String,
    pub file_server_root: String,
    pub new_doc_sub_path: Option<String>,
    pub doc_number_pattern: Option<String>,
    pub doc_kind: DocKindBrief,
    pub department: DepartmentBrief,
}

impl From<DocumentRegisterRow> for DocumentRegisterResponse {
    fn from(row: DocumentRegisterRow) -> Self {
        DocumentRegisterResponse {
            id: row.id,
            register_code: row.register_code,
            file_server_root: row.file_server_root,
            new_doc_sub_path: row.new_doc_sub_path,
            doc_number_pattern: row.doc_number_pattern,
            doc_kind: DocKindBrief {
                id: row.doc_kind_id,
                code: row.doc_kind_code,
                name: row.doc_kind_name,
            },
            department: DepartmentBrief {
                id: row.dept_id,
                code: row.dept_code,
                name: row.dept_name,
            },
        }
    }
}

/// POST リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateDocumentRegisterRequest {
    pub register_code: String,
    pub doc_kind_id: Uuid,
    pub department_id: Uuid,
    pub file_server_root: String,
    pub new_doc_sub_path: Option<String>,
    pub doc_number_pattern: Option<String>,
}

/// PUT リクエスト
#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRegisterRequest {
    pub register_code: Option<String>,
    pub file_server_root: Option<String>,
    pub new_doc_sub_path: Option<String>,
    pub doc_number_pattern: Option<String>,
}
