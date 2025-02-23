use chrono::Local;
use leptos::{
    either::{Either, EitherOf3},
    prelude::*,
};
use leptos_meta::Title;
use leptos_router::components::A;

use crate::server::api::{get_activity_latest, get_potd, get_runs_latest};

#[component]
pub fn HomePage() -> impl IntoView {
    let runs = OnceResource::new(get_runs_latest(0));
    let rankings = OnceResource::new(get_activity_latest(0));
    let potd = OnceResource::new(get_potd());

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
                                                    <div class="row">
                                                        <div class="column">
                                                            <div class="row">
                                                                {if let Some(title) = &act.title_old {
                                                                    EitherOf3::A(
                                                                        view! {
                                                                            <h5 class=title.to_string()>{title.to_string()}</h5>
                                                                        },
                                                                    )
                                                                } else if let Some(rank) = act.rank_old {
                                                                    EitherOf3::B(
                                                                        view! {
                                                                            <h5 class=format!(
                                                                                "rank-{rank}",
                                                                            )>{format!("#{}", rank.to_string())}</h5>
                                                                        },
                                                                    )
                                                                } else {
                                                                    EitherOf3::C(view! { <h5>"User Joined"</h5> })
                                                                }}
                                                                <h6>
                                                                    {if act.title_old.is_some() || act.rank_old.is_some() {
                                                                        "to"
                                                                    } else {
                                                                        ""
                                                                    }}
                                                                </h6>
                                                                {if let Some(title) = &act.title_new {
                                                                    EitherOf3::A(
                                                                        view! {
                                                                            <h5 class=title.to_string()>{title.to_string()}</h5>
                                                                        },
                                                                    )
                                                                } else if let Some(rank) = act.rank_new {
                                                                    EitherOf3::B(
                                                                        view! {
                                                                            <h5 class=format!(
                                                                                "rank-{rank}",
                                                                            )>{format!("#{}", rank.to_string())}</h5>
                                                                        },
                                                                    )
                                                                } else {
                                                                    EitherOf3::C(())
                                                                }}
                                                            </div>
                                                            <h6>"for " {act.username}</h6>
                                                        </div>
                                                        <div class="column">
                                                            <p>
                                                                {if let Some(l) = act.layout {
                                                                    Either::Left(format!("Layout {l}"))
                                                                } else {
                                                                    Either::Right(view! { <br /> })
                                                                }}
                                                            </p>
                                                            <p>
                                                                {if let Some(c) = act.category {
                                                                    Either::Left(c)
                                                                } else {
                                                                    Either::Right(view! { <br /> })
                                                                }}
                                                            </p>
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
            <Suspense fallback=|| "Loading...">
                <div>
                    <img src=move || {
                        format!(
                            "/cdn/users/{}.jpg",
                            potd
                                .get()
                                .map(|res| res.map(|u| u.pfp).ok())
                                .flatten()
                                .unwrap_or(String::from("default")),
                        )
                    } />
                    <p>{move || potd.get().map(|res| res.map(|u| u.bio.unwrap()))}</p>
                </div>
            </Suspense>
        </section>
    }
}
