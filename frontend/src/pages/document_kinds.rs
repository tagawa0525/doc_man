use leptos::prelude::*;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::types::*;
use crate::auth::AuthContext;
use crate::components::form::FormField;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;
use crate::components::toast::ToastContext;

#[component]
pub fn DocumentKindsPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let toast = expect_context::<ToastContext>();
    let page = RwSignal::new(1u32);
    let refresh = RwSignal::new(0u32);
    let show_form = RwSignal::new(false);
    let edit_id = RwSignal::new(Option::<uuid::Uuid>::None);

    let form_code = RwSignal::new(String::new());
    let form_name = RwSignal::new(String::new());
    let form_seq_digits = RwSignal::new("2".to_string());
    let saving = RwSignal::new(false);

    let is_admin = auth.role().map_or(false, |r| r.is_admin());

    let resource = LocalResource::new(
        move || {
            let p = page.get();
            let _ = refresh.get();
            async move { api::document_kinds::list(p, 20).await }
        },
    );

    let reset_form = move || {
        form_code.set(String::new());
        form_name.set(String::new());
        form_seq_digits.set("2".to_string());
        edit_id.set(None);
        show_form.set(false);
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let code = form_code.get_untracked();
        let name = form_name.get_untracked();
        let seq: i32 = form_seq_digits.get_untracked().parse().unwrap_or(2);

        if code.is_empty() || name.is_empty() {
            toast.error("コードと名前は必須です");
            return;
        }

        saving.set(true);
        let eid = edit_id.get_untracked();

        leptos::task::spawn_local(async move {
            let result = if let Some(id) = eid {
                api::document_kinds::update(id, &UpdateDocumentKindRequest {
                    code: Some(code), name: Some(name), seq_digits: Some(seq),
                }).await
            } else {
                api::document_kinds::create(&CreateDocumentKindRequest {
                    code, name, seq_digits: seq,
                }).await
            };

            match result {
                Ok(_) => {
                    toast.success(if eid.is_some() { "更新しました" } else { "作成しました" });
                    reset_form();
                    refresh.update(|v| *v += 1);
                }
                Err(e) => toast.error(format!("失敗しました: {}", e.message)),
            }
            saving.set(false);
        });
    };

    let start_edit = move |dk: DocumentKindResponse| {
        form_code.set(dk.code);
        form_name.set(dk.name);
        form_seq_digits.set(dk.seq_digits.to_string());
        edit_id.set(Some(dk.id));
        show_form.set(true);
    };

    view! {
        <div>
            <div class="level">
                <div class="level-left">
                    <h1 class="title">"文書種別管理"</h1>
                </div>
                {if is_admin {
                    view! {
                        <div class="level-right">
                            <button class="button is-primary" on:click=move |_| {
                                reset_form();
                                show_form.set(true);
                            }>
                                <span class="icon"><i class="fas fa-plus"></i></span>
                                <span>"新規作成"</span>
                            </button>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}
            </div>

            {move || if show_form.get() {
                view! {
                    <div class="box mb-5">
                        <h2 class="subtitle">{move || if edit_id.get().is_some() { "文書種別を編集" } else { "新規文書種別" }}</h2>
                        <form on:submit=on_submit>
                            <div class="columns">
                                <div class="column">
                                    <FormField label="コード">
                                        <input class="input" type="text"
                                            prop:value=move || form_code.get()
                                            on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_code.set(t.value()); }
                                        />
                                    </FormField>
                                </div>
                                <div class="column">
                                    <FormField label="名前">
                                        <input class="input" type="text"
                                            prop:value=move || form_name.get()
                                            on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_name.set(t.value()); }
                                        />
                                    </FormField>
                                </div>
                                <div class="column is-narrow">
                                    <FormField label="連番桁数">
                                        <div class="select">
                                            <select
                                                prop:value=move || form_seq_digits.get()
                                                on:change=move |ev| { let t: HtmlInputElement = event_target(&ev); form_seq_digits.set(t.value()); }
                                            >
                                                <option value="2">"2桁"</option>
                                                <option value="3">"3桁"</option>
                                            </select>
                                        </div>
                                    </FormField>
                                </div>
                            </div>
                            <div class="field is-grouped">
                                <div class="control">
                                    <button class="button is-primary" type="submit" prop:disabled=move || saving.get()>"保存"</button>
                                </div>
                                <div class="control">
                                    <button class="button" type="button" on:click=move |_| reset_form()>"キャンセル"</button>
                                </div>
                            </div>
                        </form>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    resource.get().map(|result| match result {
                        Ok(paginated) => {
                            let total = paginated.meta.total;
                            let current_page = paginated.meta.page;
                            let per_page = paginated.meta.per_page;
                            view! {
                                <div class="box">
                                    <table class="table is-fullwidth is-hoverable">
                                        <thead>
                                            <tr><th>"コード"</th><th>"名前"</th><th>"連番桁数"</th>{if is_admin { view! { <th>"操作"</th> }.into_any() } else { view! { <th></th> }.into_any() }}</tr>
                                        </thead>
                                        <tbody>
                                            {paginated.data.into_iter().map(|dk| {
                                                let dk_clone = dk.clone();
                                                view! {
                                                    <tr>
                                                        <td><span class="tag is-light">{dk.code}</span></td>
                                                        <td>{dk.name}</td>
                                                        <td>{dk.seq_digits.to_string()}"桁"</td>
                                                        <td>
                                                            {if is_admin {
                                                                view! {
                                                                    <button class="button is-small is-info is-outlined" on:click=move |_| start_edit(dk_clone.clone())>
                                                                        <span class="icon"><i class="fas fa-edit"></i></span>
                                                                    </button>
                                                                }.into_any()
                                                            } else {
                                                                view! { <span></span> }.into_any()
                                                            }}
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                    <Pagination current_page=current_page total=total per_page=per_page on_page_change=Callback::new(move |p| page.set(p)) />
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
