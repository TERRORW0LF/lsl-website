use components::{Accordion, Header, Player};
use leptos::{either::*, prelude::*};
use leptos_meta::Title;
use leptos_router::{
    components::{A, Form},
    hooks::{use_params_map, use_query_map},
};
use rust_decimal::Decimal;
use std::{cmp::Ordering, collections::HashMap};

use server::api::get_runs_category;
use types::{
    api::{PartialRun, SectionRuns},
    internal::Proof,
};

#[component]
pub fn Section(
    #[prop(into)] layouts: Signal<Vec<(String, String)>>,
    #[prop(into)] categories: Signal<Vec<(String, String)>>,
    #[prop(into)] category: Signal<String>,
) -> impl IntoView {
    let query = use_query_map();

    view! {
        <Title text="Leaderboard" />
        <Header right=move || {
            categories
                .get()
                .iter()
                .map(|c| {
                    view! {
                        <A
                            href=move || format!("../{}{}", c.0, query.get().to_query_string())
                            scroll=false
                        >
                            <span class="text">{c.1.clone()}</span>
                        </A>
                    }
                })
        }>
            {move || {
                layouts
                    .get()
                    .iter()
                    .map(|l| {
                        view! {
                            <A
                                href=move || {
                                    format!(
                                        "../../{}/{}{}",
                                        l.0,
                                        category.get(),
                                        query.get().to_query_string(),
                                    )
                                }
                                scroll=false
                            >
                                <span class="text">{l.1.clone()}</span>
                            </A>
                        }
                    })
                    .collect_view()
            }}
        </Header>
        <Accordion id="filter" header=|| "Filters">
            <Form method="GET" action="">
                <div class="group">
                    <h6>"Patch"</h6>
                    <div class="options">
                        <A href="../../../1.00/1/standard">"1.00"</A>
                        <A href="../../../1.41/1/standard">"1.41"</A>
                        <A href="../../../1.50/1/standard">"1.50"</A>
                        <A href="../../../2.00/1/standard">"2.00"</A>
                        <A href="../../../2.13/1/standard">"Current"</A>
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
        </Accordion>
    }
}

#[component]
pub fn Leaderboard(patch: Signal<String>, layout: Signal<String>, category: Signal<String>) -> impl IntoView {
    let selection = Signal::derive(move || (patch.get(), layout.get(), category.get()));
    let maps = Resource::new(selection, |mut s| {
        get_runs_category(s.0, s.1, format!("{}{}", s.2.remove(0).to_uppercase(), s.2))
    });

    view! {
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || {
                maps.map(|data| match data {
                    Err(e) => Either::Right(view! { <p>{e.to_string()}</p> }),
                    Ok(maps) => {
                        Either::Left(
                            view! {
                                <div id="lb">
                                    {maps
                                        .into_iter()
                                        .map(|map| {
                                            view! { <LeaderboardEntry map=map.clone() /> }
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
pub fn LeaderboardEntry(map: SectionRuns) -> impl IntoView {
    let filter_key = Memo::new(|_| use_query_map().read().get("filter"));
    let sort_key = Memo::new(|_| use_query_map().read().get("sort"));
    let user = Memo::new(|_| use_params_map().read().get("id"));
    let runs = Signal::derive(move || {
        let mut old_time = Decimal::new(999999, 3);
        let mut old_times = HashMap::<i64, Decimal>::new();
        let r = map.runs.clone();
        let mut runs: Vec<PartialRun> = r
            .into_iter()
            .filter(|r| user().is_none() || r.user_id == user.get().unwrap().parse::<i64>().unwrap_or(-1))
            .filter(|r| filter(r, filter_key.get(), &mut old_time, &mut old_times))
            .collect();
        runs.sort_unstable_by(sort(sort_key.get()));
        runs.into_iter().enumerate().collect::<Vec<(usize, PartialRun)>>()
    });
    let map_name = map.map.clone();
    let (top_run, set_top_run) = signal::<Option<PartialRun>>(None);
    let (sel_run, set_sel_run) = signal::<Option<PartialRun>>(None);
    let proof = Signal::derive(move || {
        sel_run.get().map(|r| Proof {
            yt_id: r.yt_id,
            url: r.proof,
        })
    });

    view! {
        <div class="lb_entry">
            <div class="header">
                <A href=format!("../../../map/{}", map.id.to_string())>
                    <h2>{map_name}</h2>
                </A>
                {move || match runs().get(0) {
                    Some((_, r)) => {
                        set_top_run(Some(r.clone()));
                        set_sel_run(Some(r.clone()));
                        Either::Left(
                            view! {
                                <a href=format!("/user/{}/leaderboard", r.user_id)>
                                    <h5>{r.name.clone()}</h5>
                                </a>
                                <h5>{r.time.to_string()} " seconds"</h5>
                            },
                        )
                    }
                    None => Either::Right(view! {}),
                }}
            </div>
            <div class="content">
                <Player proof cover=map.map />
                <div class="lb_entry_ranks">
                    <Show
                        when=move || top_run.with(|r| r.is_some())
                        fallback=|| view! { <span class="no-data">"No Runs Found"</span> }
                    >
                        <For
                            each=runs
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
                                            <A href=format!(
                                                "/user/{}/leaderboard",
                                                r.user_id,
                                            )>{r.name}</A>
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

pub fn filter<'a>(r: &'a PartialRun, f: Option<String>, t: &'a mut Decimal, ts: &'a mut HashMap<i64, Decimal>) -> bool {
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
