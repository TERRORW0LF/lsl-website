use chrono::{Local, NaiveDateTime, TimeZone};
use components::Collapsible;
use leptos::prelude::*;
use leptos_router::{
    components::{A, Form},
    hooks::use_query_map,
};
use server::api::get_activity;
use types::api::ActivityFilters;

#[component]
pub fn Activity() -> impl IntoView {
    let params = use_query_map();
    let filters = Signal::derive(move || {
        params.with(|p| ActivityFilters {
            event: p.get("event").filter(|v| !v.is_empty()),
            user: p.get("user").map(|v| v.parse::<i64>().ok()).flatten(),
            patch: p.get("patch").filter(|v| !v.is_empty()),
            layout: p.get("layout").filter(|v| !v.is_empty()),
            category: p.get("category").filter(|v| !v.is_empty()),
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
    let activities = Resource::new(
        move || (filters.get(), offset.get()),
        move |f| async move { get_activity(f.0, f.1 * 50).await },
    );
    let last = Signal::derive(move || {
        let mut last = true;
        activities.map(|res| {
            let _ = res.as_ref().inspect(|v| {
                last = v.len() < 50;
            });
        });
        last
    });

    view! {
        <section id="filter-list" class="activity">
            <Collapsible id="filter" header=|| "Show Filters">
                <Form method="GET" action="">
                    <div class="row">
                        <div class="input-box">
                            <label for="sort" class="indicator">
                                "Sort By"
                            </label>
                            <select class="select" name="sort" id="sort">
                                <option value="date">"Date"</option>
                                <option value="section">"Section"</option>
                            </select>
                        </div>
                        <div class="input-box">
                            <label for="order" class="indicator">
                                "Order By"
                            </label>
                            <select class="select" name="order" id="order">
                                <option value="asc">"Ascending"</option>
                                <option selected value="desc">
                                    "Descending"
                                </option>
                            </select>
                        </div>
                        <div class="input-box">
                            <label for="before" class="indicator">
                                "Before"
                            </label>
                            <input class="select" type="datetime-local" name="before" id="before" />
                        </div>
                        <div class="input-box">
                            <label for="after" class="indicator">
                                "After"
                            </label>
                            <input class="select" type="datetime-local" name="after" id="after" />
                        </div>
                        <div class="input-box">
                            <label for="event" class="indicator">
                                "Event"
                            </label>
                            <select class="select" name="event" id="event">
                                <option value="">"All"</option>
                                <option value="join">"User Joined"</option>
                                <option value="rank">"Rank changed"</option>
                                <option value="title">"Title changed"</option>
                            </select>
                        </div>
                        <div class="input-box">
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
                        <div class="input-box">
                            <label for="patch" class="indicator">
                                "Patch"
                            </label>
                            <select class="select" name="patch" id="patch">
                                <option value="">"All"</option>
                                <option value="1.00">"1.00"</option>
                                <option value="1.41">"1.41"</option>
                                <option value="1.50">"1.50"</option>
                                <option value="2.00">"2.00"</option>
                                <option value="2.13">"Current"</option>
                            </select>
                        </div>
                        <div class="input-box">
                            <label for="layout" class="indicator">
                                "Layout"
                            </label>
                            <select class="select" name="layout" id="layout">
                                <option value="">"All"</option>
                                <option value="1">"Layout 1"</option>
                                <option value="2">"Layout 2"</option>
                                <option value="3">"Layout 3"</option>
                                <option value="4">"Layout 4"</option>
                                <option value="5">"Layout 5"</option>
                            </select>
                        </div>
                        <div class="input-box">
                            <label for="category" class="indicator">
                                "Category"
                            </label>
                            <select class="select" name="category" id="category">
                                <option value="">"All"</option>
                                <option value="Standard">"Standard"</option>
                                <option value="Gravspeed">"Gravspeed"</option>
                            </select>
                        </div>
                    </div>
                    <input type="submit" class="button" value="Apply" />
                </Form>
            </Collapsible>
            <div class="grid">
                <span class="heading">"date"</span>
                <span class="heading">"user"</span>
                <span class="heading">"patch"</span>
                <span class="heading">"layout"</span>
                <span class="heading">"category"</span>
                <span class="heading">"old title"</span>
                <span class="heading">"new title"</span>
                <span class="heading">"old rank"</span>
                <span class="heading">"new rank"</span>
                <div class="divider header"></div>
                <Suspense fallback=|| { "Fetching Runs" }>
                    <ErrorBoundary fallback=|_| {
                        view! { <div class="error-display">"You are not logged in"</div> }
                    }>
                        {move || {
                            activities
                                .get()
                                .map(|res| {
                                    res.map(|runs| {
                                        runs.into_iter()
                                            .map(|r| {
                                                view! {
                                                    <span>
                                                        {format!("{}", r.created_at.format("%d/%m/%Y %H:%M"))}
                                                    </span>
                                                    <span>{r.username}</span>
                                                    <span>
                                                        {r
                                                            .patch
                                                            .map(|v| format!("Patch {v}"))
                                                            .unwrap_or("-".into())}
                                                    </span>
                                                    <span>
                                                        {r
                                                            .layout
                                                            .map(|v| format!("Layout {v}"))
                                                            .unwrap_or("-".into())}
                                                    </span>
                                                    <span>{r.category.unwrap_or("-".into())}</span>
                                                    <span class=format!(
                                                        "{} color",
                                                        r.title_old.map(|v| v.to_string()).unwrap_or_default(),
                                                    )>
                                                        {r
                                                            .title_old
                                                            .clone()
                                                            .map(|v| v.to_string())
                                                            .unwrap_or("-".into())}
                                                    </span>
                                                    <span class=format!(
                                                        "{} color",
                                                        r.title_new.map(|v| v.to_string()).unwrap_or_default(),
                                                    )>
                                                        {r
                                                            .title_new
                                                            .clone()
                                                            .map(|v| v.to_string())
                                                            .unwrap_or("-".into())}
                                                    </span>
                                                    <span class=format!(
                                                        "rank-{} color",
                                                        r.rank_old.map(|v| v.to_string()).unwrap_or_default(),
                                                    )>
                                                        {r.rank_old.map(|v| format!("#{v}")).unwrap_or("-".into())}
                                                    </span>
                                                    <span class=format!(
                                                        "rank-{} color",
                                                        r.rank_new.map(|v| v.to_string()).unwrap_or_default(),
                                                    )>
                                                        {r.rank_new.map(|v| format!("#{v}")).unwrap_or("-".into())}
                                                    </span>
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
