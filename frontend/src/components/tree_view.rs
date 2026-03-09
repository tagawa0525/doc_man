use leptos::prelude::*;
use uuid::Uuid;

use crate::api::types::DepartmentTree;

#[component]
pub fn TreeView(
    departments: Vec<DepartmentTree>,
    #[prop(into)] on_select: Callback<Uuid>,
    selected_id: Option<Uuid>,
) -> impl IntoView {
    view! {
        <div class="tree-node-root">
            {departments.into_iter().map(|dept| {
                view! {
                    <TreeNode dept=dept on_select=on_select selected_id=selected_id />
                }
            }).collect_view()}
        </div>
    }
}

#[component]
fn TreeNode(
    dept: DepartmentTree,
    #[prop(into)] on_select: Callback<Uuid>,
    selected_id: Option<Uuid>,
) -> impl IntoView {
    let expanded = RwSignal::new(true);
    let has_children = !dept.children.is_empty();
    let id = dept.id;
    let is_selected = selected_id == Some(id);
    let name = dept.name.clone();
    let code = dept.code.clone();
    let children = dept.children;

    let node_class = if is_selected {
        "has-text-weight-bold has-text-primary"
    } else {
        ""
    };

    view! {
        <div class="tree-node">
            <div class="is-flex is-align-items-center mb-1">
                {if has_children {
                    view! {
                        <span class="tree-toggle mr-1" on:click=move |_| expanded.update(|v| *v = !*v)>
                            <span class="icon is-small">
                                {move || if expanded.get() {
                                    view! { <i class="fas fa-caret-down"></i> }.into_any()
                                } else {
                                    view! { <i class="fas fa-caret-right"></i> }.into_any()
                                }}
                            </span>
                        </span>
                    }.into_any()
                } else {
                    view! { <span class="mr-1" style="width:1.5rem;display:inline-block;"></span> }.into_any()
                }}
                <a class=node_class on:click=move |_| on_select.run(id)>
                    <span class="tag is-light mr-1">{code}</span>
                    {name}
                </a>
            </div>
            {move || if expanded.get() && has_children {
                let children_clone = children.clone();
                view! {
                    <div class="tree-node">
                        {children_clone.into_iter().map(|child| {
                            view! {
                                <TreeNode dept=child on_select=on_select selected_id=selected_id />
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}
