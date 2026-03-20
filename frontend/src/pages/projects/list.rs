use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

use crate::api;
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;

#[component]
pub fn ProjectListPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let page = RwSignal::new(1u32);
    let search_query = RwSignal::new(String::new());
    let timer_id = RwSignal::new(0i32);

    let is_admin = auth.role().is_some_and(|r| r.is_admin());

    let resource = LocalResource::new(move || {
        let p = page.get();
        let q = search_query.get();
        async move { api::projects::list(p, 20, &q).await }
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

            <div class="field mb-4">
                <div class="control has-icons-left">
                    <input
                        class="input"
                        type="text"
                        placeholder="プロジェクト名で検索..."
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
