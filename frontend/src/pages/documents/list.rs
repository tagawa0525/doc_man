use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::documents::DocumentListParams;
use crate::api::types::{FlatDepartment, flatten_dept_tree_full};
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;
use crate::components::status_badge::StatusBadge;

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
    let page = RwSignal::new(1u32);
    let search_query = RwSignal::new(String::new());
    let project_name = RwSignal::new(String::new());
    let author_name = RwSignal::new(String::new());
    let wbs_code = RwSignal::new(String::new());
    let timer_id = RwSignal::new(0i32);
    let selected_doc_kind = RwSignal::new(String::new());
    let show_detail = RwSignal::new(false);

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
        let dk = selected_doc_kind.get();
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
                doc_kind_id: dk,
                fiscal_years: fy,
                project_name: pn,
                author_name: an,
                wbs_code: wc,
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
    let on_project_name = make_debounced_handler(project_name);
    let on_author_name = make_debounced_handler(author_name);
    let on_wbs_code = make_debounced_handler(wbs_code);

    let user_dept_codes = Memo::new(move |_| {
        auth.user
            .get()
            .map(|u| u.departments.iter().map(|d| d.code.clone()).collect::<Vec<_>>())
            .unwrap_or_default()
    });

    let default_year_strings: Vec<String> = default_years.iter().map(i32::to_string).collect();

    view! {
        <div>
            <div class="level">
                <div class="level-left"><h1 class="title">"文書一覧"</h1></div>
                <div class="level-right">
                    {if can_create {
                        view! {
                            <a href="/documents/new" class="button is-primary mr-2">
                                <span class="icon"><i class="fas fa-plus"></i></span>
                                <span>"新規作成"</span>
                            </a>
                        }.into_any()
                    } else { view! { <span></span> }.into_any() }}
                    <button
                        class="button is-small is-outlined"
                        on:click=move |_| show_detail.update(|v| *v = !*v)
                    >
                        <span class="icon"><i class=move || if show_detail.get() { "fas fa-chevron-up" } else { "fas fa-chevron-down" }></i></span>
                        <span>{move || if show_detail.get() { "簡易表示" } else { "詳細フィルタ" }}</span>
                    </button>
                </div>
            </div>

            // 部署チェックボックス
            <div class="field mb-3">
                <label class="label is-small">"部署"</label>
                <div class="is-flex is-flex-wrap-wrap" style="gap: 0.25rem 0.75rem;">
                    <Suspense fallback=|| ()>
                        {move || all_depts.get().map(|result| match result {
                            Ok(tree) => {
                                let mut flat: Vec<FlatDepartment> = Vec::new();
                                flatten_dept_tree_full(&tree, &mut flat, "");
                                let detail = show_detail.get();
                                let udc = user_dept_codes.get();
                                flat.into_iter().filter_map(move |d| {
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
                                }).collect_view().into_any()
                            }
                            Err(_) => view! { <span class="tag is-warning">"部署読込失敗"</span> }.into_any(),
                        })}
                    </Suspense>
                </div>
            </div>

            // 年度チェックボックス + 文書種別 + テキスト検索
            <div class="columns is-multiline mb-2">
                <div class="column is-narrow">
                    <label class="label is-small">"年度"</label>
                    <div class="is-flex is-flex-wrap-wrap" style="gap: 0.25rem 0.5rem;">
                        {move || {
                            let detail = show_detail.get();
                            let dys = default_year_strings.clone();
                            all_years.iter().filter_map(move |&y| {
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
                            }).collect_view()
                        }}
                    </div>
                </div>

                <div class="column is-narrow">
                    <label class="label is-small">"文書種別"</label>
                    <div class="select is-small">
                        <select on:change=move |ev| {
                            let val = event_target::<web_sys::HtmlSelectElement>(&ev).value();
                            page.set(1);
                            selected_doc_kind.set(val);
                        }>
                            <option value="">"全て"</option>
                            <Suspense fallback=|| ()>
                                {move || doc_kinds.get().map(|result| match result {
                                    Ok(paginated) => {
                                        paginated.data.into_iter().map(|dk| {
                                            let id = dk.id.to_string();
                                            view! { <option value=id>{dk.name}</option> }
                                        }).collect_view().into_any()
                                    }
                                    Err(_) => view! { <option>"読込失敗"</option> }.into_any(),
                                })}
                            </Suspense>
                        </select>
                    </div>
                </div>

                <div class="column">
                    <label class="label is-small">"タイトル・文書番号"</label>
                    <div class="control has-icons-left">
                        <input class="input is-small" type="text" placeholder="検索..." on:input=on_search />
                        <span class="icon is-left"><i class="fas fa-search"></i></span>
                    </div>
                </div>
            </div>

            // フィルタバー 2行目
            <div class="columns mb-4">
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
                    <input class="input is-small" type="text" placeholder="部分一致..." on:input=on_wbs_code />
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
