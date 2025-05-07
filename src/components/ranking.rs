use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Ranking(
    #[prop(into)] patches: Signal<Vec<(String, String)>>,
    #[prop(into)] layouts: Signal<Vec<(String, String)>>,
    #[prop(into)] patch: Signal<String>,
    #[prop(into)] layout: Signal<String>,
) -> impl IntoView {
    view! {
        <section id="ranking">
            <header id="lb_header">
                <nav class="split-row-nav">
                    <ul class="left-row-nav">
                        <For
                            each=patches
                            key=|p| p.0.to_owned()
                            children=move |p| {
                                view! {
                                    <li>
                                        <A
                                            href=move || { format!("../../{}/{}", p.0, layout.get()) }
                                            scroll=false
                                        >
                                            <span class="text">{p.1}</span>
                                        </A>
                                    </li>
                                }
                            }
                        />
                    </ul>
                    <ul class="right-row-nav">
                        <For
                            each=layouts
                            key=|l| l.0.to_owned()
                            children=move |l| {
                                view! {
                                    <li>
                                        <A href=move || { format!("../{}", l.0) } scroll=false>
                                            <span class="text">{l.1}</span>
                                        </A>
                                    </li>
                                }
                            }
                        />
                    </ul>
                </nav>
            </header>
        </section>
    }
}

#[component]
pub fn UserRanking(
    #[prop(into)] id: Signal<i64>,
    #[prop(into)] patches: Signal<Vec<(String, String)>>,
) -> impl IntoView {
    view! {
        <section id="ranking">
            <h1>"Coming"</h1>
            <img src="/soon.jpg" alt="soon tm" />
        </section>
    }
}
