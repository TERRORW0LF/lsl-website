use chrono::Local;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::server::api::get_runs_latest;

#[component]
pub fn HomePage() -> impl IntoView {
    let runs = OnceResource::new(get_runs_latest(0));

    view! {
        <Title text="Home" />
        <section id="welcome">
            <div class="container">
                <h1>"Welcome to the Lucio Surf League!"</h1>
                <p>
                    "Since its inception in 2019 "<b>"Lucio Surf"</b>
                    " has been a staple in Overwatch's custom game browser. The "
                    <b>"Lucio Surf League"</b>
                    " has been the proud maintainer and arbitrator of Lucio Surf since the beginning. "
                    "Updating the game mode and leaderboards, connecting the community, and holding tournaments, the Lucio Surf League "
                    "is your place for anything related to Lucio Surf. We welcome you as another frog in our ranks."
                </p>
                <div class="row narrow">
                    <A href="/leaderboard" attr:class="button magic">
                        "Leaderboard"
                    </A>
                    <a href="https://learn.lucio.surf/" class="button magic">
                        "Academy"
                    </a>
                    <A href="/workshop" attr:class="button magic">
                        "Play"
                    </A>
                </div>
            </div>
        </section>
        <section id="activity">
            <div class="runs">
                <Suspense fallback=|| {
                    "Loading..."
                }>
                    {move || {
                        runs.get()
                            .map(|res| {
                                res.map(|runs| {
                                    runs.into_iter()
                                        .take(5)
                                        .map(|run| {
                                            let diff = Local::now() - run.created_at;
                                            view! {
                                                <div class="row">
                                                    <a href=run.proof target="_blank" class="play"></a>
                                                    <div class="column">
                                                        <h5>{run.map}</h5>
                                                        <h6>{run.time.to_string()} " sec by " {run.username}</h6>
                                                    </div>
                                                    <div class="column">
                                                        <p>"Layout " {run.layout}</p>
                                                        <p>{run.category}</p>
                                                        <p>
                                                            {if diff.num_hours() == 0 {
                                                                format!(
                                                                    "{}m {}s",
                                                                    diff.num_minutes(),
                                                                    diff.num_seconds() - diff.num_minutes() * 60,
                                                                )
                                                            } else if diff.num_days() == 0 {
                                                                format!(
                                                                    "{}h {}m",
                                                                    diff.num_hours(),
                                                                    diff.num_minutes() - diff.num_hours() * 60,
                                                                )
                                                            } else {
                                                                format!(
                                                                    "{}d {}h",
                                                                    diff.num_days(),
                                                                    diff.num_hours() - diff.num_days() * 24,
                                                                )
                                                            }} " ago"
                                                        </p>
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect_view()
                                })
                            })
                    }}
                </Suspense>
            </div>
        </section>
        <section id="potd">
            <h2>"Player of the day"</h2>
            <div>
                <img src="" />
                <p>
                    "Test is an exceptional player combining superb routing with steady movement. This allows them to top "
                    "any leaderboard that don't involve boosts. While occasionaly dabbling in Gravspeed their main category "
                    "is Standard, prefering the challenge over the speed."
                </p>
            </div>
        </section>
    }
}
