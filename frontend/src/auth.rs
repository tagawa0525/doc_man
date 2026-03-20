use leptos::prelude::*;
use uuid::Uuid;

use crate::api::client;
use crate::api::types::{MeDepartment, MeResponse};

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Admin,
    ProjectManager,
    General,
    Viewer,
}

impl Role {
    pub fn from_str(s: &str) -> Self {
        match s {
            "admin" => Role::Admin,
            "project_manager" => Role::ProjectManager,
            "general" => Role::General,
            _ => Role::Viewer,
        }
    }

    pub fn can_manage(&self) -> bool {
        matches!(self, Role::Admin | Role::ProjectManager)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Role::Admin => "管理者",
            Role::ProjectManager => "プロジェクトマネージャー",
            Role::General => "一般",
            Role::Viewer => "閲覧者",
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: Uuid,
    pub role: Role,
    pub departments: Vec<MeDepartment>,
}

#[derive(Debug, Clone, Copy)]
pub struct AuthContext {
    pub user: RwSignal<Option<UserInfo>>,
    pub loading: RwSignal<bool>,
}

impl AuthContext {
    pub fn new() -> Self {
        Self {
            user: RwSignal::new(None),
            loading: RwSignal::new(true),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.get().is_some()
    }

    pub fn role(&self) -> Option<Role> {
        self.user.get().map(|u| u.role)
    }

    #[allow(clippy::unused_self)]
    pub fn login(&self, token: &str) {
        client::set_token(token);
    }

    pub fn logout(&self) {
        client::clear_token();
        self.user.set(None);
    }
}

pub async fn verify_token() -> Option<MeResponse> {
    if !client::has_token() {
        return None;
    }
    client::get::<MeResponse>("/api/v1/me").await.ok()
}
