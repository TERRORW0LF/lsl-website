use leptos::{either::*, prelude::*};
use leptos_meta::Title;
use leptos_router::{
    components::{Form, Outlet, A},
    hooks::{use_params_map, use_query_map},
};
use rust_decimal::Decimal;
use std::{cmp::Ordering, collections::HashMap};

use crate::server::api::{get_runs_category, MapRuns, PartialRun};

#[component]
pub fn Section(
    patch: ReadSignal<String>,
    layout: ReadSignal<String>,
    category: ReadSignal<String>,
    map: ReadSignal<Option<String>>,
) -> impl IntoView {
    let layouts = move || match patch.get().as_str() {
        "1.00" => vec![1, 2],
        "1.41" => vec![1, 2, 3, 4],
        "1.50" => vec![1, 2, 3, 4, 5],
        "2.00" => vec![1, 2, 3, 4, 5],
        "2.13" => vec![1, 2, 3, 4, 5],
        _ => vec![],
    };

    let query = use_query_map();

    view! {
        <Title text="Leaderboard" />
        <section id="leaderboard">
            <details>
                <summary>
                    <span role="term" aria-details="filters" class="icon">
                        ""
                    </span>
                </summary>
            </details>
            <header id="lb_header">
                <nav class="split-row-nav">
                    <div class="left-row-nav">
                        <For
                            each=layouts
                            key=|l| l.to_owned()
                            children=move |l| {
                                view! {
                                    <A href=move || {
                                        format!(
                                            "{}/{l}/{}{}{}",
                                            patch.get(),
                                            category.get(),
                                            map.get().map_or(String::new(), |m| format!("/{m}")),
                                            query.get().to_query_string(),
                                        )
                                    }>
                                        <span class="text">"Layout " {l}</span>
                                    </A>
                                }
                            }
                        />
                    </div>
                    <div class="right-row-nav">
                        <A href=move || {
                            format!(
                                "{}/{}/standard{}{}",
                                patch.get(),
                                layout.get(),
                                map.get().map_or(String::new(), |m| format!("/{m}")),
                                query.get().to_query_string(),
                            )
                        }>
                            <span class="text">"Standard"</span>
                        </A>
                        <A href=move || {
                            format!(
                                "{}/{}/gravspeed{}{}",
                                patch.get(),
                                layout.get(),
                                map.get().map_or(String::new(), |m| format!("/{m}")),
                                query.get().to_query_string(),
                            )
                        }>
                            <span class="text">"Gravspeed"</span>
                        </A>
                    </div>
                </nav>
            </header>
            <div role="definition" id="filters" class="content">
                <Form
                    method="GET"
                    action=move || {
                        format!(
                            "{}/{}/{}{}",
                            patch.get(),
                            layout.get(),
                            category.get(),
                            map.get().map_or(String::new(), |m| format!("/{m}")),
                        )
                    }
                >
                    <div class="group">
                        <h6>"Patch"</h6>
                        <div class="options">
                            <A href="1.00">"1.00"</A>
                            <A href="1.41">"1.41"</A>
                            <A href="1.50">"1.50"</A>
                            <A href="2.00">"2.00"</A>
                            <A href="2.13">"Current"</A>
                        </div>
                    </div>
                    <div class="group">
                        <h6>"Sort by"</h6>
                        <div class="options">
                            <input type="radio" name="sort" value="time" id="time" />
                            <label for="time">"Time"</label>

                            <input type="radio" name="sort" value="date" id="date" />
                            <label for="date">"Date"</label>
                        </div>
                    </div>
                    <div class="group">
                        <h6>"Filter"</h6>
                        <div class="options">
                            <input type="radio" name="filter" value="none" id="none" />
                            <label for="none">"None"</label>
                            <input type="radio" name="filter" value="is_pb" id="is_pb" />
                            <label for="is_pb">"Is PB"</label>
                            <input type="radio" name="filter" value="is_wr" id="is_wr" />
                            <label for="is_wr">"Is WR"</label>
                            <input type="radio" name="filter" value="was_pb" id="was_pb" />
                            <label for="was_pb">"Was PB"</label>
                            <input type="radio" name="filter" value="was_wr" id="was_wr" />
                            <label for="was_wr">"Was WR"</label>
                            <input type="radio" name="filter" value="verified" id="verified" />
                            <label for="verified">"Verified"</label>
                        </div>
                    </div>
                    <input type="submit" class="button" value="Apply" />
                </Form>
            </div>
            <Outlet />
        </section>
    }
}

