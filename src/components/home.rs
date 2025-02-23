use chrono::Local;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::server::api::{get_activity_latest, get_runs_latest};

#[component]
pub fn HomePage() -> impl IntoView {
    let runs = OnceResource::new(get_runs_latest(0));
    let rankings = OnceResource::new(get_activity_latest(0));

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
            <h2>"Activity"</h2>
            <div class="row">
                <div class="users">
                    <h3>"Rankings"</h3>
                    <Suspense fallback=|| {
                        "Loading..."
                    }>
                        {move || {
                            rankings
                                .get()
                                .map(|res| {
                                    res.map(|rankings| {
                                        rankings
                                            .into_iter()
                                            .take(5)
                                            .map(|act| {
                                                let diff = Local::now() - act.created_at;
                                                view! {
                                                    <div class="column">
                                                        <div class="row">
                                                            <h5>
                                                                {if let Some(title) = &act.title_old {
                                                                    title.to_string()
                                                                } else if let Some(rank) = act.rank_old {
                                                                    format!("#{}", rank.to_string())
                                                                } else {
                                                                    String::from("User Joined")
                                                                }}
                                                            </h5>
                                                            <h6>
                                                                {if act.title_old.is_some() || act.rank_old.is_some() {
                                                                    "➡"
                                                                } else {
                                                                    ""
                                                                }}
                                                            </h6>
                                                            <h5>
                                                                {if let Some(title) = act.title_new {
                                                                    title.to_string()
                                                                } else if let Some(rank) = act.rank_new {
                                                                    format!("#{}", rank.to_string())
                                                                } else {
                                                                    String::new()
                                                                }}
                                                            </h5>
                                                        </div>
                                                        <div class="row">
                                                            <p>"for " {act.username}</p>
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
                <div class="runs">
                    <h3>"Submissions"</h3>
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
