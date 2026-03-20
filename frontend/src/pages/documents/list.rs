use leptos::prelude::*;

use crate::api;
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;
use crate::components::status_badge::StatusBadge;

#[component]
pub fn DocumentListPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let page = RwSignal::new(1u32);

    let can_create = auth
        .role()
        .is_some_and(|r| !matches!(r, crate::auth::Role::Viewer));

    let resource = LocalResource::new(move || {
        let p = page.get();
        async move { api::documents::list(p, 20).await }
    });

    view! {
        <div>
            <div class="level">
                <div class="level-left"><h1 class="title">"文書一覧"</h1></div>
                {if can_create {
                    view! {
                        <div class="level-right">
                            <a href="/documents/new" class="button is-primary">
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
                                                <th>"文書番号"</th><th>"Rev."</th><th>"タイトル"</th><th>"ステータス"</th>
                                                <th>"種別"</th><th>"プロジェクト"</th><th>"作成者"</th>
                                                <th>"タグ"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {paginated.data.into_iter().map(|doc| {
                                                let detail_url = format!("/documents/{}", doc.id);
                                                view! {
                                                    <tr>
                                                        <td><span class="has-text-weight-semibold">{doc.doc_number}</span></td>
                                                        <td>{doc.revision.to_string()}</td>
                                                        <td><a href=detail_url>{doc.title}</a></td>
                                                        <td><StatusBadge status=doc.status /></td>
                                                        <td>{doc.doc_kind.name}</td>
                                                        <td>{doc.project.name}</td>
                                                        <td>{doc.author.name}</td>
                                                        <td>
                                                            <div class="tags">
                                                                {doc.tags.into_iter().map(|t| view! { <span class="tag is-info is-light">{t}</span> }).collect_view()}
                                                            </div>
                                                        </td>
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
