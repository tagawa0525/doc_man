use leptos::prelude::*;
use web_sys::HtmlInputElement;

use crate::api;
use crate::api::types::{
    CreateDocumentRegisterRequest, DepartmentTree, DocumentRegisterResponse,
    UpdateDocumentRegisterRequest,
};
use crate::auth::AuthContext;
use crate::components::form::FormField;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;
use crate::components::toast::ToastContext;

#[component]
pub fn DocumentRegistersPage() -> impl IntoView {
    fn flatten_depts(depts: &[DepartmentTree], result: &mut Vec<(String, String)>, prefix: &str) {
        for d in depts {
            let label = if prefix.is_empty() {
                format!("{} ({})", d.name, d.code)
            } else {
                format!("{} > {}", prefix, d.name)
            };
            result.push((d.id.to_string(), label.clone()));
            let next = if prefix.is_empty() {
                d.name.clone()
            } else {
                format!("{} > {}", prefix, d.name)
            };
            flatten_depts(&d.children, result, &next);
        }
    }

    let auth = expect_context::<AuthContext>();
    let toast = expect_context::<ToastContext>();
    let page = RwSignal::new(1u32);
    let refresh = RwSignal::new(0u32);
    let show_form = RwSignal::new(false);
    let edit_id = RwSignal::new(Option::<uuid::Uuid>::None);

    let form_register_code = RwSignal::new(String::new());
    let form_file_server_root = RwSignal::new(String::new());
    let form_new_doc_sub_path = RwSignal::new(String::new());
    let form_doc_number_pattern = RwSignal::new(String::new());
    let form_doc_kind_id = RwSignal::new(String::new());
    let form_dept_id = RwSignal::new(String::new());
    let saving = RwSignal::new(false);

    let is_admin = auth.role().is_some_and(|r| r.is_admin());

    let resource = LocalResource::new(move || {
        let p = page.get();
        let _ = refresh.get();
        async move { api::document_registers::list(p, 20).await }
    });

    let doc_kinds_resource = LocalResource::new(|| async { api::document_kinds::list_all().await });
    let depts_resource = LocalResource::new(|| async { api::departments::list().await });

    let reset_form = move || {
        form_register_code.set(String::new());
        form_file_server_root.set(String::new());
        form_new_doc_sub_path.set(String::new());
        form_doc_number_pattern.set(String::new());
        form_doc_kind_id.set(String::new());
        form_dept_id.set(String::new());
        edit_id.set(None);
        show_form.set(false);
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let rc = form_register_code.get_untracked();
        let fsr = form_file_server_root.get_untracked();

        if rc.is_empty() || fsr.is_empty() {
            toast.error("登録コードとファイルサーバールートは必須です");
            return;
        }

        saving.set(true);
        let eid = edit_id.get_untracked();
        let ndsp = form_new_doc_sub_path.get_untracked();
        let dnp = form_doc_number_pattern.get_untracked();

        leptos::task::spawn_local(async move {
            let result = if let Some(id) = eid {
                api::document_registers::update(
                    id,
                    &UpdateDocumentRegisterRequest {
                        register_code: None,
                        file_server_root: Some(fsr),
                        new_doc_sub_path: if ndsp.is_empty() { None } else { Some(ndsp) },
                        doc_number_pattern: if dnp.is_empty() { None } else { Some(dnp) },
                    },
                )
                .await
            } else {
                let dki = form_doc_kind_id.get_untracked();
                let di = form_dept_id.get_untracked();
                if dki.is_empty() || di.is_empty() {
                    toast.error("文書種別と部署は必須です");
                    saving.set(false);
                    return;
                }
                let doc_kind_id = uuid::Uuid::parse_str(&dki).unwrap();
                let department_id = uuid::Uuid::parse_str(&di).unwrap();
                api::document_registers::create(&CreateDocumentRegisterRequest {
                    register_code: rc,
                    doc_kind_id,
                    department_id,
                    file_server_root: fsr,
                    new_doc_sub_path: if ndsp.is_empty() { None } else { Some(ndsp) },
                    doc_number_pattern: if dnp.is_empty() { None } else { Some(dnp) },
                })
                .await
            };

            match result {
                Ok(_) => {
                    toast.success("保存しました");
                    reset_form();
                    refresh.update(|v| *v += 1);
                }
                Err(e) => toast.error(format!("失敗: {}", e.message)),
            }
            saving.set(false);
        });
    };

    let start_edit = move |dr: DocumentRegisterResponse| {
        form_register_code.set(dr.register_code);
        form_file_server_root.set(dr.file_server_root);
        form_new_doc_sub_path.set(dr.new_doc_sub_path.unwrap_or_default());
        form_doc_number_pattern.set(dr.doc_number_pattern.unwrap_or_default());
        form_doc_kind_id.set(dr.doc_kind.id.to_string());
        form_dept_id.set(dr.department.id.to_string());
        edit_id.set(Some(dr.id));
        show_form.set(true);
    };

    view! {
        <div>
            <div class="level">
                <div class="level-left"><h1 class="title">"文書台帳管理"</h1></div>
                {if is_admin {
                    view! { <div class="level-right"><button class="button is-primary" on:click=move |_| { reset_form(); show_form.set(true); }><span class="icon"><i class="fas fa-plus"></i></span><span>"新規作成"</span></button></div> }.into_any()
                } else { view! { <div></div> }.into_any() }}
            </div>

            {move || if show_form.get() {
                let dk_opts = doc_kinds_resource.get().and_then(std::result::Result::ok).map(|p| p.data).unwrap_or_default();
                let dept_opts = depts_resource.get().and_then(std::result::Result::ok).map(|depts| { let mut o = Vec::new(); flatten_depts(&depts, &mut o, ""); o }).unwrap_or_default();

                view! {
                    <div class="box mb-5">
                        <h2 class="subtitle">{move || if edit_id.get().is_some() { "文書台帳を編集" } else { "新規文書台帳" }}</h2>
                        <form on:submit=on_submit>
                            <div class="columns">
                                <div class="column"><FormField label="登録コード *"><input class="input" type="text" prop:value=move || form_register_code.get() prop:disabled=move || edit_id.get().is_some() on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_register_code.set(t.value()); } /></FormField></div>
                                <div class="column"><FormField label="ファイルサーバールート *"><input class="input" type="text" prop:value=move || form_file_server_root.get() on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_file_server_root.set(t.value()); } /></FormField></div>
                            </div>
                            {move || if edit_id.get().is_none() {
                                let dk_opts = dk_opts.clone();
                                let dept_opts = dept_opts.clone();
                                view! {
                                    <div class="columns">
                                        <div class="column"><FormField label="文書種別 *"><div class="select is-fullwidth"><select prop:value=move || form_doc_kind_id.get() on:change=move |ev| form_doc_kind_id.set(event_target_value(&ev))><option value="">"-- 選択 --"</option>{dk_opts.into_iter().map(|dk| view! { <option value=dk.id.to_string()>{format!("{} ({})", dk.name, dk.code)}</option> }).collect_view()}</select></div></FormField></div>
                                        <div class="column"><FormField label="部署 *"><div class="select is-fullwidth"><select prop:value=move || form_dept_id.get() on:change=move |ev| form_dept_id.set(event_target_value(&ev))><option value="">"-- 選択 --"</option>{dept_opts.into_iter().map(|(v, l)| view! { <option value=v>{l}</option> }).collect_view()}</select></div></FormField></div>
                                    </div>
                                }.into_any()
                            } else { view! { <div></div> }.into_any() }}
                            <div class="columns">
                                <div class="column"><FormField label="新規文書サブパス"><input class="input" type="text" prop:value=move || form_new_doc_sub_path.get() on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_new_doc_sub_path.set(t.value()); } /></FormField></div>
                                <div class="column"><FormField label="文書番号パターン"><input class="input" type="text" prop:value=move || form_doc_number_pattern.get() on:input=move |ev| { let t: HtmlInputElement = event_target(&ev); form_doc_number_pattern.set(t.value()); } /></FormField></div>
                            </div>
                            <div class="field is-grouped">
                                <div class="control"><button class="button is-primary" type="submit" prop:disabled=move || saving.get()>"保存"</button></div>
                                <div class="control"><button class="button" type="button" on:click=move |_| reset_form()>"キャンセル"</button></div>
                            </div>
                        </form>
                    </div>
                }.into_any()
            } else { view! { <div></div> }.into_any() }}

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
                                        <thead><tr><th>"登録コード"</th><th>"文書種別"</th><th>"部署"</th><th>"サーバールート"</th>{if is_admin { view! { <th>"操作"</th> }.into_any() } else { view! { <th></th> }.into_any() }}</tr></thead>
                                        <tbody>
                                            {paginated.data.into_iter().map(|dr| {
                                                let dr_clone = dr.clone();
                                                view! {
                                                    <tr>
                                                        <td><span class="tag is-light">{dr.register_code}</span></td>
                                                        <td>{dr.doc_kind.name}</td>
                                                        <td>{dr.department.name}</td>
                                                        <td class="is-size-7">{dr.file_server_root}</td>
                                                        <td>{if is_admin { view! { <button class="button is-small is-info is-outlined" on:click=move |_| start_edit(dr_clone.clone())><span class="icon"><i class="fas fa-edit"></i></span></button> }.into_any() } else { view! { <span></span> }.into_any() }}</td>
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
