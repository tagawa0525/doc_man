use leptos::prelude::*;
use uuid::Uuid;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::types::*;
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::toast::ToastContext;

#[component]
pub fn ApprovalSection(
    doc_id: Uuid,
    #[prop(into)] doc_status: String,
    #[prop(into)] on_change: Callback<()>,
) -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let toast = expect_context::<ToastContext>();
    let refresh = RwSignal::new(0u32);
    let show_create = RwSignal::new(false);
    let comment = RwSignal::new(String::new());

    let can_manage = auth.role().map_or(false, |r| r.can_manage());
    let can_create_route = can_manage && (doc_status == "draft" || doc_status == "rejected");

    let user_id = auth.user.get_untracked().map(|u| u.id);

    let steps_resource = LocalResource::new(
        move || {
            let _ = refresh.get();
            async move { api::approval_steps::list(doc_id).await }
        },
    );

    let employees_resource = LocalResource::new(|| async { api::employees::list_active().await });

    // Simple approval route creation with up to 3 steps
    let step1_id = RwSignal::new(String::new());
    let step2_id = RwSignal::new(String::new());
    let step3_id = RwSignal::new(String::new());

    let create_route = move |_: leptos::ev::MouseEvent| {
        let mut steps = Vec::new();
        let s1 = step1_id.get_untracked();
        let s2 = step2_id.get_untracked();
        let s3 = step3_id.get_untracked();

        if let Ok(id) = Uuid::parse_str(&s1) { steps.push(StepInput { step_order: 1, approver_id: id }); }
        if let Ok(id) = Uuid::parse_str(&s2) { steps.push(StepInput { step_order: 2, approver_id: id }); }
        if let Ok(id) = Uuid::parse_str(&s3) { steps.push(StepInput { step_order: 3, approver_id: id }); }

        if steps.is_empty() {
            toast.error("少なくとも1人の承認者を選択してください");
            return;
        }

        leptos::task::spawn_local(async move {
            match api::approval_steps::create_route(doc_id, &CreateApprovalRouteRequest { steps }).await {
                Ok(_) => {
                    toast.success("承認ルートを作成しました");
                    show_create.set(false);
                    refresh.update(|v| *v += 1);
                    on_change.run(());
                }
                Err(e) => toast.error(format!("失敗: {}", e.message)),
            }
        });
    };

    let do_approve = move |step_id: Uuid| {
        let c = comment.get_untracked();
        leptos::task::spawn_local(async move {
            match api::approval_steps::approve(doc_id, step_id, &ApprovalActionRequest {
                comment: if c.is_empty() { None } else { Some(c) },
            }).await {
                Ok(_) => {
                    toast.success("承認しました");
                    comment.set(String::new());
                    refresh.update(|v| *v += 1);
                    on_change.run(());
                }
                Err(e) => toast.error(format!("承認失敗: {}", e.message)),
            }
        });
    };

    let do_reject = move |step_id: Uuid| {
        let c = comment.get_untracked();
        leptos::task::spawn_local(async move {
            match api::approval_steps::reject(doc_id, step_id, &ApprovalActionRequest {
                comment: if c.is_empty() { None } else { Some(c) },
            }).await {
                Ok(_) => {
                    toast.success("却下しました");
                    comment.set(String::new());
                    refresh.update(|v| *v += 1);
                    on_change.run(());
                }
                Err(e) => toast.error(format!("却下失敗: {}", e.message)),
            }
        });
    };

    view! {
        <div class="box mb-4">
            <h3 class="subtitle">
                <span class="icon"><i class="fas fa-check-circle"></i></span>
                " 承認"
            </h3>

            {if can_create_route {
                view! {
                    <button class="button is-small is-primary mb-3" on:click=move |_| show_create.update(|v| *v = !*v)>
                        <span class="icon"><i class="fas fa-route"></i></span>
                        <span>"承認ルート作成"</span>
                    </button>
                }.into_any()
            } else { view! { <span></span> }.into_any() }}

            {move || if show_create.get() {
                let emps = employees_resource.get().and_then(|r| r.ok()).map(|p| p.data).unwrap_or_default();
                let emps2 = emps.clone();
                let emps3 = emps.clone();
                view! {
                    <div class="notification is-light mb-3">
                        <div class="field">
                            <label class="label is-small">"承認者1"</label>
                            <div class="select is-small is-fullwidth">
                                <select on:change=move |ev| step1_id.set(event_target_value(&ev))>
                                    <option value="">"-- 選択 --"</option>
                                    {emps.into_iter().map(|e| view! { <option value=e.id.to_string()>{e.name}</option> }).collect_view()}
                                </select>
                            </div>
                        </div>
                        <div class="field">
                            <label class="label is-small">"承認者2（任意）"</label>
                            <div class="select is-small is-fullwidth">
                                <select on:change=move |ev| step2_id.set(event_target_value(&ev))>
                                    <option value="">"-- なし --"</option>
                                    {emps2.into_iter().map(|e| view! { <option value=e.id.to_string()>{e.name}</option> }).collect_view()}
                                </select>
                            </div>
                        </div>
                        <div class="field">
                            <label class="label is-small">"承認者3（任意）"</label>
                            <div class="select is-small is-fullwidth">
                                <select on:change=move |ev| step3_id.set(event_target_value(&ev))>
                                    <option value="">"-- なし --"</option>
                                    {emps3.into_iter().map(|e| view! { <option value=e.id.to_string()>{e.name}</option> }).collect_view()}
                                </select>
                            </div>
                        </div>
                        <button class="button is-primary is-small" on:click=create_route>"作成"</button>
                    </div>
                }.into_any()
            } else { view! { <div></div> }.into_any() }}

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    steps_resource.get().map(|result| match result {
                        Ok(steps) => {
                            if steps.is_empty() {
                                return view! { <p class="has-text-grey is-size-7">"承認ルートが設定されていません"</p> }.into_any();
                            }
                            view! {
                                <div>
                                    {steps.into_iter().map(|step| {
                                        let step_id = step.id;
                                        let is_mine = user_id == Some(step.approver.id);
                                        let is_pending = step.status == "pending";
                                        let can_act = is_mine && is_pending;
                                        let status_class = match step.status.as_str() {
                                            "approved" => "has-text-success",
                                            "rejected" => "has-text-danger",
                                            _ => "has-text-grey",
                                        };
                                        let status_icon = match step.status.as_str() {
                                            "approved" => "fas fa-check",
                                            "rejected" => "fas fa-times",
                                            _ => "fas fa-clock",
                                        };
                                        view! {
                                            <div class="mb-2 p-2" style="border-left: 3px solid #dbdbdb; margin-left: 0.5rem;">
                                                <div class="is-flex is-justify-content-space-between is-align-items-center">
                                                    <span>
                                                        <span class="has-text-weight-semibold">{format!("{}. ", step.step_order)}</span>
                                                        {step.approver.name}
                                                    </span>
                                                    <span class=status_class>
                                                        <span class="icon is-small"><i class=status_icon></i></span>
                                                    </span>
                                                </div>
                                                {step.comment.map(|c| view! { <p class="is-size-7 has-text-grey ml-3">{format!("\"{}\"", c)}</p> }.into_any()).unwrap_or_else(|| view! { <span></span> }.into_any())}
                                                {if can_act {
                                                    view! {
                                                        <div class="mt-1 ml-3">
                                                            <input class="input is-small mb-1" type="text" placeholder="コメント（任意）"
                                                                prop:value=move || comment.get()
                                                                on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); comment.set(t.value()); } />
                                                            <div class="buttons are-small">
                                                                <button class="button is-success is-small" on:click=move |_| do_approve(step_id)>
                                                                    <span class="icon"><i class="fas fa-check"></i></span>
                                                                    <span>"承認"</span>
                                                                </button>
                                                                <button class="button is-danger is-small" on:click=move |_| do_reject(step_id)>
                                                                    <span class="icon"><i class="fas fa-times"></i></span>
                                                                    <span>"却下"</span>
                                                                </button>
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                } else { view! { <span></span> }.into_any() }}
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! { <p class="has-text-danger is-size-7">{e.message}</p> }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