#[component]
pub fn Leaderboard(
    patch: WriteSignal<String>,
    layout: WriteSignal<String>,
    category: WriteSignal<String>,
    map: WriteSignal<Option<String>>,
) -> impl IntoView {
    let params = use_params_map();
    Effect::new(move |_| {
        let params = params.read();
        patch.set(params.get("patch").unwrap());
        layout.set(params.get("layout").unwrap());
        category.set(params.get("category").unwrap());
        map.set(None);
    });
    let selection = Memo::new(move |_| {
        let params = params.read();
        (
            params.get("patch").unwrap(),
            params.get("layout").unwrap(),
            params.get("category").unwrap(),
        )
    });
    let maps = Resource::new(selection, |mut s| {
        get_runs_category(s.0, s.1, format!("{}{}", s.2.remove(0).to_uppercase(), s.2))
    });

    view! {
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || {
                maps.get()
                    .map(|data| match data {
                        Err(e) => Either::Right(view! { <p>{e.to_string()}</p> }),
                        Ok(maps) => {
                            Either::Left(
                                view! {
                                    <div id="lb">
                                        {maps
                                            .into_iter()
                                            .map(|map| {
                                                view! { <LeaderboardEntry map=map /> }
                                            })
                                            .collect_view()}
                                    </div>
                                },
                            )
                        }
                    })
            }}
        </Transition>
    }
}

