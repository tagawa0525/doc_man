use leptos::prelude::*;

use crate::api;
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::modal::ConfirmModal;
use crate::components::pagination::Pagination;
use crate::components::status_badge::StatusBadge;
use crate::components::toast::ToastContext;

#[component]
pub fn DocumentListPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let toast = expect_context::<ToastContext>();
    let page = RwSignal::new(1u32);
    let refresh = RwSignal::new(0u32);
    let delete_target = RwSignal::new(Option::<(uuid::Uuid, String)>::None);

    let can_create = auth.role().map_or(false, |r| !matches!(r, crate::auth::Role::Viewer));

    let resource = LocalResource::new(
        move || {
            let p = page.get();
            let _ = refresh.get();
            async move { api::documents::list(p, 20).await }
        },
    );

    let do_delete = move |id: uuid::Uuid| {
        leptos::task::spawn_local(async move {
            match api::documents::delete(id).await {
                Ok(_) => { toast.success("削除しました"); refresh.update(|v| *v += 1); }
                Err(e) => toast.error(format!("削除失敗: {}", e.message)),
            }
            delete_target.set(None);
        });
    };

    view! {
        <div>
            <div class="level">
                <div class="level-left"><h1 class="title">"文書一覧"</h1></div>
                {if can_create {
                    view! {
                        <div class="level-right">
                            <a href="/documents/new" class="button is-primary">
                                <span class="icon"><i class="fas fa-plus"></i></span>
                                <span>"新規作成"</span>
                            </a>
                        </div>
                    }.into_any()
                } else { view! { <div></div> }.into_any() }}
            </div>

            {move || delete_target.get().map(|(id, title)| view! {
                <ConfirmModal
                    title="文書削除"
                    message=format!("「{}」を削除しますか？", title)
                    on_confirm=Callback::new(move |_| do_delete(id))
                    on_cancel=Callback::new(move |_| delete_target.set(None))
                    danger=true
                />
            })}

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    resource.get().map(|result| match result {
                        Ok(paginated) => {
                            let total = paginated.meta.total;
                            let cp = paginated.meta.page;
                            let pp = paginated.meta.per_page;
                            view! {
                                <div class="box">
                                    <table class="table is-fullwidth is-hoverable">
                                        <thead>
                                            <tr>
                                                <th>"文書番号"</th><th>"タイトル"</th><th>"ステータス"</th>
                                                <th>"種別"</th><th>"プロジェクト"</th><th>"作成者"</th>
                                                <th>"タグ"</th><th>"操作"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {paginated.data.into_iter().map(|doc| {
                                                let detail_url = format!("/documents/{}", doc.id);
                                                let title = doc.title.clone();
                                                let id = doc.id;
                                                view! {
                                                    <tr>
                                                        <td><span class="has-text-weight-semibold">{doc.doc_number}</span></td>
                                                        <td><a href=detail_url>{doc.title}</a></td>
                                                        <td><StatusBadge status=doc.status /></td>
                                                        <td>{doc.doc_kind.name}</td>
                                                        <td>{doc.project.name}</td>
                                                        <td>{doc.author.name}</td>
                                                        <td>
                                                            <div class="tags">
                                                                {doc.tags.into_iter().map(|t| view! { <span class="tag is-info is-light">{t}</span> }).collect_view()}
                                                            </div>
                                                        </td>
                                                        <td>
                                                            <div class="buttons are-small">
                                                                <a href=format!("/documents/{}", id) class="button is-info is-outlined">
                                                                    <span class="icon"><i class="fas fa-eye"></i></span>
                                                                </a>
                                                                {if can_create {
                                                                    let title = title.clone();
                                                                    view! {
                                                                        <button class="button is-danger is-outlined" on:click=move |_| delete_target.set(Some((id, title.clone())))>
                                                                            <span class="icon"><i class="fas fa-trash"></i></span>
                                                                        </button>
                                                                    }.into_any()
                                                                } else { view! { <span></span> }.into_any() }}
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                    <Pagination current_page=cp total=total per_page=pp on_page_change=Callback::new(move |p| page.set(p)) />
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
