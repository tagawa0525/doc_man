use leptos::prelude::*;

#[component]
pub fn Pagination(
    current_page: u32,
    total: i64,
    per_page: u32,
    #[prop(into)] on_page_change: Callback<u32>,
) -> impl IntoView {
    let total_pages = ((total as f64) / f64::from(per_page)).ceil() as u32;

    if total_pages <= 1 {
        return view! { <div></div> }.into_any();
    }

    let pages: Vec<u32> = {
        let mut p = Vec::new();
        let start = current_page.saturating_sub(2).max(1);
        let end = (current_page + 2).min(total_pages);
        for i in start..=end {
            p.push(i);
        }
        p
    };

    view! {
        <nav class="pagination is-centered" role="navigation" aria-label="pagination">
            <button
                class="pagination-previous"
                prop:disabled=move || current_page <= 1
                on:click=move |_| {
                    if current_page > 1 {
                        on_page_change.run(current_page - 1);
                    }
                }
            >
                "前へ"
            </button>
            <button
                class="pagination-next"
                prop:disabled=move || current_page >= total_pages
                on:click=move |_| {
                    if current_page < total_pages {
                        on_page_change.run(current_page + 1);
                    }
                }
            >
                "次へ"
            </button>
            <ul class="pagination-list">
                {pages.into_iter().map(|p| {
                    let is_current = p == current_page;
                    let class = if is_current { "pagination-link is-current" } else { "pagination-link" };
                    view! {
                        <li>
                            <button class=class on:click=move |_| on_page_change.run(p)>
                                {p}
                            </button>
                        </li>
                    }
                }).collect_view()}
            </ul>
        </nav>
    }.into_any()
}
