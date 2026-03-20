use leptos::prelude::*;

#[component]
pub fn StatusBadge(#[prop(into)] status: String) -> impl IntoView {
    let (class, label) = match status.as_str() {
        "draft" => ("tag status-draft", "下書き"),
        "under_review" => ("tag status-under-review", "レビュー中"),
        "approved" => ("tag status-approved", "承認済"),
        "rejected" => ("tag status-rejected", "却下"),
        _ => ("tag", status.as_str()),
    };

    let label = label.to_string();

    view! {
        <span class=class>{label}</span>
    }
}
