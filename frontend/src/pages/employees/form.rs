use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::types::{flatten_dept_tree, CreateEmployeeRequest, UpdateEmployeeRequest};
use crate::components::form::FormField;
use crate::components::toast::ToastContext;

#[component]
pub fn EmployeeFormPage() -> impl IntoView {
    let toast = expect_context::<ToastContext>();
    let params = use_params_map();

    let employee_id = move || {
        params
            .read()
            .get("id")
            .and_then(|id| Uuid::parse_str(&id).ok())
    };

    let is_edit = move || employee_id().is_some();

    let form_name = RwSignal::new(String::new());
    let form_employee_code = RwSignal::new(String::new());
    let form_ad_account = RwSignal::new(String::new());
    let form_role = RwSignal::new("general".to_string());
    let form_dept_id = RwSignal::new(String::new());
    let form_effective_from = RwSignal::new(String::new());
    let form_is_active = RwSignal::new(true);
    let saving = RwSignal::new(false);
    let loaded = RwSignal::new(false);

    let depts_resource = LocalResource::new(|| async { api::departments::list().await });

    // Load existing employee for edit
    let _load_effect = Effect::new(move || {
        if let Some(id) = employee_id() {
            if !loaded.get_untracked() {
                leptos::task::spawn_local(async move {
                    if let Ok(emp) = api::employees::get(id).await {
                        form_name.set(emp.name);
                        form_employee_code.set(emp.employee_code.unwrap_or_default());
                        form_ad_account.set(emp.ad_account.unwrap_or_default());
                        form_role.set(emp.role);
                        form_is_active.set(emp.is_active);
                        if let Some(dept) = emp.current_department {
                            form_dept_id.set(dept.id.to_string());
                        }
                        loaded.set(true);
                    }
                });
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = form_name.get_untracked();
        if name.is_empty() {
            toast.error("名前は必須です");
            return;
        }

        saving.set(true);
        let eid = employee_id();

        leptos::task::spawn_local(async move {
            let result = if let Some(id) = eid {
                let ad = form_ad_account.get_untracked();
                let role = form_role.get_untracked();
                let active = form_is_active.get_untracked();
                api::employees::update(
                    id,
                    &UpdateEmployeeRequest {
                        name: Some(name),
                        ad_account: if ad.is_empty() { None } else { Some(ad) },
                        role: Some(role),
                        is_active: Some(active),
                    },
                )
                .await
                .map(|_| ())
            } else {
                let ec = form_employee_code.get_untracked();
                let ad = form_ad_account.get_untracked();
                let role = form_role.get_untracked();
                let dept_str = form_dept_id.get_untracked();
                let ef = form_effective_from.get_untracked();

                if dept_str.is_empty() || ef.is_empty() {
                    toast.error("部署と有効開始日は必須です");
                    saving.set(false);
                    return;
                }

                let Ok(department_id) = Uuid::parse_str(&dept_str) else {
                    toast.error("部署を選択してください");
                    saving.set(false);
                    return;
                };
                let Ok(effective_from) = chrono::NaiveDate::parse_from_str(&ef, "%Y-%m-%d") else {
                    toast.error("日付形式が不正です");
                    saving.set(false);
                    return;
                };

                api::employees::create(&CreateEmployeeRequest {
                    name,
                    employee_code: if ec.is_empty() { None } else { Some(ec) },
                    ad_account: if ad.is_empty() { None } else { Some(ad) },
                    role: Some(role),
                    department_id,
                    effective_from,
                })
                .await
                .map(|_| ())
            };

            match result {
                Ok(()) => {
                    toast.success(if eid.is_some() {
                        "更新しました"
                    } else {
                        "作成しました"
                    });
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/employees");
                    }
                }
                Err(e) => toast.error(format!("失敗: {}", e.message)),
            }
            saving.set(false);
        });
    };

    view! {
        <div>
            <h1 class="title">{move || if is_edit() { "社員編集" } else { "社員作成" }}</h1>

            <div class="box">
                <form on:submit=on_submit>
                    <div class="columns">
                        <div class="column">
                            <FormField label="名前 *">
                                <input class="input" type="text" prop:value=move || form_name.get()
                                    on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_name.set(t.value()); } />
                            </FormField>
                        </div>
                        <div class="column">
                            <FormField label="社員コード">
                                <input class="input" type="text" prop:value=move || form_employee_code.get()
                                    prop:disabled=is_edit
                                    on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_employee_code.set(t.value()); } />
                            </FormField>
                        </div>
                    </div>
                    <div class="columns">
                        <div class="column">
                            <FormField label="ADアカウント">
                                <input class="input" type="text" prop:value=move || form_ad_account.get()
                                    on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_ad_account.set(t.value()); } />
                            </FormField>
                        </div>
                        <div class="column">
                            <FormField label="ロール">
                                <div class="select is-fullwidth">
                                    <select prop:value=move || form_role.get()
                                        on:change=move |ev| form_role.set(event_target_value(&ev))>
                                        <option value="admin">"管理者"</option>
                                        <option value="project_manager">"プロジェクトマネージャー"</option>
                                        <option value="general">"一般"</option>
                                        <option value="viewer">"閲覧者"</option>
                                    </select>
                                </div>
                            </FormField>
                        </div>
                    </div>

                    {move || if is_edit() {
                        view! {
                            <FormField label="状態">
                                <label class="checkbox">
                                    <input type="checkbox" prop:checked=move || form_is_active.get()
                                        on:change=move |ev| { let t: HtmlInputElement = event_target(&ev); form_is_active.set(t.checked()); } />
                                    " 有効"
                                </label>
                            </FormField>
                        }.into_any()
                    } else {
                        let dept_options = depts_resource.get().and_then(std::result::Result::ok).map(|depts| {
                            let mut opts = Vec::new();
                            flatten_dept_tree(&depts, &mut opts, "");
                            opts
                        }).unwrap_or_default();

                        view! {
                            <div class="columns">
                                <div class="column">
                                    <FormField label="部署 *">
                                        <div class="select is-fullwidth">
                                            <select prop:value=move || form_dept_id.get()
                                                on:change=move |ev| form_dept_id.set(event_target_value(&ev))>
                                                <option value="">"-- 選択 --"</option>
                                                {dept_options.into_iter().map(|(val, label)| {
                                                    view! { <option value=val>{label}</option> }
                                                }).collect_view()}
                                            </select>
                                        </div>
                                    </FormField>
                                </div>
                                <div class="column">
                                    <FormField label="有効開始日 *">
                                        <input class="input" type="date" prop:value=move || form_effective_from.get()
                                            on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_effective_from.set(t.value()); } />
                                    </FormField>
                                </div>
                            </div>
                        }.into_any()
                    }}

                    <div class="field is-grouped">
                        <div class="control">
                            <button class="button is-primary" type="submit" prop:disabled=move || saving.get()>"保存"</button>
                        </div>
                        <div class="control">
                            <a href="/employees" class="button">"戻る"</a>
                        </div>
                    </div>
                </form>
            </div>
        </div>
    }
}
