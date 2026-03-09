use leptos::prelude::*;

#[component]
pub fn FormField(#[prop(into)] label: String, children: Children) -> impl IntoView {
    view! {
        <div class="field">
            <label class="label">{label}</label>
            <div class="control">
                {children()}
            </div>
        </div>
    }
}
