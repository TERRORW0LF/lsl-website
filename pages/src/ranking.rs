use components::{Collapsible, Header, ListElements, RankingLegend};
use leptos::prelude::*;
use leptos_router::components::A;
use server::api::get_rankings;
use types::api::Title;

#[component]
pub fn RankingHeader(#[prop(into)] links: Signal<Vec<(String, String)>>) -> impl IntoView {
    move || {
        let links = links.get();
        view! {
            <Header attr:id="ranking-header">
                <ListElements>
                    {links
                        .into_iter()
                        .map(|l| {
                            view! {
                                <A href=l.0 scroll=false>
                                    <span class="text">{l.1}</span>
                                </A>
                            }
                        })
                        .collect_view()}
                </ListElements>
                // Workaround for ChildrenFragment not accepting single
                // components
                <div></div>
            </Header>
        }
    }
}

#[component]
pub fn ComboRanking(
    #[prop(into)] patch: Signal<String>,
    #[prop(into)] layout: Signal<Option<String>>,
    #[prop(into)] categories: Signal<Vec<(Option<String>, String)>>,
) -> impl IntoView {
    let rankings = Signal::derive(move || {
        categories.get().into_iter().map(move |category| {
            Resource::new(
                move || (patch.get(), layout.get(), category.clone()),
                async move |combos| get_rankings(combos.0.clone(), combos.1.clone(), combos.2.0).await,
            )
        })
    });
    let titles = vec![
        (
            Title::Surfer,
            Signal::<String>::derive(move || match patch.get().as_str() {
                "1.00" | "1.41" | "1.50" => "300 or more rating points".into(),
                "2.00" => "150 or more rating points".into(),
                "2.13" => "1500 or more rating points".into(),
                _ => "Unknown patch selected".into(),
            }),
        ),
        (
            Title::SuperSurfer,
            Signal::<String>::derive(move || match patch.get().as_str() {
                "1.00" | "1.41" | "1.50" => "1000 or more rating points".into(),
                "2.00" => "500 or more rating points".into(),
                "2.13" => "3000 or more rating points".into(),
                _ => "Unknown patch selected".into(),
            }),
        ),
        (
            Title::EpicSurfer,
            Signal::<String>::derive(move || match patch.get().as_str() {
                "1.00" | "1.41" | "1.50" => "2000 or more rating points".into(),
                "2.00" => "1000 or more rating points".into(),
                "2.13" => "5000 or more rating points".into(),
                _ => "Unknown patch selected".into(),
            }),
        ),
        (
            Title::LegendarySurfer,
            Signal::<String>::derive(move || match patch.get().as_str() {
                "1.00" | "1.41" | "1.50" => "4000 or more rating points".into(),
                "2.00" => "2000 or more rating points".into(),
                "2.13" => "7500 or more rating points".into(),
                _ => "Unknown patch selected".into(),
            }),
        ),
        (
            Title::MythicSurfer,
            Signal::<String>::derive(move || match patch.get().as_str() {
                "1.00" | "1.41" | "1.50" => "5500 or more rating points".into(),
                "2.00" => "2750 or more rating points".into(),
                "2.13" => "9000 or more rating points".into(),
                _ => "Unknown patch selected".into(),
            }),
        ),
        (Title::TopOne, "Most points in the ranking".into()),
    ];
    view! {
        <section class="ranking">
            {move || {
                let mut c = 0;
                rankings
                    .get()
                    .map(|rs| {
                        c += 1;
                        view! {
                            <div class="ranking">
                                <h3 class="category">
                                    {move || categories.get()[c - 1].1.clone()}
                                </h3>
                                <RankingLegend titles=titles.clone() />
                                <Suspense fallback=move || { "Loading..." }>
                                    <ErrorBoundary fallback=move |_| { "Error fetching data." }>
                                        <Collapsible id="more" swapped=true>
                                            {move || {
                                                rs.and_then(|rs| {
                                                    rs.clone()
                                                        .into_iter()
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
                                                                    <A href=format!("/user/{}/ranking", r.user_id)>
                                                                        <h4 class="name">{r.username}</h4>
                                                                    </A>
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
                                                })
                                            }}
                                        </Collapsible>
                                    </ErrorBoundary>
                                </Suspense>
                            </div>
                        }
                    })
                    .collect_view()
            }}
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
