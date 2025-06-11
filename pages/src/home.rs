use chrono::Local;
use leptos::{
    either::{Either, EitherOf3},
    prelude::*,
};
use leptos_meta::Title;
use leptos_router::components::A;

use server::api::{get_activity, get_rand_user, get_runs};
use types::api::{ActivityFilters, RunFilters};

#[component]
pub fn HomePage() -> impl IntoView {
    let runs = OnceResource::new(get_runs(RunFilters::default(), 0));
    let rankings = OnceResource::new(get_activity(ActivityFilters::default(), 0));
    let potd = OnceResource::new(get_rand_user());

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
                    <A href="/faq#play" attr:class="button magic">
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
                                                                            <h5 class=format!(
                                                                                "{} color",
                                                                                title.to_string(),
                                                                            )>{title.to_string()}</h5>
                                                                        },
                                                                    )
                                                                } else if let Some(rank) = act.rank_old {
                                                                    EitherOf3::B(
                                                                        view! {
                                                                            <h5 class=format!(
                                                                                "rank-{rank} color",
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
                                                                            <h5 class=format!(
                                                                                "{} color",
                                                                                title.to_string(),
                                                                            )>{title.to_string()}</h5>
                                                                        },
                                                                    )
                                                                } else if let Some(rank) = act.rank_new {
                                                                    EitherOf3::B(
                                                                        view! {
                                                                            <h5 class=format!(
                                                                                "rank-{rank} color",
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
                    <A href="/activity" attr:class="button secondary">
                        "Show More"
                    </A>
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
                                                            <A href=format!("/leaderboard/map/{}", run.section_id)>
                                                                <h5>{run.map}</h5>
                                                            </A>
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
                    <A href="/runs" attr:class="button secondary">
                        "Show More"
                    </A>
                </div>
            </div>
        </section>
        <section id="potd">
            <h2>"Player Showcase"</h2>
            <Suspense fallback=|| "Loading...">
                <div class="row">
                    <div class="user">
                        {move || {
                            potd.get()
                                .map(|res| {
                                    res.map(|u| {
                                        let rank = u
                                            .ranks
                                            .iter()
                                            .filter(|r| {
                                                r.patch == String::from("2.13") && r.layout.is_none()
                                            })
                                            .next();
                                        view! {
                                            <div class="row narrow">
                                                <A href=format!("/user/{}/leaderboard", u.id)>
                                                    <h3>{u.username}</h3>
                                                </A>
                                                {if let Some(r) = rank {
                                                    Either::Left(
                                                        view! {
                                                            <div class="row narrow">
                                                                <h6 class=format!("rank-{} color", r.rank)>"#" {r.rank}</h6>
                                                                <h6 class=format!(
                                                                    "{} color",
                                                                    r.title.to_string(),
                                                                )>{r.title.to_string()}</h6>
                                                            </div>
                                                        },
                                                    )
                                                } else {
                                                    Either::Right(())
                                                }}
                                            </div>
                                            <img src=format!("/cdn/users/{}.jpg", u.pfp) />
                                            <p>{u.bio.unwrap()}</p>
                                        }
                                    })
                                })
                        }}
                    </div>
                    <div class="row narrow">
                        <div class="ranks">
                            <h4>"Standard"</h4>
                            {move || {
                                potd.get()
                                    .map(|res| {
                                        res.map(|mut u| {
                                            u.ranks.sort_by_key(|r| r.layout.clone());
                                            u.ranks
                                                .iter()
                                                .filter(|r| {
                                                    r.patch == String::from("2.13")
                                                        && r.category == Some(String::from("Standard"))
                                                })
                                                .map(|r| {
                                                    view! {
                                                        <div class="row">
                                                            <div class="column">
                                                                <h5 class=format!("rank-{} color", r.rank)>"#"{r.rank}</h5>
                                                                <h6>"Layout "{r.layout.clone()}</h6>
                                                            </div>
                                                            <div class="column">
                                                                <h5 class=format!(
                                                                    "{} color",
                                                                    r.title.to_string(),
                                                                )>{r.title.to_string()}</h5>
                                                                <h6>{format!("{:.0} Rating", r.rating)}</h6>
                                                            </div>
                                                        </div>
                                                    }
                                                })
                                                .collect_view()
                                        })
                                    })
                            }}
                        </div>
                        <div class="ranks">
                            <h4>"Gravspeed"</h4>
                            {move || {
                                potd.get()
                                    .map(|res| {
                                        res.map(|mut u| {
                                            u.ranks.sort_by_key(|r| r.layout.clone());
                                            u.ranks
                                                .iter()
                                                .filter(|r| {
                                                    r.patch == String::from("2.13")
                                                        && r.category == Some(String::from("Gravspeed"))
                                                })
                                                .map(|r| {
                                                    view! {
                                                        <div class="row">
                                                            <div class="column">
                                                                <h5 class=format!("rank-{} color", r.rank)>"#"{r.rank}</h5>
                                                                <h6>"Layout "{r.layout.clone()}</h6>
                                                            </div>
                                                            <div class="column">
                                                                <h5 class=format!(
                                                                    "{} color",
                                                                    r.title.to_string(),
                                                                )>{r.title.to_string()}</h5>
                                                                <h6>{format!("{:.0} Rating", r.rating)}</h6>
                                                            </div>
                                                        </div>
                                                    }
                                                })
                                                .collect_view()
                                        })
                                    })
                            }}
                        </div>
                    </div>
                </div>
            </Suspense>
        </section>
    }
}
