use leptos::prelude::*;

use crate::api;
use crate::auth::AuthContext;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;
use crate::components::toast::ToastContext;

#[component]
pub fn EmployeeListPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let _toast = expect_context::<ToastContext>();
    let page = RwSignal::new(1u32);
    let refresh = RwSignal::new(0u32);

    let is_admin = auth.role().is_some_and(|r| r.is_admin());

    let resource = LocalResource::new(move || {
        let p = page.get();
        let _ = refresh.get();
        async move { api::employees::list(p, 20).await }
    });

    view! {
        <div>
            <div class="level">
                <div class="level-left"><h1 class="title">"社員管理"</h1></div>
                {if is_admin {
                    view! {
                        <div class="level-right">
                            <a href="/employees/new" class="button is-primary">
                                <span class="icon"><i class="fas fa-plus"></i></span>
                                <span>"新規作成"</span>
                            </a>
                        </div>
                    }.into_any()
                } else { view! { <div></div> }.into_any() }}
            </div>

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    resource.get().map(|result| match result {
                        Ok(paginated) => {
                            let total = paginated.meta.total;
                            let current_page = paginated.meta.page;
                            let per_page = paginated.meta.per_page;
                            view! {
                                <div class="box">
                                    <table class="table is-fullwidth is-hoverable">
                                        <thead>
                                            <tr>
                                                <th>"名前"</th>
                                                <th>"社員コード"</th>
                                                <th>"ロール"</th>
                                                <th>"部署"</th>
                                                <th>"状態"</th>
                                                <th>"操作"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {paginated.data.into_iter().map(|emp| {
                                                let detail_url = format!("/employees/{}", emp.id);
                                                view! {
                                                    <tr>
                                                        <td><a href=detail_url>{emp.name}</a></td>
                                                        <td>{emp.employee_code.unwrap_or_else(|| "-".to_string())}</td>
                                                        <td><span class="tag is-light">{emp.role}</span></td>
                                                        <td>{emp.current_department.map_or_else(|| "-".to_string(), |d| d.name)}</td>
                                                        <td>
                                                            {if emp.is_active {
                                                                view! { <span class="tag is-success is-light">"有効"</span> }.into_any()
                                                            } else {
                                                                view! { <span class="tag is-danger is-light">"無効"</span> }.into_any()
                                                            }}
                                                        </td>
                                                        <td>
                                                            <a href=format!("/employees/{}", emp.id) class="button is-small is-info is-outlined">
                                                                <span class="icon"><i class="fas fa-eye"></i></span>
                                                            </a>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                    <Pagination current_page=current_page total=total per_page=per_page on_page_change=Callback::new(move |p| page.set(p)) />
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
