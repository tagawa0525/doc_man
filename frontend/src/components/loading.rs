use leptos::prelude::*;

#[component]
pub fn Loading() -> impl IntoView {
    view! {
        <div class="spinner-overlay">
            <span class="icon is-large">
                <i class="fas fa-spinner fa-pulse fa-3x"></i>
            </span>
        </div>
    }
}
