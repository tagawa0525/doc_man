use leptos::prelude::*;

#[component]
pub fn StatusBadge(#[prop(into)] status: String) -> impl IntoView {
    let (class, label) = match status.as_str() {
        "draft" => ("tag status-draft", "下書き"),
        "in_review" => ("tag status-in-review", "レビュー中"),
        "approved" => ("tag status-approved", "承認済"),
        "rejected" => ("tag status-rejected", "却下"),
        "superseded" => ("tag status-superseded", "旧版"),
        _ => ("tag", status.as_str()),
    };

    // For unknown status, we need to handle the lifetime issue
    let label = label.to_string();

    view! {
        <span class=class>{label}</span>
    }
}
