use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::documents::DocumentListParams;
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;
use crate::components::status_badge::StatusBadge;

fn current_fiscal_year() -> i32 {
    let now = chrono::Utc::now();
    let year = now.format("%Y").to_string().parse::<i32>().unwrap_or(2025);
    let month = now.format("%m").to_string().parse::<u32>().unwrap_or(4);
    if month < 4 { year - 1 } else { year }
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

    let fiscal_year = current_fiscal_year();
    let selected_fiscal_year = RwSignal::new(fiscal_year.to_string());
    let selected_doc_kind = RwSignal::new(String::new());

    // dept_codes は初期値空（auth ロード後に Effect で設定）
    let dept_codes = RwSignal::new(String::new());
    let dept_codes_initialized = RwSignal::new(false);

    // auth ロード完了時にデフォルト部署コードを設定
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

    // 文書種別リスト
    let doc_kinds = LocalResource::new(|| async { api::document_kinds::list_all().await });

    let resource = LocalResource::new(move || {
        let p = page.get();
        let q = search_query.get();
        let dc = dept_codes.get();
        let dk = selected_doc_kind.get();
        let fy = selected_fiscal_year.get();
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
                fiscal_year: fy,
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

    // 年度リスト
    let years: Vec<i32> = ((fiscal_year - 3)..=(fiscal_year + 1)).rev().collect();

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

            // フィルタバー 1行目
            <div class="columns is-multiline mb-2">
                // 部署チェックボックス
                <div class="column is-narrow">
                    <label class="label is-small">"部署"</label>
                    <div class="field is-grouped">
                        {move || {
                            let depts = auth.user.get().map(|u| u.departments).unwrap_or_default();
                            depts.into_iter().map(|d| {
                                let code = d.code.clone();
                                let name = d.name.clone();
                                let code_for_check = code.clone();
                                let code_for_handler = code.clone();
                                view! {
                                    <label class="checkbox mr-3">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || {
                                                let codes = dept_codes.get();
                                                codes.split(',').any(|c| c == code_for_check)
                                            }
                                            on:change=move |_| {
                                                let current = dept_codes.get_untracked();
                                                let mut codes: Vec<&str> = current.split(',').filter(|c| !c.is_empty()).collect();
                                                if codes.contains(&code_for_handler.as_str()) {
                                                    codes.retain(|c| *c != code_for_handler.as_str());
                                                } else {
                                                    codes.push(&code_for_handler);
                                                }
                                                page.set(1);
                                                dept_codes.set(codes.join(","));
                                            }
                                        />
                                        " " {name}
                                    </label>
                                }
                            }).collect_view()
                        }}
                    </div>
                </div>

                // 文書種別
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

                // タイトル・文書番号
                <div class="column">
                    <label class="label is-small">"タイトル・文書番号"</label>
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
