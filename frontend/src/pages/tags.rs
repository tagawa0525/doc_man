use leptos::prelude::*;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::types::CreateTagRequest;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;
use crate::components::toast::ToastContext;

#[component]
pub fn TagsPage() -> impl IntoView {
    let toast = expect_context::<ToastContext>();
    let page = RwSignal::new(1u32);
    let refresh = RwSignal::new(0u32);
    let new_tag_name = RwSignal::new(String::new());
    let creating = RwSignal::new(false);

    let tags_resource = LocalResource::new(move || {
        let p = page.get();
        let _ = refresh.get();
        async move { api::tags::list(p, 20).await }
    });

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = new_tag_name.get_untracked();
        if name.is_empty() {
            toast.error("タグ名を入力してください");
            return;
        }

        creating.set(true);
        leptos::task::spawn_local(async move {
            match api::tags::create(&CreateTagRequest { name }).await {
                Ok(_) => {
                    toast.success("タグを作成しました");
                    new_tag_name.set(String::new());
                    refresh.update(|v| *v += 1);
                }
                Err(e) => toast.error(format!("作成に失敗しました: {}", e.message)),
            }
            creating.set(false);
        });
    };

    view! {
        <div>
            <h1 class="title">"タグ管理"</h1>

            <div class="box mb-5">
                <h2 class="subtitle">"新規タグ作成"</h2>
                <form on:submit=on_create>
                    <div class="field has-addons">
                        <div class="control is-expanded">
                            <input
                                class="input"
                                type="text"
                                placeholder="タグ名"
                                prop:value=move || new_tag_name.get()
                                on:input=move |ev| {
                                    let target: HtmlInputElement = event_target(&ev);
                                    new_tag_name.set(target.value());
                                }
                                prop:disabled=move || creating.get()
                            />
                        </div>
                        <div class="control">
                            <button class="button is-primary" type="submit" prop:disabled=move || creating.get()>
                                <span class="icon"><i class="fas fa-plus"></i></span>
                                <span>"作成"</span>
                            </button>
                        </div>
                    </div>
                </form>
            </div>

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    tags_resource.get().map(|result| {
                        match result {
                            Ok(paginated) => {
                                let total = paginated.meta.total;
                                let current_page = paginated.meta.page;
                                let per_page = paginated.meta.per_page;
                                view! {
                                    <div class="box">
                                        <table class="table is-fullwidth is-hoverable">
                                            <thead>
                                                <tr>
                                                    <th>"タグ名"</th>
                                                    <th>"ID"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {paginated.data.into_iter().map(|tag| {
                                                    view! {
                                                        <tr>
                                                            <td>
                                                                <span class="tag is-info is-light">{tag.name}</span>
                                                            </td>
                                                            <td class="has-text-grey is-size-7">{tag.id.to_string()}</td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                        <Pagination
                                            current_page=current_page
                                            total=total
                                            per_page=per_page
                                            on_page_change=Callback::new(move |p| page.set(p))
                                        />
                                    </div>
                                }.into_any()
                            }
                            Err(e) => view! {
                                <div class="notification is-danger">{format!("読み込みに失敗しました: {}", e.message)}</div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
