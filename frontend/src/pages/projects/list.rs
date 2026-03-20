use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::projects::ProjectListParams;
use crate::api::types::{flatten_dept_tree_full, FlatDepartment};
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;

fn current_fiscal_year() -> i32 {
    let now = chrono::Utc::now();
    let year = now.format("%Y").to_string().parse::<i32>().unwrap_or(2025);
    let month = now.format("%m").to_string().parse::<u32>().unwrap_or(4);
    if month < 4 {
        year - 1
    } else {
        year
    }
}

fn csv_contains(csv: &str, key: &str) -> bool {
    csv.split(',').any(|c| c == key)
}

fn csv_toggle(csv: &str, key: &str) -> String {
    let mut items: Vec<&str> = csv.split(',').filter(|c| !c.is_empty()).collect();
    if items.contains(&key) {
        items.retain(|c| *c != key);
    } else {
        items.push(key);
    }
    items.join(",")
}

#[component]
pub fn ProjectListPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let page = RwSignal::new(1u32);
    let search_query = RwSignal::new(String::new());
    let manager_name = RwSignal::new(String::new());
    let show_detail_dept = RwSignal::new(false);
    let show_detail_year = RwSignal::new(false);

    let fy = current_fiscal_year();
    let default_years: Vec<i32> = ((fy - 2)..=(fy + 1)).collect();
    let all_years: Vec<i32> = ((fy - 5)..=(fy + 1)).rev().collect();

    let fiscal_years = RwSignal::new(
        default_years
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(","),
    );

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

    let can_create = Memo::new(move |_| auth.role().is_some_and(|r| r.can_manage()));

    let all_depts = LocalResource::new(|| async { api::departments::list().await });

    let resource = LocalResource::new(move || {
        let p = page.get();
        let q = search_query.get();
        let di = dept_ids.get();
        let fy = fiscal_years.get();
        let mn = manager_name.get();
        async move {
            api::projects::list_filtered(&ProjectListParams {
                page: p,
                per_page: 20,
                q,
                dept_ids: di,
                fiscal_years: fy,
                manager_name: mn,
            })
            .await
        }
    });

    let make_debounced_handler = |signal: RwSignal<String>| {
        let tid = RwSignal::new(0i32);
        move |ev: leptos::ev::Event| {
            let value = event_target::<HtmlInputElement>(&ev).value();
            let window = web_sys::window().unwrap();
            let prev = tid.get_untracked();
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
            tid.set(id);
        }
    };

    let on_search = make_debounced_handler(search_query);
    let on_manager_name = make_debounced_handler(manager_name);

    // ユーザー所属部署のID集合
    let user_dept_ids = Memo::new(move |_| {
        auth.user
            .get()
            .map(|u| {
                u.departments
                    .iter()
                    .map(|d| d.id.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    let default_year_strings: Vec<String> = default_years.iter().map(i32::to_string).collect();

    view! {
        <div>
            <div class="level">
                <div class="level-left"><h1 class="title">"プロジェクト管理"</h1></div>
                {move || if can_create.get() {
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

            // 部署
            <div class="is-flex is-align-items-center is-flex-wrap-wrap mb-2" style="gap: 0.25rem 0.75rem;">
                <span class="has-text-weight-semibold is-size-7">"部署："</span>
                <Suspense fallback=|| ()>
                    {move || all_depts.get().map(|result| match result {
                        Ok(tree) => {
                            let mut flat: Vec<FlatDepartment> = Vec::new();
                            flatten_dept_tree_full(&tree, &mut flat, "");
                            let detail = show_detail_dept.get();
                            let udi = user_dept_ids.get();
                            let visible_ids: Vec<String> = flat.iter()
                                .filter(|d| detail || udi.contains(&d.id))
                                .map(|d| d.id.clone())
                                .collect();
                            let vi = visible_ids.clone();
                            let vi2 = visible_ids.clone();
                            let toggle_all = view! {
                                <a class="is-size-7 has-text-grey" style="cursor:pointer; white-space:nowrap;"
                                    on:click=move |_| {
                                        let current = dept_ids.get_untracked();
                                        let all_checked = vi.iter().all(|id| csv_contains(&current, id));
                                        if all_checked {
                                            dept_ids.set(String::new());
                                        } else {
                                            let mut items: Vec<&str> = current.split(',').filter(|c| !c.is_empty()).collect();
                                            for id in &vi {
                                                if !items.contains(&id.as_str()) {
                                                    items.push(id);
                                                }
                                            }
                                            dept_ids.set(items.join(","));
                                        }
                                        page.set(1);
                                    }
                                >
                                    {move || {
                                        let current = dept_ids.get();
                                        if vi2.iter().all(|id| csv_contains(&current, id)) { "全解除" } else { "全選択" }
                                    }}
                                </a>
                            };
                            let checkboxes = flat.into_iter().filter_map(move |d| {
                                if !detail && !udi.contains(&d.id) {
                                    return None;
                                }
                                let id = d.id.clone();
                                let id2 = id.clone();
                                Some(view! {
                                    <label class="checkbox is-size-7">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || csv_contains(&dept_ids.get(), &id)
                                            on:change=move |_| {
                                                page.set(1);
                                                dept_ids.set(csv_toggle(&dept_ids.get_untracked(), &id2));
                                            }
                                        />
                                        " " {d.label}
                                    </label>
                                })
                            }).collect_view();
                            view! { {toggle_all} {checkboxes} }.into_any()
                        }
                        Err(_) => view! { <span class="tag is-warning">"部署読込失敗"</span> }.into_any(),
                    })}
                </Suspense>
                <a class="is-size-7 has-text-link" style="cursor:pointer; white-space:nowrap;"
                    on:click=move |_| show_detail_dept.update(|v| *v = !*v)
                >
                    {move || if show_detail_dept.get() { "閉じる" } else { "全部署..." }}
                </a>
            </div>

            // 年度
            <div class="is-flex is-align-items-center is-flex-wrap-wrap mb-3" style="gap: 0.25rem 0.5rem;">
                <span class="has-text-weight-semibold is-size-7">"年度："</span>
                {move || {
                    let detail = show_detail_year.get();
                    let dys = default_year_strings.clone();
                    let visible_years: Vec<String> = all_years.iter()
                        .filter(|&&y| detail || dys.contains(&y.to_string()))
                        .map(i32::to_string)
                        .collect();
                    let vy = visible_years.clone();
                    let vy2 = visible_years.clone();
                    let toggle_all = view! {
                        <a class="is-size-7 has-text-grey" style="cursor:pointer; white-space:nowrap;"
                            on:click=move |_| {
                                let current = fiscal_years.get_untracked();
                                let all_checked = vy.iter().all(|y| csv_contains(&current, y));
                                if all_checked {
                                    fiscal_years.set(String::new());
                                } else {
                                    let mut items: Vec<&str> = current.split(',').filter(|c| !c.is_empty()).collect();
                                    for y in &vy {
                                        if !items.contains(&y.as_str()) {
                                            items.push(y);
                                        }
                                    }
                                    fiscal_years.set(items.join(","));
                                }
                                page.set(1);
                            }
                        >
                            {move || {
                                let current = fiscal_years.get();
                                if vy2.iter().all(|y| csv_contains(&current, y)) { "全解除" } else { "全選択" }
                            }}
                        </a>
                    };
                    let checkboxes = all_years.iter().filter_map(move |&y| {
                        let ys = y.to_string();
                        if !detail && !dys.contains(&ys) {
                            return None;
                        }
                        let ys2 = ys.clone();
                        let label = format!("{y}");
                        Some(view! {
                            <label class="checkbox is-size-7">
                                <input
                                    type="checkbox"
                                    prop:checked=move || csv_contains(&fiscal_years.get(), &ys)
                                    on:change=move |_| {
                                        page.set(1);
                                        fiscal_years.set(csv_toggle(&fiscal_years.get_untracked(), &ys2));
                                    }
                                />
                                " " {label}
                            </label>
                        })
                    }).collect_view();
                    view! { {toggle_all} {checkboxes} }
                }}
                <a class="is-size-7 has-text-link" style="cursor:pointer; white-space:nowrap;"
                    on:click=move |_| show_detail_year.update(|v| *v = !*v)
                >
                    {move || if show_detail_year.get() { "閉じる" } else { "全年度..." }}
                </a>
            </div>

            // 絞り込み検索
            <div class="columns mb-4">
                <div class="column">
                    <label class="label is-small">"プロジェクト名"</label>
                    <div class="control has-icons-left">
                        <input class="input is-small" type="text" placeholder="検索..." on:input=on_search />
                        <span class="icon is-left"><i class="fas fa-search"></i></span>
                    </div>
                </div>
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
