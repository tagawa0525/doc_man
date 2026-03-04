use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "project_manager")]
    ProjectManager,
    #[serde(rename = "general")]
    General,
    #[serde(rename = "viewer")]
    Viewer,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub role: Role,
    pub is_active: bool,
}

// スタブ実装
// PR-1で FromRequestParts を実装して DB 連携を追加する
