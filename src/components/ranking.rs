use crate::server::api::get_rankings;
use futures::future::join_all;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn RankingHeader(
    #[prop(into)] patches: Signal<Vec<(String, String)>>,
    #[prop(into)] layouts: Signal<Vec<(String, String)>>,
) -> impl IntoView {
    view! {
        <header id="ranking-header">
            <nav class="split-row-nav">
                <ul class="left-row-nav">
                    <For
                        each=patches
                        key=|p| p.0.to_owned()
                        children=move |p| {
                            view! {
                                <li>
                                    <A href=format!("../../{}/1", p.0) scroll=false>
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
                                    <A href=format!("../{}", l.0) scroll=false>
                                        <span class="text">{l.1}</span>
                                    </A>
                                </li>
                            }
                        }
                    />
                </ul>
            </nav>
        </header>
    }
}

#[component]
pub fn Ranking(
    #[prop(into)] patch: Signal<String>,
    #[prop(into)] layout: Signal<String>,
    #[prop(into)] categories: Signal<Vec<(String, String)>>,
) -> impl IntoView {
    let rankings = Resource::new(
        move || (patch.get(), layout.get(), categories.get()),
        async |combos| {
            let mut res = vec![];
            for c in combos.2 {
                res.push(get_rankings(combos.0.clone(), combos.1.clone(), c.0));
            }
            join_all(res).await
        },
    );
    view! {
        <section class="ranking">
            <Suspense fallback=move || {
                "Loading..."
            }>
                {move || {
                    rankings
                        .get()
                        .map(|rs| {
                            let mut c = 0;
                            rs.into_iter()
                                .map(|rs| {
                                    c += 1;
                                    view! {
                                        <div class="ranking">
                                            <h3 class="category">
                                                {move || categories.get()[c - 1].1.clone()}
                                            </h3>
                                            <ErrorBoundary fallback=move |_| { "Error fetching data." }>
                                                <div class="columns">
                                                    {rs
                                                        .map(|rs| {
                                                            rs.into_iter()
                                                                .map(|r| {
                                                                    view! {
                                                                        <div class="grid-row">
                                                                            <div
                                                                                class=format!("rank {} bg", r.title.to_string())
                                                                                class=("rank-1", r.rank == 1)
                                                                                class=("rank-2", r.rank == 2)
                                                                                class=("rank-3", r.rank == 3)
                                                                            >
                                                                                <h5>{r.rank}</h5>
                                                                            </div>
                                                                            <h4 class="name">{r.username}</h4>
                                                                            <div class="rating row narrow">
                                                                                <h4 class=format!(
                                                                                    "{} color",
                                                                                    r.title.to_string(),
                                                                                )>{format!("{}", r.rating.round())}</h4>
                                                                                <h6>"RP"</h6>
                                                                            </div>
                                                                        </div>
                                                                    }
                                                                })
                                                                .collect_view()
                                                        })}
                                                </div>
                                            </ErrorBoundary>
                                        </div>
                                    }
                                })
                                .collect_view()
                        })
                }}
            </Suspense>
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
