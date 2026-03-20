use leptos::prelude::*;

use crate::api;
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;

#[component]
pub fn ProjectListPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let page = RwSignal::new(1u32);

    let is_admin = auth.role().is_some_and(|r| r.is_admin());

    let resource = LocalResource::new(move || {
        let p = page.get();
        async move { api::projects::list(p, 20).await }
    });

    view! {
        <div>
            <div class="level">
                <div class="level-left"><h1 class="title">"プロジェクト管理"</h1></div>
                {if is_admin {
                    view! {
                        <div class="level-right">
                            <a href="/projects/new" class="button is-primary">
                                <span class="icon"><i class="fas fa-plus"></i></span>
                                <span>"新規作成"</span>
                            </a>
                        </div>
                    }.into_any()
                } else { view! { <div></div> }.into_any() }}
            </div>

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    resource.get().map(|result| match result {
                        Ok(paginated) => {
                            let total = paginated.meta.total;
                            let cp = paginated.meta.page;
                            let pp = paginated.meta.per_page;
                            view! {
                                <div class="box">
                                    <table class="table is-fullwidth is-hoverable">
                                        <thead>
                                            <tr>
                                                <th>"名前"</th><th>"ステータス"</th><th>"専門分野"</th>
                                                <th>"部署"</th><th>"マネージャー"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {paginated.data.into_iter().map(|p| {
                                                let detail_url = format!("/projects/{}", p.id);
                                                view! {
                                                    <tr>
                                                        <td><a href=detail_url>{p.name}</a></td>
                                                        <td><span class="tag is-light">{p.status}</span></td>
                                                        <td>{p.discipline.name}</td>
                                                        <td>{p.discipline.department.name}</td>
                                                        <td>{p.manager.map_or_else(|| "-".to_string(), |m| m.name)}</td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                    <Pagination current_page=cp total=total per_page=pp on_page_change=Callback::new(move |p| page.set(p)) />
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! { <div class="notification is-danger">{format!("読み込み失敗: {}", e.message)}</div> }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
