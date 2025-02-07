use crate::{
    app::UserResource,
    server::{
        api::{get_maps, get_runs_user, get_user, ApiError, RunFilters},
        auth::Delete,
    },
};
use chrono::{Local, NaiveDateTime, TimeZone};
use leptos::{either::Either, prelude::*};
use leptos_router::{
    components::{Form, Outlet, A},
    hooks::{use_params_map, use_query_map},
};

#[component]
pub fn Profile() -> impl IntoView {
    let user_id = Signal::derive(|| use_params_map().get().get("id"));
    let user = Resource::new(
        move || user_id.get().unwrap().parse::<i64>().unwrap_or(0),
        |id: i64| get_user(id),
    );
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
            sort: p.get("sort").unwrap_or("date".into()),
            ascending: !p.get("order").is_none_or(|s| s == "desc"),
            layout: p.get("layout"),
            category: p.get("category"),
            map: p.get("map"),
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
        move |f| async move {
            let user = user.await?;
            get_runs_user(user.id, f.0, f.2 * 50).await
        },
    );

    view! {
        <section id="manage-runs">
            <Outlet />
            <details>
                <summary>
                    <span role="term" aria-details="filters" class="icon">
                        "Show Filters"
                    </span>
                </summary>
            </details>
            <div role="definition" id="filters" class="content">
                <Form method="GET" action="">
                    <div class="row">
                        <div class="input-box">
                            <label for="sort" class="indicator">
                                "Sort By"
                            </label>
                            <select class="select" name="sort" id="sort">
                                <option value="date">"Date"</option>
                                <option value="time">"Time"</option>
                                <option value="section">"Section"</option>
                            </select>
                        </div>
                        <div class="input-box">
                            <label for="order" class="indicator">
                                "Order By"
                            </label>
                            <select class="select" name="order" id="order">
                                <option value="asc">"Ascending"</option>
                                <option value="desc">"Descending"</option>
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
                        <div class="input-box">
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
                        <div class="input-box">
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
                    </div>
                    <input type="submit" class="button" value="Apply" />
                </Form>
            </div>
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
                {move || {
                    if offset.read() == 0 {
                        Either::Left(view! { <div class="arrow disabled">"<"</div> })
                    } else {
                        Either::Right(
                            view! {
                                <A
                                    class:arrow=true
                                    href=format!(
                                        "{}$page={}",
                                        params.get().to_query_string(),
                                        offset.get() - 1,
                                    )
                                >
                                    "<"
                                </A>
                            },
                        )
                    }
                }} <div class="page">{move || offset.get() + 1}</div>
                <A
                    href=move || {
                        format!("{}&page={}", params.get().to_query_string(), offset.get() + 1)
                    }
                    class:arrow=true
                    class:disabled=move || {
                        runs
                            .get()
                            .map(|res| res.map(|r| r.len()).unwrap_or_default())
                            .unwrap_or_default() < 50
                    }
                >
                    ">"
                </A>
            </div>
        </section>
    }
}

#[component]
pub fn Delete() -> impl IntoView {
    let id = Signal::derive(|| {
        use_params_map()
            .get()
            .get("id")
            .unwrap()
            .parse::<i32>()
            .unwrap_or(-1)
    });
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
                            if e.is::<ServerFnError<ApiError>>() {
                                let e = e.downcast_ref::<ServerFnError<ApiError>>().unwrap();
                                match e {
                                    ServerFnError::WrappedServerError(err) => {
                                        match err {
                                            ApiError::Unauthenticated => "🛈 Please log in",
                                            ApiError::Unauthorized => "🛈 Missing permission",
                                            ApiError::NotFound => "🛈 Invalid run",
                                            _ => "🛈 Something went wrong. Try again",
                                        }
                                    }
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
