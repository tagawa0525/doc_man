use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

use crate::api;
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;
use crate::components::status_badge::StatusBadge;

#[component]
pub fn DocumentListPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let page = RwSignal::new(1u32);
    let search_query = RwSignal::new(String::new());
    let timer_id = RwSignal::new(0i32);

    let can_create = auth
        .role()
        .is_some_and(|r| !matches!(r, crate::auth::Role::Viewer));

    let resource = LocalResource::new(move || {
        let p = page.get();
        let q = search_query.get();
        async move { api::documents::list(p, 20, &q).await }
    });

    let on_input = move |ev: leptos::ev::Event| {
        let value = event_target::<HtmlInputElement>(&ev).value();
        let window = web_sys::window().unwrap();
        let prev = timer_id.get_untracked();
        if prev != 0 {
            window.clear_timeout_with_handle(prev);
        }
        let cb = Closure::once(move || {
            page.set(1);
            search_query.set(value);
        });
        let id = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), 300)
            .unwrap();
        cb.forget();
        timer_id.set(id);
    };

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

            <div class="field mb-4">
                <div class="control has-icons-left">
                    <input
                        class="input"
                        type="text"
                        placeholder="タイトル・文書番号で検索..."
                        on:input=on_input
                    />
                    <span class="icon is-left"><i class="fas fa-search"></i></span>
                </div>
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