#[component]
pub fn LeaderboardEntry(map: MapRuns) -> impl IntoView {
    let filter_key = Memo::new(|_| use_query_map().read().get("filter"));
    let sort_key = Memo::new(|_| use_query_map().read().get("sort"));
    let user = Memo::new(|_| use_params_map().read().get("id"));
    let runs = move || {
        let mut old_time = Decimal::new(999999, 3);
        let mut old_times = HashMap::<i64, Decimal>::new();
        let r = map.runs.clone();
        let mut runs: Vec<PartialRun> = r
            .into_iter()
            .filter(|r| {
                user().is_none() || r.user_id == user.get().unwrap().parse::<i64>().unwrap_or(-1)
            })
            .filter(|r| filter(r, filter_key.get(), &mut old_time, &mut old_times))
            .collect();
        runs.sort_unstable_by(sort(sort_key.get()));
        runs.into_iter()
            .enumerate()
            .collect::<Vec<(usize, PartialRun)>>()
    };
    let runs2 = runs.clone();
    let map_name = map.map.clone();
    let (top_run, set_top_run) = signal::<Option<PartialRun>>(None);
    let (sel_run, set_sel_run) = signal::<Option<PartialRun>>(None);
    let yt_id = Signal::derive(move || sel_run.get().map(|r| r.yt_id).flatten());
    let fallback_url = Signal::derive(move || sel_run.get().map(|r| r.proof));

    view! {
        <div class="lb_entry">
            <div class="header">
                <A href=map.id.to_string()>
                    <h2>{map_name}</h2>
                </A>
                {move || match runs().get(0) {
                    Some((_, r)) => {
                        set_top_run(Some(r.clone()));
                        set_sel_run(Some(r.clone()));
                        Either::Left(
                            view! {
                                <a href=format!("/user/{}", r.user_id)>
                                    <h5>{r.username.clone()}</h5>
                                </a>
                                <h5>{r.time.to_string()} " seconds"</h5>
                            },
                        )
                    }
                    None => Either::Right(view! {}),
                }}
            </div>
            <div class="content">
                <Player yt_id url=fallback_url cover=map.map />
                <div class="lb_entry_ranks">
                    <Show
                        when=move || top_run.with(|r| r.is_some())
                        fallback=|| view! { <span class="no-data">"No Runs Found"</span> }
                    >
                        <For
                            each=runs2.clone()
                            key=|r| r.1.id
                            children=move |(i, r)| {
                                let selected = move || sel_run().is_some_and(|s| s.id == r.id);
                                let run = r.clone();
                                view! {
                                    <div
                                        class="lb_entry_rank"
                                        on:click=move |_| {
                                            set_sel_run(Some(run.clone()));
                                        }
                                        class:selected=selected
                                    >
                                        <span class="rank">
                                            {move || match sort_key() {
                                                Some(k) => {
                                                    if k == "time" {
                                                        "#".to_string() + &(i + 1).to_string()
                                                    } else {
                                                        format!("{}", r.created_at.format("%d/%m/%y"))
                                                    }
                                                }
                                                None => "#".to_string() + &(i + 1).to_string(),
                                            }}
                                        </span>
                                        <span class="name">
                                            <A href=format!("/user/{}", r.user_id)>{r.username}</A>
                                        </span>
                                        <span class="time">{r.time.to_string()} " s"</span>
                                    </div>
                                }
                            }
                        />
                    </Show>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Player(
    yt_id: Signal<Option<String>>,
    url: Signal<Option<String>>,
    cover: String,
) -> impl IntoView {
    let (play, set_play) = signal(false);
    Effect::new(move |old: Option<Option<String>>| {
        if old.flatten() != *url.read() {
            set_play(false);
        }
        url.get()
    });
    view! {
        <div class="video">
            {move || {
                if *play.read() && yt_id.read().is_some() {
                    EitherOf3::A(
                        view! {
                            <iframe
                                src=format!(
                                    "https://www.youtube.com/embed/{}?autoplay=1&rel=0&modestbranding=1&showinfo=0",
                                    yt_id.get().unwrap(),
                                )
                                allowfullscreen
                            ></iframe>
                        },
                    )
                } else if let Some(url) = url.get() {
                    let url2 = url.clone();
                    if *play.read() {
                        Effect::new(move |_| { window().open_with_url_and_target(&url, "_blank") });
                    }
                    EitherOf3::B(
                        view! {
                            <div class="buttons">
                                <button class="play-wrapper" on:click=move |_| { set_play(true) }>
                                    <div></div>
                                </button>
                                <br />
                                <a class="external" href=url2 target="_blank">
                                    "Open in new Tab"
                                </a>
                            </div>
                            <div class="no-vid">
                                <img
                                    src=format!("/cdn/maps/{}.jpg", cover)
                                    alt=format!("Picture of {}", cover)
                                />
                            </div>
                        },
                    )
                } else {
                    EitherOf3::C(
                        view! {
                            <div class="no-vid">
                                <img
                                    src=format!("/cdn/maps/{}.jpg", cover)
                                    alt=format!("Picture of {}", cover)
                                />
                            </div>
                        },
                    )
                }
            }}
        </div>
    }
}

pub fn filter<'a>(
    r: &'a PartialRun,
    f: Option<String>,
    t: &'a mut Decimal,
    ts: &'a mut HashMap<i64, Decimal>,
) -> bool {
    match f {
        Some(f) => match f.as_str() {
            "verified" => r.verified,
            "was_pb" => {
                if &r.time < ts.get(&r.user_id).unwrap_or(&Decimal::new(999999, 3)) {
                    ts.insert(r.user_id, r.time);
                    true
                } else {
                    false
                }
            }
            "is_pb" => r.is_pb,
            "was_wr" => {
                if r.time < *t {
                    *t = r.time;
                    true
                } else {
                    false
                }
            }
            "is_wr" => r.is_wr,
            _ => true,
        },
        None => r.is_pb,
    }
}

pub fn sort(s: Option<String>) -> impl Fn(&PartialRun, &PartialRun) -> Ordering {
    match s {
        Some(s) => match s.as_str() {
            "date" => move |r1: &PartialRun, r2: &PartialRun| -> Ordering {
                if r1.created_at > r2.created_at {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            },
            _ => move |r1: &PartialRun, r2: &PartialRun| -> Ordering {
                if r1.time == r2.time {
                    if r1.created_at < r2.created_at {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                } else {
                    if r1.time < r2.time {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
            },
        },
        None => move |r1: &PartialRun, r2: &PartialRun| -> Ordering {
            if r1.time < r2.time {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        },
    }
}
