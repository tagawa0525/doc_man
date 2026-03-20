use leptos::prelude::*;
use web_sys::HtmlInputElement;

use crate::api::client;
use crate::api::types::MeResponse;
use crate::auth::{AuthContext, Role, UserInfo};
use crate::components::toast::ToastContext;

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let toast = expect_context::<ToastContext>();
    let employee_code = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let code = employee_code.get_untracked();
        if code.is_empty() {
            error_msg.set(Some("社員コードを入力してください".to_string()));
            return;
        }

        loading.set(true);
        error_msg.set(None);

        leptos::task::spawn_local(async move {
            client::set_token(&code);
            match client::get::<MeResponse>("/api/v1/me").await {
                Ok(me) => {
                    auth.user.set(Some(UserInfo {
                        id: me.id,
                        name: me.name,
                        role: Role::from_str(&me.role),
                        departments: me.departments,
                    }));
                    toast.success("ログインしました");
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
                Err(e) => {
                    client::clear_token();
                    error_msg.set(Some(format!("ログインに失敗しました: {}", e.message)));
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="login-container">
            <div class="login-box">
                <h1 class="title has-text-centered">
                    <span class="icon is-large"><i class="fas fa-file-contract"></i></span>
                    <br />
                    "Doc Man"
                </h1>
                <p class="subtitle has-text-centered has-text-grey">"文書管理システム"</p>

                <form on:submit=on_submit>
                    <div class="field">
                        <label class="label">"社員コード"</label>
                        <div class="control has-icons-left">
                            <input
                                class="input"
                                type="text"
                                placeholder="社員コードを入力"
                                prop:value=move || employee_code.get()
                                on:input=move |ev| {
                                    let target: HtmlInputElement = event_target(&ev);
                                    employee_code.set(target.value());
                                }
                                prop:disabled=move || loading.get()
                            />
                            <span class="icon is-small is-left">
                                <i class="fas fa-id-badge"></i>
                            </span>
                        </div>
                    </div>

                    {move || error_msg.get().map(|msg| view! {
                        <div class="notification is-danger is-light">
                            {msg}
                        </div>
                    })}

                    <div class="field">
                        <div class="control">
                            <button
                                class="button is-primary is-fullwidth"
                                type="submit"
                                prop:disabled=move || loading.get()
                            >
                                {move || if loading.get() {
                                    "ログイン中...".to_string()
                                } else {
                                    "ログイン".to_string()
                                }}
                            </button>
                        </div>
                    </div>
                </form>
            </div>
        </div>
    }
}
