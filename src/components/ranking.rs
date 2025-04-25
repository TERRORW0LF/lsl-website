use leptos::prelude::*;

#[component]
pub fn Ranking(
    patches: Signal<Vec<(String, String)>>,
    layouts: Signal<Vec<(String, String)>>,
    patch: Signal<String>,
    layout: Signal<String>,
) -> impl IntoView {
    view! { <section id="ranking"></section> }
}

#[component]
pub fn UserRanking(id: Signal<i64>, patches: Signal<Vec<(String, String)>>) -> impl IntoView {
    view! {
        <section id="ranking">
            <h1>"Coming"</h1>
            <img src="/soon.jpg" alt="soon tm" />
        </section>
    }
}
