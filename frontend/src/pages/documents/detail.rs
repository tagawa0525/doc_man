use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;

use crate::api;
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::status_badge::StatusBadge;
use crate::pages::documents::approval::ApprovalSection;
use crate::pages::documents::circulation::CirculationSection;

#[component]
pub fn DocumentDetailPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let params = use_params_map();
    let refresh = RwSignal::new(0u32);

    let doc_id = move || params.read().get("id").and_then(|id| Uuid::parse_str(&id).ok());
    let can_edit = auth.role().map_or(false, |r| !matches!(r, crate::auth::Role::Viewer));

    let doc_resource = LocalResource::new(
        move || {
            let id = doc_id();
            let _ = refresh.get();
            async move {
                match id {
                    Some(id) => api::documents::get(id).await.ok(),
                    None => None,
                }
            }
        },
    );

    view! {
        <div>
            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    doc_resource.get().map(|doc| match doc {
                        Some(doc) => {
                            let id = doc.id;
                            let edit_url = format!("/documents/{}/edit", id);
                            let status = doc.status.clone();
                            view! {
                                <div class="level">
                                    <div class="level-left">
                                        <h1 class="title">
                                            {doc.doc_number.clone()}" - "{doc.title.clone()}
                                        </h1>
                                    </div>
                                    <div class="level-right">
                                        <div class="buttons">
                                            {if can_edit {
                                                view! {
                                                    <a href=edit_url class="button is-info">
                                                        <span class="icon"><i class="fas fa-edit"></i></span>
                                                        <span>"編集"</span>
                                                    </a>
                                                }.into_any()
                                            } else { view! { <span></span> }.into_any() }}
                                            <a href="/documents" class="button">"一覧に戻る"</a>
                                        </div>
                                    </div>
                                </div>

                                <div class="columns">
                                    <div class="column is-8">
                                        <div class="box">
                                            <h2 class="subtitle">"文書情報"</h2>
                                            <table class="table is-fullwidth">
                                                <tbody>
                                                    <tr><th style="width:150px">"文書番号"</th><td>{doc.doc_number}</td></tr>
                                                    <tr><th>"タイトル"</th><td>{doc.title}</td></tr>
                                                    <tr><th>"リビジョン"</th><td>{doc.revision.to_string()}</td></tr>
                                                    <tr><th>"ステータス"</th><td><StatusBadge status=doc.status /></td></tr>
                                                    <tr><th>"機密区分"</th><td>{doc.confidentiality}</td></tr>
                                                    <tr><th>"ファイルパス"</th><td class="is-size-7">{doc.file_path}</td></tr>
                                                    <tr><th>"部署コード"</th><td>{doc.frozen_dept_code}</td></tr>
                                                    <tr><th>"文書種別"</th><td>{format!("{} ({})", doc.doc_kind.name, doc.doc_kind.code)}</td></tr>
                                                    <tr><th>"プロジェクト"</th><td>{doc.project.name}</td></tr>
                                                    <tr><th>"作成者"</th><td>{doc.author.name}</td></tr>
                                                    <tr><th>"タグ"</th><td>
                                                        <div class="tags">
                                                            {doc.tags.into_iter().map(|t| view! { <span class="tag is-info is-light">{t}</span> }).collect_view()}
                                                        </div>
                                                    </td></tr>
                                                    <tr><th>"作成日時"</th><td>{doc.created_at.format("%Y-%m-%d %H:%M").to_string()}</td></tr>
                                                    <tr><th>"更新日時"</th><td>{doc.updated_at.format("%Y-%m-%d %H:%M").to_string()}</td></tr>
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                    <div class="column is-4">
                                        <ApprovalSection doc_id=id doc_status=status.clone() on_change=Callback::new(move |_| refresh.update(|v| *v += 1)) />
                                        <CirculationSection doc_id=id doc_status=status on_change=Callback::new(move |_| refresh.update(|v| *v += 1)) />
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
