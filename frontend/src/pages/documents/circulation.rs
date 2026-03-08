use leptos::prelude::*;
use uuid::Uuid;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::types::CreateCirculationRequest;
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::toast::ToastContext;

#[component]
pub fn CirculationSection(
    doc_id: Uuid,
    #[prop(into)] doc_status: String,
    #[prop(into)] on_change: Callback<()>,
) -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let toast = expect_context::<ToastContext>();
    let refresh = RwSignal::new(0u32);
    let show_create = RwSignal::new(false);
    let selected_ids = RwSignal::new(Vec::<String>::new());

    let can_manage = auth.role().map_or(false, |r| r.can_manage());
    let can_create = can_manage && doc_status == "approved";
    let user_id = auth.user.get_untracked().map(|u| u.id);

    let circ_resource = LocalResource::new(
        move || {
            let _ = refresh.get();
            async move { api::circulations::list(doc_id).await }
        },
    );

    let employees_resource = LocalResource::new(|| async { api::employees::list_active().await });

    let create_circulations = move |_: leptos::ev::MouseEvent| {
        let ids: Vec<Uuid> = selected_ids.get_untracked().iter()
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect();

        if ids.is_empty() {
            toast.error("少なくとも1人の回覧先を選択してください");
            return;
        }

        leptos::task::spawn_local(async move {
            match api::circulations::create(doc_id, &CreateCirculationRequest { recipient_ids: ids }).await {
                Ok(_) => {
                    toast.success("回覧を開始しました");
                    show_create.set(false);
                    selected_ids.set(Vec::new());
                    refresh.update(|v| *v += 1);
                    on_change.run(());
                }
                Err(e) => toast.error(format!("失敗: {}", e.message)),
            }
        });
    };

    let do_confirm = move |_: leptos::ev::MouseEvent| {
        leptos::task::spawn_local(async move {
            match api::circulations::confirm(doc_id).await {
                Ok(_) => {
                    toast.success("確認しました");
                    refresh.update(|v| *v += 1);
                    on_change.run(());
                }
                Err(e) => toast.error(format!("確認失敗: {}", e.message)),
            }
        });
    };

    view! {
        <div class="box">
            <h3 class="subtitle">
                <span class="icon"><i class="fas fa-share-alt"></i></span>
                " 回覧"
            </h3>

            {if can_create {
                view! {
                    <button class="button is-small is-primary mb-3" on:click=move |_| show_create.update(|v| *v = !*v)>
                        <span class="icon"><i class="fas fa-paper-plane"></i></span>
                        <span>"回覧開始"</span>
                    </button>
                }.into_any()
            } else { view! { <span></span> }.into_any() }}

            {move || if show_create.get() {
                let emps = employees_resource.get().and_then(|r| r.ok()).map(|p| p.data).unwrap_or_default();
                view! {
                    <div class="notification is-light mb-3">
                        <label class="label is-small">"回覧先を選択（複数可）"</label>
                        <div style="max-height:200px;overflow-y:auto;">
                            {emps.into_iter().map(|e| {
                                let eid = e.id.to_string();
                                view! {
                                    <label class="checkbox is-block mb-1">
                                        <input type="checkbox" on:change=move |ev| {
                                            let t: HtmlInputElement = event_target(&ev);
                                            selected_ids.update(|ids| {
                                                if t.checked() {
                                                    ids.push(eid.clone());
                                                } else {
                                                    ids.retain(|id| id != &eid);
                                                }
                                            });
                                        } />
                                        " "{e.name}
                                    </label>
                                }
                            }).collect_view()}
                        </div>
                        <button class="button is-primary is-small mt-2" on:click=create_circulations>"送信"</button>
                    </div>
                }.into_any()
            } else { view! { <div></div> }.into_any() }}

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    circ_resource.get().map(|result| match result {
                        Ok(circs) => {
                            if circs.is_empty() {
                                return view! { <p class="has-text-grey is-size-7">"回覧なし"</p> }.into_any();
                            }

                            let my_unconfirmed = circs.iter().any(|c| {
                                user_id == Some(c.recipient.id) && c.confirmed_at.is_none()
                            });

                            view! {
                                <div>
                                    {circs.into_iter().map(|c| {
                                        let confirmed = c.confirmed_at.is_some();
                                        view! {
                                            <div class="is-flex is-justify-content-space-between is-align-items-center mb-1">
                                                <span>{c.recipient.name}</span>
                                                {if confirmed {
                                                    view! { <span class="tag is-success is-light is-small">"確認済"</span> }.into_any()
                                                } else {
                                                    view! { <span class="tag is-warning is-light is-small">"未確認"</span> }.into_any()
                                                }}
                                            </div>
                                        }
                                    }).collect_view()}

                                    {if my_unconfirmed {
                                        view! {
                                            <button class="button is-success is-small mt-2" on:click=do_confirm>
                                                <span class="icon"><i class="fas fa-check"></i></span>
                                                <span>"確認する"</span>
                                            </button>
                                        }.into_any()
                                    } else { view! { <span></span> }.into_any() }}
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
