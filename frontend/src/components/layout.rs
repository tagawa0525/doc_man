use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

use crate::auth::{AuthContext, Role};

#[component]
pub fn AppLayout(children: Children) -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let location = use_location();

    let nav_items: Vec<(&str, &str, &str, bool)> = vec![
        ("/", "ダッシュボード", "fas fa-home", false),
        ("/documents", "文書", "fas fa-file-alt", false),
        ("/projects", "プロジェクト", "fas fa-project-diagram", false),
        ("/departments", "部署", "fas fa-building", true),
        ("/employees", "社員", "fas fa-users", true),
        ("/disciplines", "専門分野", "fas fa-microscope", true),
        ("/document-kinds", "文書種別", "fas fa-folder-open", true),
        ("/document-registers", "文書台帳", "fas fa-book", true),
        ("/tags", "タグ", "fas fa-tags", false),
    ];

    view! {
        <div class="app-container">
            <aside class="sidebar">
                <div class="sidebar-brand">
                    <span class="icon mr-2"><i class="fas fa-file-contract"></i></span>
                    "Doc Man"
                </div>
                <div class="menu p-2">
                    <p class="menu-label">"メニュー"</p>
                    <ul class="menu-list">
                        {nav_items.into_iter().map(|(path, label, icon, admin_only)| {
                            let path_owned = path.to_string();
                            let is_active = {
                                let path_owned = path_owned.clone();
                                move || {
                                    let current = location.pathname.get();
                                    if path_owned == "/" {
                                        current == "/"
                                    } else {
                                        current.starts_with(&path_owned)
                                    }
                                }
                            };

                            let show = if admin_only {
                                auth.role().is_some_and(|r| r.is_admin())
                            } else {
                                true
                            };

                            if show {
                                view! {
                                    <li>
                                        <A href=path attr:class=move || if is_active() { "is-active" } else { "" }>
                                            <span class="icon"><i class=icon></i></span>
                                            <span>{label}</span>
                                        </A>
                                    </li>
                                }.into_any()
                            } else {
                                view! { <li></li> }.into_any()
                            }
                        }).collect_view()}
                    </ul>
                    <p class="menu-label">"アカウント"</p>
                    <ul class="menu-list">
                        <li>
                            {move || {
                                let role = auth.role().unwrap_or(Role::Viewer);
                                view! {
                                    <span class="px-3 py-2 is-block">
                                        <span class="tag is-info is-light">{role.display_name()}</span>
                                    </span>
                                }
                            }}
                        </li>
                        <li>
                            <a on:click=move |_| {
                                auth.logout();
                                if let Some(window) = web_sys::window() {
                                    let _ = window.location().set_href("/login");
                                }
                            }>
                                <span class="icon"><i class="fas fa-sign-out-alt"></i></span>
                                <span>"ログアウト"</span>
                            </a>
                        </li>
                    </ul>
                </div>
            </aside>
            <main class="main-area">
                {children()}
            </main>
        </div>
    }
}
