use leptos::prelude::*;

#[component]
pub fn Header(children: ChildrenFragment) -> impl IntoView {
    let iter = children().nodes.into_iter();
    let len = iter.len();
    view! {
        <header>
            <nav class="row nav">
                {iter
                    .enumerate()
                    .map(|(i, v)| {
                        view! {
                            <ul
                                class:middle=i != 0 && i != len - 1
                                class:right=i == len - 1 && len != 1
                            >
                                {v}
                            </ul>
                        }
                    })
                    .collect_view()}
            </nav>
        </header>
    }
}

#[component]
pub fn ListElements(children: ChildrenFragment) -> impl IntoView {
    children()
        .nodes
        .into_iter()
        .map(|v| {
            view! { <li>{v}</li> }
        })
        .collect_view()
}
