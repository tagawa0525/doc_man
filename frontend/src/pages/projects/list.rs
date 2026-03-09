use leptos::prelude::*;

use crate::api;
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::modal::ConfirmModal;
use crate::components::pagination::Pagination;
use crate::components::toast::ToastContext;

#[component]
pub fn ProjectListPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let toast = expect_context::<ToastContext>();
    let page = RwSignal::new(1u32);
    let refresh = RwSignal::new(0u32);
    let delete_target = RwSignal::new(Option::<(uuid::Uuid, String)>::None);

    let is_admin = auth.role().is_some_and(|r| r.is_admin());

    let resource = LocalResource::new(move || {
        let p = page.get();
        let _ = refresh.get();
        async move { api::projects::list(p, 20).await }
    });

    let do_delete = move |id: uuid::Uuid| {
        leptos::task::spawn_local(async move {
            match api::projects::delete(id).await {
                Ok(()) => {
                    toast.success("削除しました");
                    refresh.update(|v| *v += 1);
                }
                Err(e) => toast.error(format!("削除失敗: {}", e.message)),
            }
            delete_target.set(None);
        });
    };

    view! {
        <div>
            <div class="level">
                <div class="level-left"><h1 class="title">"プロジェクト管理"</h1></div>
                {if is_admin {
                    view! {
                        <div class="level-right">
                            <a href="/projects/new" class="button is-primary">
                                <span class="icon"><i class="fas fa-plus"></i></span>
                                <span>"新規作成"</span>
                            </a>
                        </div>
                    }.into_any()
                } else { view! { <div></div> }.into_any() }}
            </div>

            {move || delete_target.get().map(|(id, name)| {
                view! {
                    <ConfirmModal
                        title="プロジェクト削除"
                        message=format!("「{}」を削除しますか？", name)
                        on_confirm=Callback::new(move |()| do_delete(id))
                        on_cancel=Callback::new(move |()| delete_target.set(None))
                        danger=true
                    />
                }
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
                                                <th>"名前"</th><th>"ステータス"</th><th>"専門分野"</th>
                                                <th>"部署"</th><th>"マネージャー"</th><th>"操作"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {paginated.data.into_iter().map(|p| {
                                                let detail_url = format!("/projects/{}", p.id);
                                                let name = p.name.clone();
                                                let id = p.id;
                                                view! {
                                                    <tr>
                                                        <td><a href=detail_url>{p.name}</a></td>
                                                        <td><span class="tag is-light">{p.status}</span></td>
                                                        <td>{p.discipline.name}</td>
                                                        <td>{p.discipline.department.name}</td>
                                                        <td>{p.manager.map_or_else(|| "-".to_string(), |m| m.name)}</td>
                                                        <td>
                                                            <div class="buttons are-small">
                                                                <a href=format!("/projects/{}", id) class="button is-info is-outlined">
                                                                    <span class="icon"><i class="fas fa-eye"></i></span>
                                                                </a>
                                                                {if is_admin {
                                                                    let name = name.clone();
                                                                    view! {
                                                                        <button class="button is-danger is-outlined" on:click=move |_| delete_target.set(Some((id, name.clone())))>
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
