use leptos::prelude::*;
use uuid::Uuid;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::types::CreateDocumentRequest;
use crate::components::form::FormField;
use crate::components::toast::ToastContext;

#[component]
pub fn DocumentCreatePage() -> impl IntoView {
    let toast = expect_context::<ToastContext>();

    let form_title = RwSignal::new(String::new());
    let form_file_path = RwSignal::new(String::new());
    let form_confidentiality = RwSignal::new("internal".to_string());
    let form_doc_kind_id = RwSignal::new(String::new());
    let form_project_id = RwSignal::new(String::new());
    let form_tags = RwSignal::new(String::new());
    let saving = RwSignal::new(false);

    let doc_kinds_resource = LocalResource::new(|| async { api::document_kinds::list_all().await });
    let projects_resource = LocalResource::new(|| async { api::projects::list(1, 100).await });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let title = form_title.get_untracked();
        let file_path = form_file_path.get_untracked();

        if title.is_empty() || file_path.is_empty() {
            toast.error("タイトルとファイルパスは必須です");
            return;
        }

        saving.set(true);
        let confidentiality = form_confidentiality.get_untracked();
        let dki = form_doc_kind_id.get_untracked();
        let pi = form_project_id.get_untracked();
        let tags_str = form_tags.get_untracked();
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        leptos::task::spawn_local(async move {
            if dki.is_empty() || pi.is_empty() {
                toast.error("文書種別とプロジェクトは必須です");
                saving.set(false);
                return;
            }
            let result = api::documents::create(&CreateDocumentRequest {
                title,
                file_path,
                confidentiality: if confidentiality.is_empty() {
                    None
                } else {
                    Some(confidentiality)
                },
                doc_kind_id: Uuid::parse_str(&dki).unwrap(),
                project_id: Uuid::parse_str(&pi).unwrap(),
                tags: if tags.is_empty() { None } else { Some(tags) },
            })
            .await;

            match result {
                Ok(doc) => {
                    toast.success("作成しました");
                    if let Some(window) = web_sys::window() {
                        let _ = window
                            .location()
                            .set_href(&format!("/documents/{}", doc.id));
                    }
                }
                Err(e) => toast.error(format!("失敗: {}", e.message)),
            }
            saving.set(false);
        });
    };

    view! {
        <div>
            <h1 class="title">"文書作成"</h1>
            <div class="box">
                <form on:submit=on_submit>
                    <FormField label="タイトル *">
                        <input class="input" type="text" prop:value=move || form_title.get()
                            on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_title.set(t.value()); } />
                    </FormField>
                    <FormField label="ファイルパス *">
                        <input class="input" type="text" prop:value=move || form_file_path.get()
                            on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_file_path.set(t.value()); } />
                    </FormField>
                    <div class="columns">
                        <div class="column">
                            <FormField label="機密区分">
                                <div class="select is-fullwidth">
                                    <select prop:value=move || form_confidentiality.get()
                                        on:change=move |ev| form_confidentiality.set(event_target_value(&ev))>
                                        <option value="public">"公開"</option>
                                        <option value="internal">"社内"</option>
                                        <option value="restricted">"限定"</option>
                                        <option value="confidential">"機密"</option>
                                    </select>
                                </div>
                            </FormField>
                        </div>
                        <div class="column">
                            <FormField label="文書種別 *">
                                <div class="select is-fullwidth">
                                    <select prop:value=move || form_doc_kind_id.get()
                                        on:change=move |ev| form_doc_kind_id.set(event_target_value(&ev))>
                                        <option value="">"-- 選択 --"</option>
                                        {move || doc_kinds_resource.get().and_then(std::result::Result::ok).map(|p| {
                                            p.data.into_iter().map(|dk| view! { <option value=dk.id.to_string()>{format!("{} ({})", dk.name, dk.code)}</option> }).collect_view()
                                        })}
                                    </select>
                                </div>
                            </FormField>
                        </div>
                        <div class="column">
                            <FormField label="プロジェクト *">
                                <div class="select is-fullwidth">
                                    <select prop:value=move || form_project_id.get()
                                        on:change=move |ev| form_project_id.set(event_target_value(&ev))>
                                        <option value="">"-- 選択 --"</option>
                                        {move || projects_resource.get().and_then(std::result::Result::ok).map(|p| {
                                            p.data.into_iter().map(|proj| view! { <option value=proj.id.to_string()>{proj.name}</option> }).collect_view()
                                        })}
                                    </select>
                                </div>
                            </FormField>
                        </div>
                    </div>
                    <FormField label="タグ（カンマ区切り）">
                        <input class="input" type="text" placeholder="例: 設計, レビュー済" prop:value=move || form_tags.get()
                            on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_tags.set(t.value()); } />
                    </FormField>
                    <div class="field is-grouped">
                        <div class="control"><button class="button is-primary" type="submit" prop:disabled=move || saving.get()>"作成"</button></div>
                        <div class="control"><a href="/documents" class="button">"戻る"</a></div>
                    </div>
                </form>
            </div>
        </div>
    }
}
