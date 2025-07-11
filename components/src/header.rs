use leptos::prelude::*;
use types::leptos::ViewFragment;

#[component]
pub fn Header(#[prop(optional, into)] right: ViewFragment, children: ChildrenFragment) -> impl IntoView {
    view! {
        <header>
            <nav class="row nav">
                <ul class="row narrow">
                    {children().nodes.into_iter().map(|e| view! { <li>{e}</li> }).collect_view()}
                </ul>
                <ul class="row narrow right">
                    {right.run().nodes.into_iter().map(|e| view! { <li>{e}</li> }).collect_view()}
                </ul>
            </nav>
        </header>
    }
}
