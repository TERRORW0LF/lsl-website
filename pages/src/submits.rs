use chrono::{Local, NaiveDateTime, TimeZone};
use components::{Collapsible, Filter, Select};
use leptos::{either::Either, prelude::*};
use leptos_router::{components::A, hooks::use_query_map};
use server::api::{get_maps, get_runs};
use types::api::RunFilters;

#[component]
pub fn Submits() -> impl IntoView {
    let params = use_query_map();
    let filters = Signal::derive(move || {
        params.with(|p| RunFilters {
            user: p.get("user").map(|v| v.parse::<i64>().ok()).flatten(),
            patch: p.get("patch").filter(|v| !v.is_empty()),
            layout: p.get("layout").filter(|v| !v.is_empty()),
            category: p.get("category").filter(|v| !v.is_empty()),
            map: p.get("map").filter(|v| !v.is_empty()),
            faster: p.get("faster").map(|s| s.parse().ok()).flatten(),
            slower: p.get("slower").map(|s| s.parse().ok()).flatten(),
            before: p
                .get("before")
                .map(|s| {
                    let st = s.chars().take(16).collect::<String>();
                    let ndt = NaiveDateTime::parse_from_str(&st, "%Y-%m-%dT%H:%M").ok()?;
                    Local.from_local_datetime(&ndt).latest()
                })
                .flatten(),
            after: p
                .get("after")
                .map(|s| {
                    let st = s.chars().take(16).collect::<String>();
                    let ndt = NaiveDateTime::parse_from_str(&st, "%Y-%m-%dT%H:%M").ok()?;
                    Local.from_local_datetime(&ndt).earliest()
                })
                .flatten(),
            sort: p.get("sort").filter(|v| !v.is_empty()).unwrap_or("date".into()),
            ascending: !p.get("order").filter(|v| !v.is_empty()).is_none_or(|s| s == "desc"),
        })
    });
    let offset = Signal::derive(move || {
        params
            .get()
            .get("page")
            .unwrap_or(String::from("0"))
            .parse::<i32>()
            .unwrap()
    });
    let runs = Resource::new(
        move || (filters.get(), offset.get()),
        move |f| async move { get_runs(f.0, f.1 * 50).await },
    );
    let last = Signal::derive(move || {
        let mut last = true;
        runs.map(|res| {
            let _ = res.as_ref().inspect(|v| last = v.len() < 50);
        });
        last
    });

    view! {
        <section id="filter-list" class="runs">
            <Collapsible id="filter" class="filter" header=|| "Show Filters">
                <Filter attr:class="filter">
                    <Select
                        name="sort"
                        indicator="Sort By"
                        options=[("date", "Date"), ("time", "Time"), ("section", "Section")]
                    />
                    <Select
                        name="order"
                        indicator="Order By"
                        selected=1
                        options=[("asc", "Ascending"), ("desc", "Descending")]
                    />
                    <div>
                        <label for="before" class="indicator">
                            "Before"
                        </label>
                        <input class="select" type="datetime-local" name="before" id="before" />
                    </div>
                    <div>
                        <label for="after" class="indicator">
                            "After"
                        </label>
                        <input class="select" type="datetime-local" name="after" id="after" />
                    </div>
                    <div>
                        <label for="faster" class="indicator">
                            "Faster Than"
                        </label>
                        <input
                            class="select"
                            type="number"
                            name="faster"
                            id="faster"
                            min="0"
                            step="0.001"
                        />
                    </div>
                    <div>
                        <label for="slower" class="indicator">
                            "Slower Than"
                        </label>
                        <input
                            class="select"
                            type="number"
                            name="slower"
                            id="slower"
                            min="0"
                            step="0.001"
                        />
                    </div>
                    <div>
                        <label for="user" class="indicator">
                            "User ID"
                        </label>
                        <input
                            class="select"
                            type="number"
                            name="user"
                            id="user"
                            min="1"
                            step="1"
                        />
                    </div>
                    <Select
                        name="patch"
                        indicator="Patch"
                        options=[
                            ("", "All"),
                            ("1.00", "1.00"),
                            ("1.41", "1.41"),
                            ("1.50", "1.50"),
                            ("2.00", "2.00"),
                            ("2.13", "Current"),
                        ]
                    />
                    <Select
                        name="layout"
                        indicator="Layout"
                        options=[
                            ("", "All"),
                            ("1", "Layout 1"),
                            ("2", "Layout 2"),
                            ("3", "Layout 3"),
                            ("4", "Layout 4"),
                            ("5", "Layout 5"),
                        ]
                    />
                    <Select
                        name="category"
                        indicator="Category"
                        options=[("", "All"), ("Standard", "Standard"), ("Gravspeed", "Gravspeed")]
                    />
                    <div>
                        <label for="map" class="indicator">
                            "Map"
                        </label>
                        <input class="select" list="maps" name="map" id="map" />
                        <datalist id="maps">
                            <Await future=get_maps() let:maps>
                                {match maps {
                                    Ok(v) => {
                                        Either::Left(
                                            v
                                                .into_iter()
                                                .map(|m| {
                                                    view! {
                                                        <option value=m.name.clone()>{m.name.clone()}</option>
                                                    }
                                                })
                                                .collect_view(),
                                        )
                                    }
                                    Err(_) => Either::Right(view! {}),
                                }}
                            </Await>
                        </datalist>
                    </div>
                </Filter>
            </Collapsible>
            <div class="grid">
                <span class="heading">"date"</span>
                <span class="heading">"user"</span>
                <span class="heading">"patch"</span>
                <span class="heading">"layout"</span>
                <span class="heading">"category"</span>
                <span class="heading">"map"</span>
                <span class="heading">"proof"</span>
                <span class="heading last">"time"</span>
                <div class="divider header"></div>
                <Suspense fallback=|| { "Fetching Runs" }>
                    <ErrorBoundary fallback=|_| {
                        view! { <div class="error-display">"You are not logged in"</div> }
                    }>
                        {move || {
                            runs.get()
                                .map(|res| {
                                    res.map(|runs| {
                                        runs.into_iter()
                                            .map(|r| {
                                                view! {
                                                    <span>
                                                        {format!("{}", r.created_at.format("%d/%m/%Y %H:%M"))}
                                                    </span>
                                                    <span>{r.username}</span>
                                                    <span>"Patch " {r.patch}</span>
                                                    <span>"Layout " {r.layout}</span>
                                                    <span>{r.category}</span>
                                                    <span>{r.map}</span>
                                                    <span>
                                                        <a href=r.proof>"link"</a>
                                                    </span>
                                                    <span class="last">{r.time.to_string()} " sec"</span>
                                                    <div class="divider"></div>
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                    })
                                })
                        }}
                    </ErrorBoundary>
                </Suspense>
            </div>
            <div class="pages row">
                <Show
                    when=move || offset.read() != 0
                    fallback=|| view! { <div class="arrow disabled">"<"</div> }
                >
                    <A
                        class:arrow=true
                        href=move || {
                            let mut map = params.get();
                            map.replace("page", (offset.get() - 1).to_string());
                            map.to_query_string()
                        }
                    >
                        "<"
                    </A>
                </Show>
                <div class="page">{move || offset.get() + 1}</div>
                <Suspense fallback=|| view! { <div class="arrow disabled">">"</div> }>
                    <Show
                        when=move || !*last.read()
                        fallback=|| view! { <div class="arrow disabled">">"</div> }
                    >
                        <A
                            class:arrow=true
                            href=move || {
                                let mut map = params.get();
                                map.replace("page", (offset.get() + 1).to_string());
                                map.to_query_string()
                            }
                        >
                            ">"
                        </A>
                    </Show>
                </Suspense>
            </div>
        </section>
    }
}
