use std::collections::HashMap;

use charming::{
    component::{Axis, DataZoom, FilterMode, Grid, Legend},
    datatype::CompositeValue,
    element::{AxisLabel, AxisType, JsFunction, Step, Tooltip, Trigger},
    series::Line,
};
use chrono::Local;
use components::{Chart, Collapsible, Player};
use leptos::{either::Either, prelude::*};
use leptos_router::{
    components::A,
    hooks::{use_params_map, use_query_map},
};
use rust_decimal::{
    Decimal,
    prelude::{FromPrimitive, ToPrimitive},
};

use crate::leaderboard::{filter, sort};
use server::api::get_runs_id;
use types::{api::PartialRun, internal::Proof};

#[component]
pub fn Map(id: Signal<i32>) -> impl IntoView {
    let map = Resource::new(id, |s| get_runs_id(s));

    view! {
        <section id="map">
            <Transition fallback=move || {
                view! { <p>"Loading..."</p> }
            }>
                <ErrorBoundary fallback=|_| {
                    view! { <span class="error">"🛈 Something went wrong. Try again"</span> }
                }>
                    {move || {
                        map.get()
                            .map(|data| {
                                data.map(|runs| {
                                    view! {
                                        <div>
                                            <h1>{runs.map.clone()}</h1>
                                            <PbChart runs=runs.runs.clone() />
                                            <MapRunList map=runs.map runs=runs.runs />
                                        </div>
                                    }
                                })
                            })
                    }}
                </ErrorBoundary>
            </Transition>
        </section>
    }
}

#[component]
fn PbChart(mut runs: Vec<PartialRun>) -> impl IntoView {
    let user = Memo::new(|_| use_params_map().read().get("id"));
    let mut old_times = HashMap::<i64, Decimal>::new();
    runs.sort_by_key(|r| r.created_at);
    runs = runs
        .into_iter()
        .filter(|r| user.get().is_none() || r.user_id == user.get().unwrap().parse::<i64>().unwrap_or(-1))
        .filter(|r| {
            if &r.time < old_times.get(&r.user_id).unwrap_or(&Decimal::new(999999, 3)) {
                old_times.insert(r.user_id, r.time);
                true
            } else {
                false
            }
        })
        .collect();
    let mut users = HashMap::<String, Vec<CompositeValue>>::new();
    let mut min = Decimal::from_i32(90).unwrap();
    let mut max = Decimal::ZERO;
    for run in runs {
        if run.time < min {
            min = run.time;
        }
        if run.time > max && run.time <= Decimal::from_i32(90).unwrap() {
            max = run.time;
        }
        if let Some(user) = users.get_mut(&run.name) {
            user.push(CompositeValue::Array(vec![
                CompositeValue::String(run.created_at.to_rfc3339()),
                CompositeValue::Number(charming::datatype::NumericValue::Float(run.time.to_f64().unwrap())),
            ]));
        } else {
            users.insert(
                run.name.clone(),
                vec![CompositeValue::Array(vec![
                    CompositeValue::String(run.created_at.to_rfc3339()),
                    CompositeValue::Number(charming::datatype::NumericValue::Float(run.time.to_f64().unwrap())),
                ])],
            );
        }
    }
    for user in &mut users {
        if let Some(CompositeValue::Array(vec)) = user.1.last() {
            user.1.push(CompositeValue::Array(vec![
                CompositeValue::String(Local::now().to_rfc3339()),
                vec[1].clone(),
            ]));
        }
    }
    let mut chart =
        charming::Chart::new()
            .legend(Legend::new())
            .grid(Grid::new().contain_label(true).left(25).right(50))
            .tooltip(Tooltip::new().trigger(Trigger::Item).formatter(
                JsFunction::new_with_args(
                    "params",
                    r#"
                        const date = new Date(params.data[0]);
                        const day = `${date.getDate()}`.padStart(2, '0');
                        const hour = `${date.getHours()}`.padStart(2, '0');
                        const min = `${date.getMinutes()}`.padStart(2, '0');
                        const month = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
                        return `<div class="header" style="color: ${params.color};">${params.seriesName}</div>
                        ${params.marker} ${params.data[1].toFixed(3)} sec
                        <div class="date">${day} ${month[date.getMonth()]} ${date.getFullYear()} ${hour}:${min}</div>`;
                    "#,
                ),
            ))
            .data_zoom(
                DataZoom::new()
                    .show(true)
                    .realtime(true)
                    .start(0)
                    .end(100)
                    .filter_mode(FilterMode::None),
            )
            .data_zoom(
                DataZoom::new()
                    .show(true)
                    .realtime(true)
                    .start(0)
                    .end(100)
                    .y_axis_index(0),
            )
            .x_axis(Axis::new().type_(AxisType::Time))
            .y_axis(
                Axis::new()
                    .type_(AxisType::Value)
                    .min(
                        min.round_dp_with_strategy(
                            1,
                            rust_decimal::RoundingStrategy::ToNegativeInfinity,
                        )
                        .to_f64()
                        .unwrap(),
                    )
                    .max(
                        max.round_dp_with_strategy(
                            1,
                            rust_decimal::RoundingStrategy::ToPositiveInfinity,
                        )
                        .to_f64()
                        .unwrap(),
                    )
                    .axis_label(AxisLabel::new().formatter(JsFunction::new_with_args(
                        "value",
                        "return `${value} sec`",
                    ))),
            );
    for user in users {
        chart = chart.series(Line::new().name(user.0).data(user.1).step(Step::End));
    }
    view! { <Chart chart /> }
}

