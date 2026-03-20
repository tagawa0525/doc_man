use leptos::prelude::*;
use uuid::Uuid;

use crate::api;
use crate::api::types::{CreateDistributionRequest, DistributionResponse};
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::toast::ToastContext;

#[component]
pub fn DistributionSection(doc_id: Uuid) -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let toast = expect_context::<ToastContext>();
    let refresh = RwSignal::new(0u32);
    let show_form = RwSignal::new(false);

    let can_distribute = auth.role().is_some_and(|r| r.can_manage());

    let dist_resource = LocalResource::new(move || {
        let _ = refresh.get();
        async move { api::distributions::list(doc_id).await }
    });

    let employees_resource = LocalResource::new(|| async { api::employees::list_active().await });

    // チェックボックスの状態: employee_id → checked (id keyed で index 非依存)
    let selected = RwSignal::new(std::collections::HashMap::<Uuid, bool>::new());

    // 前回の配布先で初期化するためのヘルパー
    let init_selected = move |distributions: &[DistributionResponse],
                              employees: &[(Uuid, String)]| {
        // 直近の配布バッチの distributed_at を取得
        let latest_at = distributions.first().map(|d| d.distributed_at);

        let latest_recipients: std::collections::HashSet<Uuid> = distributions
            .iter()
            .filter(|d| Some(d.distributed_at) == latest_at)
            .map(|d| d.recipient.id)
            .collect();

        let has_history = !latest_recipients.is_empty();

        employees
            .iter()
            .map(|(id, _)| (*id, has_history && latest_recipients.contains(id)))
            .collect::<std::collections::HashMap<_, _>>()
    };

    let do_distribute = move |_: leptos::ev::MouseEvent| {
        let ids: Vec<Uuid> = selected
            .get_untracked()
            .into_iter()
            .filter(|(_, checked)| *checked)
            .map(|(id, _)| id)
            .collect();

        if ids.is_empty() {
            toast.error("配布先を選択してください");
            return;
        }

        leptos::task::spawn_local(async move {
            match api::distributions::create(
                doc_id,
                &CreateDistributionRequest { recipient_ids: ids },
            )
            .await
            {
                Ok(_) => {
                    toast.success("配布しました");
                    show_form.set(false);
                    refresh.update(|v| *v += 1);
                }
                Err(e) => toast.error(format!("配布失敗: {}", e.message)),
            }
        });
    };

    view! {
        <div class="box mb-4">
            <h3 class="subtitle">
                <span class="icon"><i class="fas fa-paper-plane"></i></span>
                " 配布"
            </h3>

            {if can_distribute {
                view! {
                    <button
                        class="button is-small is-primary mb-3"
                        on:click=move |_| {
                            // フォーム表示トグル、初回表示時に選択状態を初期化
                            let opening = !show_form.get_untracked();
                            if opening {
                                let dists = dist_resource.get().and_then(std::result::Result::ok).unwrap_or_default();
                                let emps = employees_resource.get()
                                    .and_then(std::result::Result::ok)
                                    .map(|p| p.data.into_iter().map(|e| (e.id, e.name)).collect::<Vec<_>>())
                                    .unwrap_or_default();
                                selected.set(init_selected(&dists, &emps));
                            }
                            show_form.set(opening);
                        }
                    >
                        <span class="icon"><i class="fas fa-paper-plane"></i></span>
                        <span>"配布する"</span>
                    </button>
                }.into_any()
            } else { view! { <span></span> }.into_any() }}

            {move || if show_form.get() {
                let emps = employees_resource.get()
                    .and_then(std::result::Result::ok)
                    .map(|p| p.data)
                    .unwrap_or_default();
                view! {
                    <div class="notification is-light mb-3">
                        <div class="field">
                            <label class="label is-small">"配布先を選択"</label>
                            {emps.into_iter().map(|e| {
                                let emp_id = e.id;
                                let checked = move || selected.get().get(&emp_id).copied().unwrap_or(false);
                                view! {
                                    <label class="checkbox is-block mb-1">
                                        <input
                                            type="checkbox"
                                            prop:checked=checked
                                            on:change=move |_| {
                                                selected.update(|sel| {
                                                    let entry = sel.entry(emp_id).or_insert(false);
                                                    *entry = !*entry;
                                                });
                                            }
                                        />
                                        " "{e.name}
                                    </label>
                                }
                            }).collect_view()}
                        </div>
                        <button class="button is-primary is-small" on:click=do_distribute>"送信"</button>
                    </div>
                }.into_any()
            } else { view! { <div></div> }.into_any() }}

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    dist_resource.get().map(|result| match result {
                        Ok(dists) => {
                            if dists.is_empty() {
                                return view! { <p class="has-text-grey is-size-7">"配布履歴はありません"</p> }.into_any();
                            }
                            // distributed_at でグルーピング
                            let mut batches: Vec<(String, String, Vec<String>)> = Vec::new();
                            for d in &dists {
                                let key = d.distributed_at.format("%Y-%m-%d %H:%M").to_string();
                                let by_name = d.distributed_by.name.clone();
                                if let Some(batch) = batches.last_mut().filter(|(k, _, _)| *k == key) {
                                    batch.2.push(d.recipient.name.clone());
                                } else {
                                    batches.push((key, by_name, vec![d.recipient.name.clone()]));
                                }
                            }
                            view! {
                                <div>
                                    <p class="has-text-weight-semibold is-size-7 mb-2">"配布履歴"</p>
                                    {batches.into_iter().map(|(at, by, recipients)| {
                                        view! {
                                            <div class="mb-3 p-2" style="border-left: 3px solid #3273dc; margin-left: 0.5rem;">
                                                <p class="is-size-7 has-text-weight-semibold">
                                                    {format!("{at}　{by}")}
                                                </p>
                                                <ul class="is-size-7 ml-3" style="list-style: disc;">
                                                    {recipients.into_iter().map(|name| view! { <li>{name}</li> }).collect_view()}
                                                </ul>
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
