use chrono::{Local, NaiveDateTime, TimeZone};
use components::{Collapsible, Filter, Select};
use leptos::{either::Either, prelude::*};
use leptos_router::{
    components::{A, Outlet},
    hooks::{use_params_map, use_query_map},
};
use server::{
    api::{get_maps, get_runs, get_user},
    auth::Delete,
};
use types::{
    api::{ApiError, RunFilters},
    leptos::UserResource,
};

#[component]
pub fn Profile(id: Signal<i64>) -> impl IntoView {
    let user = Resource::new(id, |s| get_user(s));
    view! {
        <section id="profile">
            <Suspense fallback=|| {
                "Fetching User"
            }>
                {move || {
                    user.get()
                        .map(|res| {
                            res.map(|u| {
                                view! {
                                    <h1>{u.username}</h1>
                                    <div class="row">
                                        <div>
                                            <img
                                                class="avatar"
                                                src=format!("/cdn/users/{}.jpg", u.pfp)
                                            />
                                            <p>
                                                {u.bio.unwrap_or("This user has no about me.".into())}
                                            </p>
                                        </div>
                                        <div class="medals"></div>
                                    </div>
                                    <div class="rankings"></div>
                                }
                            })
                        })
                }}
            </Suspense>
        </section>
    }
}

#[component]
pub fn ManageRuns() -> impl IntoView {
    let params = use_query_map();
    let filters = Signal::derive(move || {
        params.with(|p| RunFilters {
            user: None,
            patch: Some("2.13".into()),
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
    let delete = ServerAction::<Delete>::new();
    provide_context(delete);
    let user = expect_context::<UserResource>();
    let runs = Resource::new(
        move || (filters.get(), delete.version().get(), offset.get()),
        move |mut f| async move {
            let user = user.await?;
            f.0.user = Some(user.id);
            get_runs(f.0, f.2 * 50).await
        },
    );
    let last = Signal::derive(move || {
        let mut last = true;
        runs.map(|res| {
            let _ = res.as_ref().inspect(|v| last = v.len() < 50);
        });
        last
    });

    view! {
        <section id="filter-list" class="manage">
            <Outlet />
            <Collapsible id="filter" header=|| "Show Filters">
                <Filter>
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
                <span class="heading">"id"</span>
                <span class="heading">"date"</span>
                <span class="heading">"layout"</span>
                <span class="heading">"category"</span>
                <span class="heading">"map"</span>
                <span class="heading">"proof"</span>
                <span class="heading">"time"</span>
                <span></span>
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
                                                    <span>{r.id}</span>
                                                    <span>
                                                        {format!("{}", r.created_at.format("%d/%m/%Y %H:%M"))}
                                                    </span>
                                                    <span>"Layout " {r.layout}</span>
                                                    <span>{r.category}</span>
                                                    <span>{r.map}</span>
                                                    <span>
                                                        <a href=r.proof>"link"</a>
                                                    </span>
                                                    <span>{r.time.to_string()} " sec"</span>
                                                    <A
                                                        attr:class="delete"
                                                        href=move || {
                                                            format!(
                                                                "{}{}",
                                                                r.id.to_string(),
                                                                params.get().to_query_string(),
                                                            )
                                                        }
                                                    >
                                                        <div></div>
                                                    </A>
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

#[component]
pub fn Delete() -> impl IntoView {
    let id = Signal::derive(|| use_params_map().get().get("id").unwrap().parse::<i32>().unwrap_or(-1));
    let action = expect_context::<ServerAction<Delete>>();
    let result = Signal::derive(move || action.value().get());
    view! {
        <A attr:class="toner" href="../">
            <div />
        </A>
        <section id="box">
            <h1>"Delete Run"</h1>
            <p>"Please confirm your deletion of the run with id " {id}</p>
            <ErrorBoundary fallback=|e| {
                view! {
                    <span class="error">
                        {move || {
                            let e = e.get().into_iter().next().unwrap().1;
                            if e.is::<ApiError>() {
                                let e = e.downcast_ref::<ApiError>().unwrap();
                                match e {
                                    ApiError::Unauthenticated => "🛈 Please log in",
                                    ApiError::Unauthorized => "🛈 Missing permission",
                                    ApiError::NotFound => "🛈 Invalid run",
                                    _ => "🛈 Something went wrong. Try again",
                                }
                            } else {
                                "🛈 Something went wrong. Try again"
                            }
                        }}
                    </span>
                }
            }>
                <div class="hidden">{result}</div>
            </ErrorBoundary>
            <ActionForm action>
                <input
                    type="text"
                    name="redirect"
                    hidden
                    value=|| format!("user/@me/manage{}", use_query_map().get().to_query_string())
                />
                <input type="text" name="id" hidden value=id />
                <div class="row">
                    <A
                        attr:class="button secondary"
                        href=|| format!("../{}", use_query_map().get().to_query_string())
                    >
                        "Cancel"
                    </A>
                    <input type="submit" class="button danger" value="Delete" />
                </div>
            </ActionForm>
        </section>
    }
}
