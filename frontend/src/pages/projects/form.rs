use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::types::{CreateProjectRequest, UpdateProjectRequest};
use crate::auth::AuthContext;
use crate::components::form::FormField;
use crate::components::modal::ConfirmModal;
use crate::components::toast::ToastContext;

#[component]
pub fn ProjectFormPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let toast = expect_context::<ToastContext>();
    let params = use_params_map();
    let is_admin = auth.role().is_some_and(|r| r.is_admin());

    let project_id = move || {
        params
            .read()
            .get("id")
            .and_then(|id| Uuid::parse_str(&id).ok())
    };
    let is_edit = move || project_id().is_some();

    let form_name = RwSignal::new(String::new());
    let form_status = RwSignal::new("active".to_string());
    let form_start_date = RwSignal::new(String::new());
    let form_end_date = RwSignal::new(String::new());
    let form_wbs_code = RwSignal::new(String::new());
    let form_discipline_id = RwSignal::new(String::new());
    let form_manager_id = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    let loaded = RwSignal::new(false);
    let delete_target = RwSignal::new(Option::<(Uuid, String)>::None);

    let do_delete = move |id: Uuid| {
        leptos::task::spawn_local(async move {
            match api::projects::delete(id).await {
                Ok(()) => {
                    toast.success("削除しました");
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/projects");
                    }
                }
                Err(e) => toast.error(format!("削除失敗: {}", e.message)),
            }
            delete_target.set(None);
        });
    };

    let disciplines_resource = LocalResource::new(|| async { api::disciplines::list_all().await });
    let employees_resource = LocalResource::new(|| async { api::employees::list_active().await });

    let _load_effect = Effect::new(move || {
        if let Some(id) = project_id() {
            if !loaded.get_untracked() {
                leptos::task::spawn_local(async move {
                    if let Ok(p) = api::projects::get(id).await {
                        form_name.set(p.name);
                        form_status.set(p.status);
                        form_start_date.set(p.start_date.to_string());
                        form_end_date.set(p.end_date.map(|d| d.to_string()).unwrap_or_default());
                        form_wbs_code.set(p.wbs_code.unwrap_or_default());
                        form_discipline_id.set(p.discipline.id.to_string());
                        form_manager_id
                            .set(p.manager.map(|m| m.id.to_string()).unwrap_or_default());
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
        let eid = project_id();
        let status = form_status.get_untracked();
        let sd = form_start_date.get_untracked();
        let ed = form_end_date.get_untracked();
        let wbs = form_wbs_code.get_untracked();
        let did = form_discipline_id.get_untracked();
        let mid = form_manager_id.get_untracked();

        leptos::task::spawn_local(async move {
            let parse_date = |s: &str| -> Option<chrono::NaiveDate> {
                if s.is_empty() {
                    None
                } else {
                    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
                }
            };

            let result = if let Some(id) = eid {
                api::projects::update(
                    id,
                    &UpdateProjectRequest {
                        name: Some(name),
                        status: Some(status),
                        start_date: parse_date(&sd),
                        end_date: parse_date(&ed),
                        wbs_code: if wbs.is_empty() { None } else { Some(wbs) },
                        discipline_id: Uuid::parse_str(&did).ok(),
                        manager_id: if mid.is_empty() {
                            None
                        } else {
                            Uuid::parse_str(&mid).ok()
                        },
                    },
                )
                .await
                .map(|_| ())
            } else {
                if did.is_empty() {
                    toast.error("専門分野は必須です");
                    saving.set(false);
                    return;
                }
                let Some(start_date) = parse_date(&sd) else {
                    toast.error("開始日は必須です");
                    saving.set(false);
                    return;
                };
                api::projects::create(&CreateProjectRequest {
                    name,
                    status: if status.is_empty() {
                        None
                    } else {
                        Some(status)
                    },
                    start_date,
                    end_date: parse_date(&ed),
                    wbs_code: if wbs.is_empty() { None } else { Some(wbs) },
                    discipline_id: Uuid::parse_str(&did).unwrap(),
                    manager_id: if mid.is_empty() {
                        None
                    } else {
                        Uuid::parse_str(&mid).ok()
                    },
                })
                .await
                .map(|_| ())
            };

            match result {
                Ok(()) => {
                    toast.success("保存しました");
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/projects");
                    }
                }
                Err(e) => toast.error(format!("失敗: {}", e.message)),
            }
            saving.set(false);
        });
    };

    view! {
        <div>
            <h1 class="title">{move || if is_edit() { "プロジェクト編集" } else { "プロジェクト作成" }}</h1>

            {move || delete_target.get().map(|(id, name)| view! {
                <ConfirmModal
                    title="プロジェクト削除"
                    message=format!("「{}」を削除しますか？この操作は取り消せません。", name)
                    on_confirm=Callback::new(move |()| do_delete(id))
                    on_cancel=Callback::new(move |()| delete_target.set(None))
                    danger=true
                />
            })}

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
                            <FormField label="ステータス">
                                <div class="select is-fullwidth">
                                    <select prop:value=move || form_status.get()
                                        on:change=move |ev| form_status.set(event_target_value(&ev))>
                                        <option value="active">"進行中"</option>
                                        <option value="completed">"完了"</option>
                                        <option value="suspended">"中断"</option>
                                    </select>
                                </div>
                            </FormField>
                        </div>
                    </div>
                    <div class="columns">
                        <div class="column">
                            <FormField label="開始日 *">
                                <input class="input" type="date" prop:value=move || form_start_date.get()
                                    on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_start_date.set(t.value()); } />
                            </FormField>
                        </div>
                        <div class="column">
                            <FormField label="終了日">
                                <input class="input" type="date" prop:value=move || form_end_date.get()
                                    on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_end_date.set(t.value()); } />
                            </FormField>
                        </div>
                        <div class="column">
                            <FormField label="WBSコード">
                                <input class="input" type="text" prop:value=move || form_wbs_code.get()
                                    on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_wbs_code.set(t.value()); } />
                            </FormField>
                        </div>
                    </div>
                    <div class="columns">
                        <div class="column">
                            <FormField label="専門分野 *">
                                <div class="select is-fullwidth">
                                    <select prop:value=move || form_discipline_id.get()
                                        on:change=move |ev| form_discipline_id.set(event_target_value(&ev))>
                                        <option value="">"-- 選択 --"</option>
                                        {move || disciplines_resource.get().and_then(std::result::Result::ok).map(|p| {
                                            p.data.into_iter().map(|d| {
                                                view! { <option value=d.id.to_string()>{format!("{} ({})", d.name, d.code)}</option> }
                                            }).collect_view()
                                        })}
                                    </select>
                                </div>
                            </FormField>
                        </div>
                        <div class="column">
                            <FormField label="マネージャー">
                                <div class="select is-fullwidth">
                                    <select prop:value=move || form_manager_id.get()
                                        on:change=move |ev| form_manager_id.set(event_target_value(&ev))>
                                        <option value="">"-- なし --"</option>
                                        {move || employees_resource.get().and_then(std::result::Result::ok).map(|p| {
                                            p.data.into_iter().map(|e| {
                                                view! { <option value=e.id.to_string()>{e.name}</option> }
                                            }).collect_view()
                                        })}
                                    </select>
                                </div>
                            </FormField>
                        </div>
                    </div>
                    <div class="field is-grouped">
                        <div class="control"><button class="button is-primary" type="submit" prop:disabled=move || saving.get()>"保存"</button></div>
                        <div class="control"><a href="/projects" class="button">"戻る"</a></div>
                    </div>
                </form>

                {move || {
                    if is_edit() && is_admin {
                        let name = form_name.get();
                        let id = project_id();
                        view! {
                            <div class="mt-5 pt-4" style="border-top: 1px solid #dbdbdb">
                                <button class="button is-danger is-outlined"
                                    on:click=move |_| {
                                        if let Some(id) = id {
                                            delete_target.set(Some((id, name.clone())));
                                        }
                                    }>
                                    <span class="icon"><i class="fas fa-trash"></i></span>
                                    <span>"このプロジェクトを削除"</span>
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
