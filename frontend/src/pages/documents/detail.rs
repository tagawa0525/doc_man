use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::types::{ReviseDocumentRequest, UpdateDocumentRequest};
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::modal::ConfirmModal;
use crate::components::status_badge::StatusBadge;
use crate::components::toast::ToastContext;
use crate::pages::documents::approval::ApprovalSection;
use crate::pages::documents::distribution::DistributionSection;

#[component]
pub fn DocumentDetailPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let toast = expect_context::<ToastContext>();
    let params = use_params_map();
    let refresh = RwSignal::new(0u32);

    let doc_id = move || {
        params
            .read()
            .get("id")
            .and_then(|id| Uuid::parse_str(&id).ok())
    };
    let can_edit = auth
        .role()
        .is_some_and(|r| !matches!(r, crate::auth::Role::Viewer));
    let is_admin = auth.role().is_some_and(|r| r.is_admin());

    let editing = RwSignal::new(false);
    let form_title = RwSignal::new(String::new());
    let form_confidentiality = RwSignal::new(String::new());
    let form_tags = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    let delete_target = RwSignal::new(Option::<(Uuid, String)>::None);

    // 改訂関連
    let revise_open = RwSignal::new(false);
    let revise_reason = RwSignal::new(String::new());
    let revising = RwSignal::new(false);

    let doc_resource = LocalResource::new(move || {
        let id = doc_id();
        let _ = refresh.get();
        async move {
            match id {
                Some(id) => api::documents::get(id).await.ok(),
                None => None,
            }
        }
    });

    let revisions_resource = LocalResource::new(move || {
        let id = doc_id();
        let _ = refresh.get();
        async move {
            match id {
                Some(id) => api::documents::list_revisions(id).await.ok(),
                None => None,
            }
        }
    });

    let start_editing = move |title: String, confidentiality: String, tags: Vec<String>| {
        form_title.set(title);
        form_confidentiality.set(confidentiality);
        form_tags.set(tags.join(", "));
        editing.set(true);
    };

    let cancel_editing = move || {
        editing.set(false);
    };

    let do_save = move || {
        let Some(id) = doc_id() else { return };
        let title = form_title.get_untracked();

        if title.is_empty() {
            toast.error("タイトルは必須です");
            return;
        }

        saving.set(true);
        let confidentiality = form_confidentiality.get_untracked();
        let tags_str = form_tags.get_untracked();
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        leptos::task::spawn_local(async move {
            match api::documents::update(
                id,
                &UpdateDocumentRequest {
                    title: Some(title),
                    confidentiality: Some(confidentiality),
                    tags: Some(tags),
                    doc_number: None,
                    frozen_dept_code: None,
                    status: None,
                },
            )
            .await
            {
                Ok(_) => {
                    toast.success("保存しました");
                    editing.set(false);
                    refresh.update(|v| *v += 1);
                }
                Err(e) => toast.error(format!("保存失敗: {}", e.message)),
            }
            saving.set(false);
        });
    };

    let do_revise = move || {
        let Some(id) = doc_id() else { return };
        let reason = revise_reason.get_untracked();

        if reason.trim().is_empty() {
            toast.error("改訂理由は必須です");
            return;
        }

        revising.set(true);
        leptos::task::spawn_local(async move {
            match api::documents::revise(id, &ReviseDocumentRequest { reason }).await {
                Ok(_) => {
                    toast.success("改訂しました");
                    revise_open.set(false);
                    revise_reason.set(String::new());
                    refresh.update(|v| *v += 1);
                }
                Err(e) => toast.error(format!("改訂失敗: {}", e.message)),
            }
            revising.set(false);
        });
    };

    let do_delete = move |id: Uuid| {
        leptos::task::spawn_local(async move {
            match api::documents::delete(id).await {
                Ok(()) => {
                    toast.success("削除しました");
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/documents");
                    }
                }
                Err(e) => toast.error(format!("削除失敗: {}", e.message)),
            }
            delete_target.set(None);
        });
    };

    view! {
        <div>
            {move || delete_target.get().map(|(id, title)| view! {
                <ConfirmModal
                    title="文書削除"
                    message=format!("「{}」を削除しますか？この操作は取り消せません。", title)
                    on_confirm=Callback::new(move |()| do_delete(id))
                    on_cancel=Callback::new(move |()| delete_target.set(None))
                    danger=true
                />
            })}

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    doc_resource.get().map(|doc| match doc {
                        Some(doc) => {
                            let id = doc.id;
                            let status = doc.status.clone();
                            let doc_title = doc.title.clone();
                            let doc_number_display = doc.doc_number.clone();
                            let is_approved = status == "approved";
                            // Clone fields for edit button closure
                            let edit_title = doc.title.clone();
                            let edit_conf = doc.confidentiality.clone();
                            let edit_tags = doc.tags.clone();
                            // Clone for delete
                            let del_title = doc.title.clone();

                            view! {
                                <div class="level">
                                    <div class="level-left">
                                        <h1 class="title">
                                            {doc_number_display}" - "{doc_title}
                                        </h1>
                                    </div>
                                    <div class="level-right">
                                        <div class="buttons">
                                            {move || {
                                                if editing.get() {
                                                    view! {
                                                        <button class="button is-primary" prop:disabled=move || saving.get()
                                                            on:click=move |_| do_save()>
                                                            <span class="icon"><i class="fas fa-save"></i></span>
                                                            <span>"保存"</span>
                                                        </button>
                                                        <button class="button" on:click=move |_| cancel_editing()>
                                                            "キャンセル"
                                                        </button>
                                                    }.into_any()
                                                } else if can_edit {
                                                    let et = edit_title.clone();
                                                    let ec = edit_conf.clone();
                                                    let etags = edit_tags.clone();
                                                    view! {
                                                        <button class="button is-info"
                                                            on:click=move |_| start_editing(et.clone(), ec.clone(), etags.clone())>
                                                            <span class="icon"><i class="fas fa-edit"></i></span>
                                                            <span>"編集"</span>
                                                        </button>
                                                        {if is_approved && can_edit {
                                                            view! {
                                                                <button class="button is-warning"
                                                                    on:click=move |_| revise_open.set(true)>
                                                                    <span class="icon"><i class="fas fa-code-branch"></i></span>
                                                                    <span>"改訂"</span>
                                                                </button>
                                                            }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                        <a href="/documents" class="button">"一覧に戻る"</a>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <a href="/documents" class="button">"一覧に戻る"</a>
                                                    }.into_any()
                                                }
                                            }}
                                        </div>
                                    </div>
                                </div>

                                // 改訂理由入力フォーム
                                {move || if revise_open.get() {
                                    view! {
                                        <div class="notification is-warning is-light">
                                            <p class="mb-2"><strong>"改訂理由を入力してください"</strong></p>
                                            <div class="field">
                                                <textarea class="textarea" rows=2
                                                    prop:value=move || revise_reason.get()
                                                    on:input=move |ev| { let t: web_sys::HtmlTextAreaElement = event_target(&ev); revise_reason.set(t.value()); }
                                                    placeholder="改訂理由（必須）">
                                                </textarea>
                                            </div>
                                            <div class="buttons">
                                                <button class="button is-warning" prop:disabled=move || revising.get()
                                                    on:click=move |_| do_revise()>
                                                    "改訂実行"
                                                </button>
                                                <button class="button" on:click=move |_| { revise_open.set(false); revise_reason.set(String::new()); }>
                                                    "キャンセル"
                                                </button>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}

                                <div class="columns">
                                    <div class="column is-8">
                                        <div class="box">
                                            <h2 class="subtitle">"文書情報"</h2>
                                            <table class="table is-fullwidth">
                                                <tbody>
                                                    <tr><th style="width:150px">"文書番号"</th><td>{doc.doc_number}</td></tr>
                                                    <tr><th>"タイトル"</th><td>
                                                        {move || if editing.get() {
                                                            view! {
                                                                <input class="input" type="text" prop:value=move || form_title.get()
                                                                    on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_title.set(t.value()); } />
                                                            }.into_any()
                                                        } else {
                                                            view! { <span>{doc.title.clone()}</span> }.into_any()
                                                        }}
                                                    </td></tr>
                                                    <tr><th>"リビジョン"</th><td>{"Rev."}{doc.revision.to_string()}</td></tr>
                                                    <tr><th>"ステータス"</th><td><StatusBadge status=doc.status /></td></tr>
                                                    <tr><th>"機密区分"</th><td>
                                                        {move || if editing.get() {
                                                            view! {
                                                                <div class="select">
                                                                    <select prop:value=move || form_confidentiality.get()
                                                                        on:change=move |ev| form_confidentiality.set(event_target_value(&ev))>
                                                                        <option value="public">"公開"</option>
                                                                        <option value="internal">"社内"</option>
                                                                        <option value="restricted">"限定"</option>
                                                                        <option value="confidential">"機密"</option>
                                                                    </select>
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! { <span>{doc.confidentiality.clone()}</span> }.into_any()
                                                        }}
                                                    </td></tr>
                                                    <tr><th>"ファイルパス"</th><td>
                                                        <span class="is-size-7">{doc.file_path.clone()}</span>
                                                    </td></tr>
                                                    <tr><th>"部署コード"</th><td>{doc.frozen_dept_code}</td></tr>
                                                    <tr><th>"文書種別"</th><td>{format!("{} ({})", doc.doc_kind.name, doc.doc_kind.code)}</td></tr>
                                                    <tr><th>"プロジェクト"</th><td>{doc.project.name}</td></tr>
                                                    <tr><th>"作成者"</th><td>{doc.author.name}</td></tr>
                                                    <tr><th>"タグ"</th><td>
                                                        {move || if editing.get() {
                                                            view! {
                                                                <input class="input" type="text" placeholder="カンマ区切り" prop:value=move || form_tags.get()
                                                                    on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_tags.set(t.value()); } />
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="tags">
                                                                    {doc.tags.clone().into_iter().map(|t| view! { <span class="tag is-info is-light">{t}</span> }).collect_view()}
                                                                </div>
                                                            }.into_any()
                                                        }}
                                                    </td></tr>
                                                    <tr><th>"作成日時"</th><td>{doc.created_at.format("%Y-%m-%d %H:%M").to_string()}</td></tr>
                                                    <tr><th>"更新日時"</th><td>{doc.updated_at.format("%Y-%m-%d %H:%M").to_string()}</td></tr>
                                                </tbody>
                                            </table>

                                            {if is_admin {
                                                let del_title = del_title.clone();
                                                view! {
                                                    <div class="mt-5 pt-4" style="border-top: 1px solid #dbdbdb">
                                                        <button class="button is-danger is-outlined"
                                                            on:click=move |_| delete_target.set(Some((id, del_title.clone())))>
                                                            <span class="icon"><i class="fas fa-trash"></i></span>
                                                            <span>"この文書を削除"</span>
                                                        </button>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }}
                                        </div>

                                        // 改訂履歴セクション
                                        <div class="box">
                                            <h2 class="subtitle">"改訂履歴"</h2>
                                            <Suspense fallback=move || view! { <Loading /> }>
                                                {move || {
                                                    revisions_resource.get().map(|revs| match revs {
                                                        Some(revisions) if !revisions.is_empty() => {
                                                            view! {
                                                                <table class="table is-fullwidth is-hoverable is-narrow">
                                                                    <thead>
                                                                        <tr>
                                                                            <th>"Rev."</th>
                                                                            <th>"ファイルパス"</th>
                                                                            <th>"理由"</th>
                                                                            <th>"作成者"</th>
                                                                            <th>"有効開始"</th>
                                                                            <th>"有効終了"</th>
                                                                        </tr>
                                                                    </thead>
                                                                    <tbody>
                                                                        {revisions.into_iter().map(|rev| {
                                                                            view! {
                                                                                <tr>
                                                                                    <td>{rev.revision.to_string()}</td>
                                                                                    <td class="is-size-7">{rev.file_path}</td>
                                                                                    <td>{rev.reason.unwrap_or_default()}</td>
                                                                                    <td>{rev.created_by.name}</td>
                                                                                    <td>{rev.effective_from.format("%Y-%m-%d %H:%M").to_string()}</td>
                                                                                    <td>{rev.effective_to.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_default()}</td>
                                                                                </tr>
                                                                            }
                                                                        }).collect_view()}
                                                                    </tbody>
                                                                </table>
                                                            }.into_any()
                                                        }
                                                        _ => view! { <p class="has-text-grey">"改訂履歴はありません"</p> }.into_any(),
                                                    })
                                                }}
                                            </Suspense>
                                        </div>
                                    </div>
                                    <div class="column is-4">
                                        <ApprovalSection doc_id=id doc_status=status on_change=Callback::new(move |()| refresh.update(|v| *v += 1)) />
                                        <DistributionSection doc_id=id />
                                    </div>
                                </div>
                            }.into_any()
                        }
                        None => view! { <div class="notification is-warning">"文書が見つかりません"</div> }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
