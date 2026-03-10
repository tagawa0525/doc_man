#![allow(dead_code)]

mod api;
mod auth;
mod components;
mod pages;

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use auth::{AuthContext, Role, UserInfo};
use components::layout::AppLayout;
use components::toast::{ToastContainer, ToastContext};
use pages::dashboard::DashboardPage;
use pages::departments::DepartmentsPage;
use pages::disciplines::DisciplinesPage;
use pages::document_kinds::DocumentKindsPage;
use pages::document_registers::DocumentRegistersPage;
use pages::documents::create::DocumentCreatePage;
use pages::documents::detail::DocumentDetailPage;
use pages::documents::list::DocumentListPage;
use pages::employees::form::EmployeeFormPage;
use pages::employees::list::EmployeeListPage;
use pages::login::LoginPage;
use pages::projects::form::ProjectFormPage;
use pages::projects::list::ProjectListPage;
use pages::tags::TagsPage;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let auth_ctx = AuthContext::new();
    let toast_ctx = ToastContext::new();

    provide_context(auth_ctx);
    provide_context(toast_ctx);

    // Check auth on mount
    let auth = auth_ctx;
    leptos::task::spawn_local(async move {
        if let Some(me) = auth::verify_token().await {
            auth.user.set(Some(UserInfo {
                id: me.id,
                role: Role::from_str(&me.role),
            }));
        }
        auth.loading.set(false);
    });

    view! {
        <ToastContainer />
        <Router>
            <Routes fallback=|| view! { <div class="main-area"><h1 class="title">"404 - ページが見つかりません"</h1></div> }>
                <Route path=path!("/login") view=LoginPage />
                <Route path=path!("/") view=|| view! { <AuthGuard><AppLayout><DashboardPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/departments") view=|| view! { <AuthGuard><AppLayout><DepartmentsPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/employees") view=|| view! { <AuthGuard><AppLayout><EmployeeListPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/employees/new") view=|| view! { <AuthGuard><AppLayout><EmployeeFormPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/employees/:id") view=|| view! { <AuthGuard><AppLayout><EmployeeFormPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/disciplines") view=|| view! { <AuthGuard><AppLayout><DisciplinesPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/document-kinds") view=|| view! { <AuthGuard><AppLayout><DocumentKindsPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/document-registers") view=|| view! { <AuthGuard><AppLayout><DocumentRegistersPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/projects") view=|| view! { <AuthGuard><AppLayout><ProjectListPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/projects/new") view=|| view! { <AuthGuard><AppLayout><ProjectFormPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/projects/:id") view=|| view! { <AuthGuard><AppLayout><ProjectFormPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/documents") view=|| view! { <AuthGuard><AppLayout><DocumentListPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/documents/new") view=|| view! { <AuthGuard><AppLayout><DocumentCreatePage /></AppLayout></AuthGuard> } />
                <Route path=path!("/documents/:id") view=|| view! { <AuthGuard><AppLayout><DocumentDetailPage /></AppLayout></AuthGuard> } />
                <Route path=path!("/tags") view=|| view! { <AuthGuard><AppLayout><TagsPage /></AppLayout></AuthGuard> } />
            </Routes>
        </Router>
    }
}

#[component]
fn AuthGuard(children: Children) -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let loading = auth.loading;

    // Redirect when not authenticated
    Effect::new(move || {
        if !loading.get() && !auth.is_authenticated() {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/login");
            }
        }
    });

    let children = children();

    view! {
        <div style:display=move || if loading.get() { "block" } else { "none" }>
            <div class="login-container">
                <div class="spinner-overlay">
                    <span class="icon is-large"><i class="fas fa-spinner fa-pulse fa-3x"></i></span>
                </div>
            </div>
        </div>
        <div style:display=move || if !loading.get() && auth.is_authenticated() { "block" } else { "none" }>
            {children}
        </div>
    }
}
