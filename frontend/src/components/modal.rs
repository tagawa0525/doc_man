use leptos::prelude::*;

#[component]
pub fn ConfirmModal(
    #[prop(into)] title: String,
    #[prop(into)] message: String,
    #[prop(into)] on_confirm: Callback<()>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(optional)] danger: bool,
) -> impl IntoView {
    let confirm_class = if danger {
        "button is-danger"
    } else {
        "button is-primary"
    };

    view! {
        <div class="modal is-active">
            <div class="modal-background" on:click=move |_| on_cancel.run(())></div>
            <div class="modal-card">
                <header class="modal-card-head">
                    <p class="modal-card-title">{title}</p>
                    <button class="delete" aria-label="close" on:click=move |_| on_cancel.run(())></button>
                </header>
                <section class="modal-card-body">
                    <p>{message}</p>
                </section>
                <footer class="modal-card-foot">
                    <button class=confirm_class on:click=move |_| on_confirm.run(())>"確認"</button>
                    <button class="button" on:click=move |_| on_cancel.run(())>"キャンセル"</button>
                </footer>
            </div>
        </div>
    }
}
