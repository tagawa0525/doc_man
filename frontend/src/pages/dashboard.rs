use leptos::prelude::*;

use crate::auth::AuthContext;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();

    view! {
        <div>
            <h1 class="title">"ダッシュボード"</h1>
            <div class="columns">
                <div class="column is-4">
                    <div class="box">
                        <h2 class="subtitle">
                            <span class="icon"><i class="fas fa-file-alt"></i></span>
                            " 文書管理"
                        </h2>
                        <p class="mb-3">"文書の作成・閲覧・承認を行います。"</p>
                        <a href="/documents" class="button is-primary is-outlined">"文書一覧へ"</a>
                    </div>
                </div>
                <div class="column is-4">
                    <div class="box">
                        <h2 class="subtitle">
                            <span class="icon"><i class="fas fa-project-diagram"></i></span>
                            " プロジェクト"
                        </h2>
                        <p class="mb-3">"プロジェクトの管理を行います。"</p>
                        <a href="/projects" class="button is-info is-outlined">"プロジェクト一覧へ"</a>
                    </div>
                </div>
                <div class="column is-4">
                    <div class="box">
                        <h2 class="subtitle">
                            <span class="icon"><i class="fas fa-tags"></i></span>
                            " タグ管理"
                        </h2>
                        <p class="mb-3">"文書に付与するタグを管理します。"</p>
                        <a href="/tags" class="button is-warning is-outlined">"タグ一覧へ"</a>
                    </div>
                </div>
            </div>
            {move || {
                let role = auth.role();
                if role.is_some_and(|r| r.is_admin()) {
                    view! {
                        <div class="columns">
                            <div class="column is-4">
                                <div class="box">
                                    <h2 class="subtitle">
                                        <span class="icon"><i class="fas fa-building"></i></span>
                                        " 部署管理"
                                    </h2>
                                    <a href="/departments" class="button is-link is-outlined">"部署一覧へ"</a>
                                </div>
                            </div>
                            <div class="column is-4">
                                <div class="box">
                                    <h2 class="subtitle">
                                        <span class="icon"><i class="fas fa-users"></i></span>
                                        " 社員管理"
                                    </h2>
                                    <a href="/employees" class="button is-link is-outlined">"社員一覧へ"</a>
                                </div>
                            </div>
                            <div class="column is-4">
                                <div class="box">
                                    <h2 class="subtitle">
                                        <span class="icon"><i class="fas fa-cog"></i></span>
                                        " マスタ管理"
                                    </h2>
                                    <a href="/disciplines" class="button is-link is-outlined">"専門分野へ"</a>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}
