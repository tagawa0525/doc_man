use leptos::prelude::*;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::types::{CreatePositionRequest, PositionResponse, UpdatePositionRequest};
use crate::auth::AuthContext;
use crate::components::form::FormField;
use crate::components::loading::Loading;
use crate::components::toast::ToastContext;

#[component]
pub fn PositionsPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let toast = expect_context::<ToastContext>();
    let refresh = RwSignal::new(0u32);
    let show_form = RwSignal::new(false);
    let edit_id = RwSignal::new(Option::<uuid::Uuid>::None);

    let form_name = RwSignal::new(String::new());
    let form_default_role = RwSignal::new(String::new());
    let form_sort_order = RwSignal::new(String::new());
    let saving = RwSignal::new(false);

    let is_admin = auth.role().is_some_and(|r| r.is_admin());

    let resource = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::positions::list().await }
    });

    let reset_form = move || {
        form_name.set(String::new());
        form_default_role.set(String::new());
        form_sort_order.set(String::new());
        edit_id.set(None);
        show_form.set(false);
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = form_name.get_untracked();
        let default_role = form_default_role.get_untracked();
        let sort_order_str = form_sort_order.get_untracked();

        if name.is_empty() || default_role.is_empty() || sort_order_str.is_empty() {
            toast.error("全項目を入力してください");
            return;
        }

        let Ok(sort_order) = sort_order_str.parse::<i32>() else {
            toast.error("表示順は数値で入力してください");
            return;
        };

        saving.set(true);
        let eid = edit_id.get_untracked();

        leptos::task::spawn_local(async move {
            let result = if let Some(id) = eid {
                api::positions::update(
                    id,
                    &UpdatePositionRequest {
                        name: Some(name),
                        default_role: Some(default_role),
                        sort_order: Some(sort_order),
                    },
                )
                .await
            } else {
                api::positions::create(&CreatePositionRequest {
                    name,
                    default_role,
                    sort_order,
                })
                .await
            };

            match result {
                Ok(_) => {
                    toast.success(if eid.is_some() {
                        "更新しました"
                    } else {
                        "作成しました"
                    });
                    reset_form();
                    refresh.update(|v| *v += 1);
                }
                Err(e) => toast.error(format!("失敗: {}", e.message)),
            }
            saving.set(false);
        });
    };

    let start_edit = move |p: PositionResponse| {
        form_name.set(p.name);
        form_default_role.set(p.default_role);
        form_sort_order.set(p.sort_order.to_string());
        edit_id.set(Some(p.id));
        show_form.set(true);
    };

    let role_options = [
        ("admin", "管理者"),
        ("project_manager", "PM"),
        ("general", "一般"),
        ("viewer", "閲覧者"),
    ];

    view! {
        <div>
            <div class="level">
                <div class="level-left"><h1 class="title">"職位管理"</h1></div>
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
                        <h2 class="subtitle">{move || if edit_id.get().is_some() { "職位を編集" } else { "新規職位" }}</h2>
                        <form on:submit=on_submit>
                            <div class="columns">
                                <div class="column">
                                    <FormField label="名前">
                                        <input class="input" type="text" prop:value=move || form_name.get()
                                            on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_name.set(t.value()); } />
                                    </FormField>
                                </div>
                                <div class="column">
                                    <FormField label="デフォルトロール">
                                        <div class="select is-fullwidth">
                                            <select prop:value=move || form_default_role.get()
                                                on:change=move |ev| form_default_role.set(event_target_value(&ev))>
                                                <option value="">"-- 選択 --"</option>
                                                {role_options.iter().map(|(val, label)| {
                                                    view! { <option value=*val>{*label}</option> }
                                                }).collect_view()}
                                            </select>
                                        </div>
                                    </FormField>
                                </div>
                                <div class="column is-narrow">
                                    <FormField label="表示順">
                                        <input class="input" type="number" style="width: 100px;"
                                            prop:value=move || form_sort_order.get()
                                            on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_sort_order.set(t.value()); } />
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

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    resource.get().map(|result| match result {
                        Ok(positions) => {
                            view! {
                                <div class="box">
                                    <table class="table is-fullwidth is-hoverable">
                                        <thead><tr>
                                            <th>"表示順"</th>
                                            <th>"名前"</th>
                                            <th>"デフォルトロール"</th>
                                            {if is_admin { view! { <th>"操作"</th> }.into_any() } else { view! { <th></th> }.into_any() }}
                                        </tr></thead>
                                        <tbody>
                                            {positions.into_iter().map(|p| {
                                                let p_clone = p.clone();
                                                let role_label = match p.default_role.as_str() {
                                                    "admin" => "管理者",
                                                    "project_manager" => "PM",
                                                    "general" => "一般",
                                                    "viewer" => "閲覧者",
                                                    _ => &p.default_role,
                                                };
                                                view! {
                                                    <tr>
                                                        <td>{p.sort_order}</td>
                                                        <td>{p.name}</td>
                                                        <td><span class="tag is-light">{role_label.to_string()}</span></td>
                                                        <td>
                                                            {if is_admin {
                                                                view! { <button class="button is-small is-info is-outlined" on:click=move |_| start_edit(p_clone.clone())><span class="icon"><i class="fas fa-edit"></i></span></button> }.into_any()
                                                            } else { view! { <span></span> }.into_any() }}
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
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
