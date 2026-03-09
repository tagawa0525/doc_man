use leptos::prelude::*;
use uuid::Uuid;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::types::*;
use crate::auth::AuthContext;
use crate::components::form::FormField;
use crate::components::loading::Loading;
use crate::components::toast::ToastContext;
use crate::components::tree_view::TreeView;

#[component]
pub fn DepartmentsPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let toast = expect_context::<ToastContext>();
    let refresh = RwSignal::new(0u32);
    let selected_id = RwSignal::new(Option::<Uuid>::None);
    let show_form = RwSignal::new(false);

    let form_code = RwSignal::new(String::new());
    let form_name = RwSignal::new(String::new());
    let form_parent_id = RwSignal::new(String::new());
    let form_effective_from = RwSignal::new(String::new());
    let saving = RwSignal::new(false);

    let is_admin = auth.role().is_some_and(|r| r.is_admin());

    let resource = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::departments::list_include_inactive().await }
    });

    let selected_dept = LocalResource::new(move || {
        let id = selected_id.get();
        async move {
            match id {
                Some(id) => api::departments::get(id).await.ok(),
                None => None,
            }
        }
    });

    let reset_form = move || {
        form_code.set(String::new());
        form_name.set(String::new());
        form_parent_id.set(String::new());
        form_effective_from.set(String::new());
        show_form.set(false);
    };

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let code = form_code.get_untracked();
        let name = form_name.get_untracked();
        let ef = form_effective_from.get_untracked();

        if code.is_empty() || name.is_empty() || ef.is_empty() {
            toast.error("コード、名前、有効開始日は必須です");
            return;
        }

        let effective_from = match chrono::NaiveDate::parse_from_str(&ef, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                toast.error("日付形式が不正です");
                return;
            }
        };

        let parent_id = {
            let pid = form_parent_id.get_untracked();
            if pid.is_empty() {
                None
            } else {
                Uuid::parse_str(&pid).ok()
            }
        };

        saving.set(true);
        leptos::task::spawn_local(async move {
            match api::departments::create(&CreateDepartmentRequest {
                code,
                name,
                parent_id,
                effective_from,
            })
            .await
            {
                Ok(_) => {
                    toast.success("部署を作成しました");
                    reset_form();
                    refresh.update(|v| *v += 1);
                }
                Err(e) => toast.error(format!("作成失敗: {}", e.message)),
            }
            saving.set(false);
        });
    };

    view! {
        <div>
            <div class="level">
                <div class="level-left"><h1 class="title">"部署管理"</h1></div>
                {if is_admin {
                    view! {
                        <div class="level-right">
                            <button class="button is-primary" on:click=move |_| { reset_form(); show_form.set(true); }>
                                <span class="icon"><i class="fas fa-plus"></i></span>
                                <span>"新規作成"</span>
                            </button>
                        </div>
                    }.into_any()
                } else { view! { <div></div> }.into_any() }}
            </div>

            {move || if show_form.get() {
                view! {
                    <div class="box mb-5">
                        <h2 class="subtitle">"新規部署"</h2>
                        <form on:submit=on_create>
                            <div class="columns">
                                <div class="column">
                                    <FormField label="コード">
                                        <input class="input" type="text" prop:value=move || form_code.get()
                                            on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_code.set(t.value()); } />
                                    </FormField>
                                </div>
                                <div class="column">
                                    <FormField label="名前">
                                        <input class="input" type="text" prop:value=move || form_name.get()
                                            on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_name.set(t.value()); } />
                                    </FormField>
                                </div>
                                <div class="column">
                                    <FormField label="有効開始日">
                                        <input class="input" type="date" prop:value=move || form_effective_from.get()
                                            on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_effective_from.set(t.value()); } />
                                    </FormField>
                                </div>
                            </div>
                            <div class="field is-grouped">
                                <div class="control"><button class="button is-primary" type="submit" prop:disabled=move || saving.get()>"保存"</button></div>
                                <div class="control"><button class="button" type="button" on:click=move |_| reset_form()>"キャンセル"</button></div>
                            </div>
                        </form>
                    </div>
                }.into_any()
            } else { view! { <div></div> }.into_any() }}

            <div class="columns">
                <div class="column is-5">
                    <div class="box">
                        <h2 class="subtitle">"部署ツリー"</h2>
                        <Suspense fallback=move || view! { <Loading /> }>
                            {move || {
                                resource.get().map(|result| match result {
                                    Ok(depts) => {
                                        let sel = selected_id.get();
                                        view! {
                                            <TreeView departments=depts on_select=Callback::new(move |id| selected_id.set(Some(id))) selected_id=sel />
                                        }.into_any()
                                    }
                                    Err(e) => view! { <div class="notification is-danger">{e.message}</div> }.into_any(),
                                })
                            }}
                        </Suspense>
                    </div>
                </div>
                <div class="column is-7">
                    <div class="box">
                        <h2 class="subtitle">"部署詳細"</h2>
                        <Suspense fallback=move || view! { <Loading /> }>
                            {move || {
                                selected_dept.get().map(|dept| match dept {
                                    Some(d) => view! {
                                        <table class="table is-fullwidth">
                                            <tbody>
                                                <tr><th>"コード"</th><td>{d.code}</td></tr>
                                                <tr><th>"名前"</th><td>{d.name}</td></tr>
                                                <tr><th>"有効開始日"</th><td>{d.effective_from.to_string()}</td></tr>
                                                <tr><th>"有効終了日"</th><td>{d.effective_to.map(|d| d.to_string()).unwrap_or_else(|| "-".to_string())}</td></tr>
                                                <tr><th>"ID"</th><td class="is-size-7 has-text-grey">{d.id.to_string()}</td></tr>
                                            </tbody>
                                        </table>
                                    }.into_any(),
                                    None => view! { <p class="has-text-grey">"左のツリーから部署を選択してください"</p> }.into_any(),
                                })
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>
        </div>
    }
}