#[component]
fn MapRunList(map: String, runs: Vec<PartialRun>) -> impl IntoView {
    let filter_key = Memo::new(|_| use_query_map().read().get("filter"));
    let sort_key = Memo::new(|_| use_query_map().read().get("sort"));
    let user = Memo::new(|_| use_params_map().read().get("id"));
    let runs_disp = Signal::derive(move || {
        let mut old_time = Decimal::new(999999, 3);
        let mut old_times = HashMap::<i64, Decimal>::new();
        let r = runs.clone();
        let mut runs: Vec<PartialRun> = r
            .into_iter()
            .filter(|r| user.get().is_none() || r.user_id == user.get().unwrap().parse::<i64>().unwrap_or(-1))
            .filter(|r| filter(r, filter_key.get(), &mut old_time, &mut old_times))
            .collect();
        runs.sort_unstable_by(sort(sort_key.get()));
        runs.into_iter().enumerate().collect::<Vec<(usize, PartialRun)>>()
    });

    view! {
        {if runs_disp.read().is_empty() {
            Either::Left(view! { <h2>"No Runs Found"</h2> })
        } else {
            Either::Right(
                view! {
                    <For
                        each=runs_disp
                        key=|r| r.1.id
                        children=move |(i, r)| {
                            let username = r.name.clone();
                            let map2 = map.clone();
                            view! {
                                <div class="map-entry">
                                    <Collapsible
                                        id=format!("run_{}", r.id)
                                        header=move || {
                                            view! {
                                                <div class="row">
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
                                                        )>{username}</A>
                                                    </span>
                                                    <span class="time">{r.time.to_string()} " s"</span>
                                                </div>
                                            }
                                        }
                                    >
                                        <div class="row">
                                            <Player
                                                proof=Proof {
                                                    yt_id: r.yt_id,
                                                    url: r.proof.clone(),
                                                }
                                                    .into()
                                                cover=map2
                                            />
                                            <div class="run-data">
                                                <div class="grid">
                                                    <div class="entry">
                                                        <h3>"RANK"</h3>
                                                        <p>"#"{i + 1}</p>
                                                    </div>
                                                    <div class="entry">
                                                        <h3>"DATE"</h3>
                                                        <p>
                                                            {r.created_at.format("%a %d %b %Y %k:%M:%S").to_string()}
                                                        </p>
                                                    </div>
                                                    <div class="entry">
                                                        <h3>"USER"</h3>
                                                        <p>
                                                            <A href=format!(
                                                                "/user/{}/leaderboard",
                                                                r.user_id,
                                                            )>{r.name}</A>
                                                        </p>
                                                    </div>
                                                    <div class="entry">
                                                        <h3>"TIME"</h3>
                                                        <p>{r.time.to_string()} " sec"</p>
                                                    </div>
                                                    <div class="entry">
                                                        <h3>"STATUS"</h3>
                                                        <p>
                                                            {if r.is_wr {
                                                                "World Record"
                                                            } else if r.is_pb {
                                                                "Personal Best"
                                                            } else if r.verified {
                                                                "Verified"
                                                            } else {
                                                                "Unverified"
                                                            }}
                                                        </p>
                                                    </div>
                                                    <div class="entry">
                                                        <h3>"PROOF"</h3>
                                                        <p>
                                                            <a class="extern".to_string() href=r.proof target="_blank">
                                                                "link"
                                                            </a>
                                                        </p>
                                                    </div>
                                                </div>
                                                <div class="id">{r.id}</div>
                                            </div>
                                        </div>
                                    </Collapsible>
                                </div>
                            }
                        }
                    />
                },
            )
        }}
    }
}
