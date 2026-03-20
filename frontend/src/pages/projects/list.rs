use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::projects::ProjectListParams;
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;

fn current_fiscal_year() -> i32 {
    let now = chrono::Utc::now();
    let year = now.format("%Y").to_string().parse::<i32>().unwrap_or(2025);
    let month = now.format("%m").to_string().parse::<u32>().unwrap_or(4);
    if month < 4 { year - 1 } else { year }
}

#[component]
pub fn ProjectListPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let page = RwSignal::new(1u32);
    let search_query = RwSignal::new(String::new());
    let manager_name = RwSignal::new(String::new());
    let timer_id = RwSignal::new(0i32);

    let fiscal_year = current_fiscal_year();
    let selected_fiscal_year = RwSignal::new(fiscal_year.to_string());

    // dept_ids は初期値空（auth ロード後に Effect で設定）
    let dept_ids = RwSignal::new(String::new());
    let dept_ids_initialized = RwSignal::new(false);

    Effect::new(move || {
        if let Some(ref u) = auth.user.get() {
            if !dept_ids_initialized.get_untracked() {
                let ids = u
                    .departments
                    .iter()
                    .map(|d| d.id.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                dept_ids.set(ids);
                dept_ids_initialized.set(true);
            }
        }
    });

    let is_admin = auth.role().is_some_and(|r| r.is_admin());

    let resource = LocalResource::new(move || {
        let p = page.get();
        let q = search_query.get();
        let di = dept_ids.get();
        let fy = selected_fiscal_year.get();
        let mn = manager_name.get();
        async move {
            api::projects::list_filtered(&ProjectListParams {
                page: p,
                per_page: 20,
                q,
                dept_ids: di,
                fiscal_year: fy,
                manager_name: mn,
            })
            .await
        }
    });

    let make_debounced_handler = |signal: RwSignal<String>| {
        move |ev: leptos::ev::Event| {
            let value = event_target::<HtmlInputElement>(&ev).value();
            let window = web_sys::window().unwrap();
            let prev = timer_id.get_untracked();
            if prev != 0 {
                window.clear_timeout_with_handle(prev);
            }
            let cb = Closure::once(move || {
                page.set(1);
                signal.set(value);
            });
            let id = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    300,
                )
                .unwrap();
            cb.forget();
            timer_id.set(id);
        }
    };

    let on_search = make_debounced_handler(search_query);
    let on_manager_name = make_debounced_handler(manager_name);

    let years: Vec<i32> = ((fiscal_year - 3)..=(fiscal_year + 1)).rev().collect();

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

            // フィルタバー
            <div class="columns is-multiline mb-4">
                // 部署チェックボックス
                <div class="column is-narrow">
                    <label class="label is-small">"部署"</label>
                    <div class="field is-grouped">
                        {move || {
                            let depts = auth.user.get().map(|u| u.departments).unwrap_or_default();
                            depts.into_iter().map(|d| {
                                let id_str = d.id.to_string();
                                let name = d.name.clone();
                                let id_for_check = id_str.clone();
                                let id_for_handler = id_str.clone();
                                view! {
                                    <label class="checkbox mr-3">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || {
                                                let ids = dept_ids.get();
                                                ids.split(',').any(|c| c == id_for_check)
                                            }
                                            on:change=move |_| {
                                                let current = dept_ids.get_untracked();
                                                let mut ids: Vec<&str> = current.split(',').filter(|c| !c.is_empty()).collect();
                                                if ids.contains(&id_for_handler.as_str()) {
                                                    ids.retain(|c| *c != id_for_handler.as_str());
                                                } else {
                                                    ids.push(&id_for_handler);
                                                }
                                                page.set(1);
                                                dept_ids.set(ids.join(","));
                                            }
                                        />
                                        " " {name}
                                    </label>
                                }
                            }).collect_view()
                        }}
                    </div>
                </div>

                // 年度
                <div class="column is-narrow">
                    <label class="label is-small">"年度"</label>
                    <div class="select is-small">
                        <select
                            prop:value=move || selected_fiscal_year.get()
                            on:change=move |ev| {
                                let val = event_target::<web_sys::HtmlSelectElement>(&ev).value();
                                page.set(1);
                                selected_fiscal_year.set(val);
                            }
                        >
                            <option value="">"全て"</option>
                            {years.into_iter().map(|y| {
                                let label = format!("{y}年度");
                                let val = y.to_string();
                                let selected = y == fiscal_year;
                                view! { <option value=val selected=selected>{label}</option> }
                            }).collect_view()}
                        </select>
                    </div>
                </div>

                // プロジェクト名
                <div class="column">
                    <label class="label is-small">"プロジェクト名"</label>
                    <div class="control has-icons-left">
                        <input
                            class="input is-small"
                            type="text"
                            placeholder="検索..."
                            on:input=on_search
                        />
                        <span class="icon is-left"><i class="fas fa-search"></i></span>
                    </div>
                </div>

                // 担当者
                <div class="column">
                    <label class="label is-small">"担当者"</label>
                    <input class="input is-small" type="text" placeholder="部分一致..." on:input=on_manager_name />
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
