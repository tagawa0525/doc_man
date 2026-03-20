use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::documents::DocumentListParams;
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
pub fn DocumentListPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let query_map = leptos_router::hooks::use_query_map();
    let page = RwSignal::new(1u32);
    let search_query = RwSignal::new(String::new());
    let project_name = RwSignal::new(String::new());
    let author_name = RwSignal::new(String::new());
    let wbs_code = RwSignal::new(String::new());

    // URLクエリパラメータ ?wbs_code= をリアクティブに同期
    Effect::new(move || {
        let wc = query_map.get().get("wbs_code").unwrap_or_default();
        wbs_code.set(wc);
    });
    let selected_doc_kinds = RwSignal::new(String::new());
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

    let dept_codes = RwSignal::new(String::new());
    let dept_codes_initialized = RwSignal::new(false);

    Effect::new(move || {
        if let Some(ref u) = auth.user.get() {
            if !dept_codes_initialized.get_untracked() {
                let codes = u
                    .departments
                    .iter()
                    .map(|d| d.code.clone())
                    .collect::<Vec<_>>()
                    .join(",");
                dept_codes.set(codes);
                dept_codes_initialized.set(true);
            }
        }
    });

    let can_create = auth
        .role()
        .is_some_and(|r| !matches!(r, crate::auth::Role::Viewer));

    let doc_kinds = LocalResource::new(|| async { api::document_kinds::list_all().await });
    let all_depts = LocalResource::new(|| async { api::departments::list().await });

    let resource = LocalResource::new(move || {
        let p = page.get();
        let q = search_query.get();
        let dc = dept_codes.get();
        let dk = selected_doc_kinds.get();
        let fy = fiscal_years.get();
        let pn = project_name.get();
        let an = author_name.get();
        let wc = wbs_code.get();
        async move {
            api::documents::list_filtered(&DocumentListParams {
                page: p,
                per_page: 20,
                q,
                dept_codes: dc,
                doc_kind_ids: dk,
                fiscal_years: fy,
                project_name: pn,
                author_name: an,
                wbs_code: wc,
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
    let on_project_name = make_debounced_handler(project_name);
    let on_author_name = make_debounced_handler(author_name);
    let on_wbs_code = make_debounced_handler(wbs_code);

    let user_dept_codes = Memo::new(move |_| {
        auth.user
            .get()
            .map(|u| {
                u.departments
                    .iter()
                    .map(|d| d.code.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    let default_year_strings: Vec<String> = default_years.iter().map(i32::to_string).collect();

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

            // 部署
            <div class="is-flex is-align-items-center is-flex-wrap-wrap mb-2" style="gap: 0.25rem 0.75rem;">
                <span class="has-text-weight-semibold is-size-7">"部署："</span>
                <Suspense fallback=|| ()>
                    {move || all_depts.get().map(|result| match result {
                        Ok(tree) => {
                            let mut flat: Vec<FlatDepartment> = Vec::new();
                            flatten_dept_tree_full(&tree, &mut flat, "");
                            let detail = show_detail_dept.get();
                            let udc = user_dept_codes.get();
                            // 表示中の部署コード一覧
                            let visible_codes: Vec<String> = flat.iter()
                                .filter(|d| detail || udc.contains(&d.code))
                                .map(|d| d.code.clone())
                                .collect();
                            let vc = visible_codes.clone();
                            let toggle_all = view! {
                                <a class="is-size-7 has-text-grey" style="cursor:pointer; white-space:nowrap;"
                                    on:click=move |_| {
                                        let current = dept_codes.get_untracked();
                                        let all_checked = vc.iter().all(|c| csv_contains(&current, c));
                                        if all_checked {
                                            dept_codes.set(String::new());
                                        } else {
                                            let mut items: Vec<&str> = current.split(',').filter(|c| !c.is_empty()).collect();
                                            for c in &vc {
                                                if !items.contains(&c.as_str()) {
                                                    items.push(c);
                                                }
                                            }
                                            dept_codes.set(items.join(","));
                                        }
                                        page.set(1);
                                    }
                                >
                                    {
                                        let vc2 = visible_codes.clone();
                                        move || {
                                            let current = dept_codes.get();
                                            if vc2.iter().all(|c| csv_contains(&current, c)) { "全解除" } else { "全選択" }
                                        }
                                    }
                                </a>
                            };
                            let checkboxes = flat.into_iter().filter_map(move |d| {
                                if !detail && !udc.contains(&d.code) {
                                    return None;
                                }
                                let code = d.code.clone();
                                let code2 = code.clone();
                                Some(view! {
                                    <label class="checkbox is-size-7">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || csv_contains(&dept_codes.get(), &code)
                                            on:change=move |_| {
                                                page.set(1);
                                                dept_codes.set(csv_toggle(&dept_codes.get_untracked(), &code2));
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

            // 文書種別
            <div class="is-flex is-align-items-center is-flex-wrap-wrap mb-3" style="gap: 0.25rem 0.75rem;">
                <span class="has-text-weight-semibold is-size-7">"種別："</span>
                <Suspense fallback=|| ()>
                    {move || doc_kinds.get().map(|result| match result {
                        Ok(paginated) => {
                            let all_ids: Vec<String> = paginated.data.iter().map(|dk| dk.id.to_string()).collect();
                            let ai = all_ids.clone();
                            let ai2 = all_ids.clone();
                            let toggle_all = view! {
                                <a class="is-size-7 has-text-grey" style="cursor:pointer; white-space:nowrap;"
                                    on:click=move |_| {
                                        let current = selected_doc_kinds.get_untracked();
                                        let all_checked = ai.iter().all(|id| csv_contains(&current, id));
                                        if all_checked {
                                            selected_doc_kinds.set(String::new());
                                        } else {
                                            selected_doc_kinds.set(ai.join(","));
                                        }
                                        page.set(1);
                                    }
                                >
                                    {move || {
                                        let current = selected_doc_kinds.get();
                                        if !current.is_empty() && ai2.iter().all(|id| csv_contains(&current, id)) { "全解除" } else { "全選択" }
                                    }}
                                </a>
                            };
                            let checkboxes = paginated.data.into_iter().map(|dk| {
                                let id = dk.id.to_string();
                                let id2 = id.clone();
                                view! {
                                    <label class="checkbox is-size-7">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || csv_contains(&selected_doc_kinds.get(), &id)
                                            on:change=move |_| {
                                                page.set(1);
                                                selected_doc_kinds.set(csv_toggle(&selected_doc_kinds.get_untracked(), &id2));
                                            }
                                        />
                                        " " {dk.name}
                                    </label>
                                }
                            }).collect_view();
                            view! { {toggle_all} {checkboxes} }.into_any()
                        }
                        Err(_) => view! { <span class="tag is-warning">"種別読込失敗"</span> }.into_any(),
                    })}
                </Suspense>
            </div>

            // 絞り込み検索
            <div class="columns is-multiline mb-2">
                <div class="column">
                    <label class="label is-small">"タイトル・文書番号"</label>
                    <div class="control has-icons-left">
                        <input class="input is-small" type="text" placeholder="検索..." on:input=on_search />
                        <span class="icon is-left"><i class="fas fa-search"></i></span>
                    </div>
                </div>
                <div class="column">
                    <label class="label is-small">"プロジェクト名"</label>
                    <input class="input is-small" type="text" placeholder="部分一致..." on:input=on_project_name />
                </div>
                <div class="column">
                    <label class="label is-small">"作成者"</label>
                    <input class="input is-small" type="text" placeholder="部分一致..." on:input=on_author_name />
                </div>
                <div class="column">
                    <label class="label is-small">"WBSコード"</label>
                    <input class="input is-small" type="text" placeholder="部分一致..."
                        prop:value=move || wbs_code.get()
                        on:input=on_wbs_code />
                </div>
            </div>

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    resource.get().map(|result| match result {
                        Ok(paginated) => {
                            let total = paginated.meta.total;
                            let cp = paginated.meta.page;
                            let pp = paginated.meta.per_page;

                            // doc_kind ごとにグループ化（出現順を保持）
                            let mut kind_order: Vec<(String, String)> = Vec::new();
                            let mut groups: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
                            for doc in paginated.data {
                                let kind_id = doc.doc_kind.id.to_string();
                                if !groups.contains_key(&kind_id) {
                                    kind_order.push((kind_id.clone(), doc.doc_kind.name.clone()));
                                }
                                groups.entry(kind_id).or_default().push(doc);
                            }

                            let tables = kind_order.into_iter().map(|(kind_id, kind_name)| {
                                let docs = groups.remove(&kind_id).unwrap_or_default();
                                view! {
                                    <div class="box mb-4">
                                        <h2 class="subtitle is-6 mb-2">{kind_name}</h2>
                                        <table class="table is-fullwidth is-hoverable">
                                            <thead>
                                                <tr>
                                                    <th>"文書番号"</th><th>"Rev."</th><th>"タイトル"</th>
                                                    <th>"WBSコード"</th><th>"作成者"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {docs.into_iter().map(|doc| {
                                                    let detail_url = format!("/documents/{}", doc.id);
                                                    view! {
                                                        <tr>
                                                            <td><span class="has-text-weight-semibold">{doc.doc_number}</span></td>
                                                            <td>{doc.revision.to_string()}</td>
                                                            <td><a href=detail_url>{doc.title}</a></td>
                                                            <td>{doc.project.wbs_code.unwrap_or_default()}</td>
                                                            <td>{doc.author.name}</td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    </div>
                                }
                            }).collect_view();

                            view! {
                                <div>
                                    {tables}
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
