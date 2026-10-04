use chrono::{Local, NaiveDateTime, TimeZone};
use components::{Collapsible, Dialogue, Filter, Modal, Pager, Select, Table, TableLine};
use leptos::{prelude::*, server_fn::serde::Serialize};
use leptos_router::{
    components::{A, Outlet},
    hooks::{use_params_map, use_query_map},
};
use server::{
    admin::UpdateRun,
    api::{get_maps, get_runs},
    auth::Delete,
};
use types::{
    api::{ApiError, Run, RunFilters},
    leptos::UserResource,
};

#[component]
pub fn Admin() -> impl IntoView {
    view! {
      <div class="sidebar-container">
        <div class="hamburger" class="sidebar-toggle">
          <input type="checkbox" />
          <span></span>
          <span></span>
          <span></span>
        </div>
        <nav class="sidebar-nav">
          <ul>
            <li>
              <A href="runs">"Submits"</A>
            </li>
            <li>
              <A href="sections">"Sections"</A>
            </li>
            <li>
              <A href="users">"Users"</A>
            </li>
          </ul>
        </nav>
        <main>
          <Outlet />
        </main>
      </div>
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
    let offset = Signal::derive(move || params.get().get("page").unwrap_or(String::from("0")).parse::<i32>().unwrap());
    let delete = ServerAction::<Delete>::new();
    let update = ServerAction::<UpdateRun>::new();
    provide_context(delete);
    provide_context(update);
    let user = expect_context::<UserResource>();
    let runs = Resource::new(
        move || (filters.get(), update.version().get(), delete.version().get(), offset.get()),
        move |mut f| async move {
            let user = user.await?;
            f.0.user = Some(user.id);
            get_runs(f.0, f.3 * 50).await
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
              <input class="select" type="number" name="faster" id="faster" min="0" step="0.001" />
            </div>
            <div>
              <label for="slower" class="indicator">
                "Slower Than"
              </label>
              <input class="select" type="number" name="slower" id="slower" min="0" step="0.001" />
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
                <ErrorBoundary fallback=|_| ()>
                  <Await future=get_maps() let:maps>
                    {maps
                      .clone()
                      .map(|v| {
                        v.into_iter()
                          .map(|m| {
                            view! { <option value=m.map.clone()>{m.map.clone()}</option> }
                          })
                          .collect_view()
                      })}
                  </Await>
                </ErrorBoundary>
              </datalist>
            </div>
          </Filter>
        </Collapsible>
        <Suspense fallback=|| { "Fetching Runs" }>
          <ErrorBoundary fallback=|_| {
            view! { <div class="error-display">"You are not logged in"</div> }
          }>
            {move || {
              runs
                .and_then(|runs| {
                  let runs = runs.clone();
                  view! {
                    <Table headers=vec![
                      "id".into(),
                      "date".into(),
                      "layout".into(),
                      "category".into(),
                      "map".into(),
                      "proof".into(),
                      "time".into(),
                      "".into(),
                    ]>
                      {runs
                        .into_iter()
                        .map(|r| {
                          view! {
                            <TableLine>
                              {r.id} {format!("{}", r.created_at.format("%d/%m/%Y %H:%M"))}
                              {format!("Layout {}", r.section.layout)} {r.section.category.clone()}
                              {r.section.map.clone()} <a href=r.proof.clone()>"link"</a>
                              {format!("{} sec", r.time.to_string())}
                              <A
                                attr:class="edit"
                                href=move || {
                                  let mut params = params.get();
                                  params.replace("run", serde_qs::to_string(&r).unwrap_or("".into()));
                                  format!("{}{}", r.id.to_string(), params.to_query_string())
                                }
                              >
                                <div></div>
                              </A>
                            </TableLine>
                          }
                        })
                        .collect::<Vec<_>>()}
                    </Table>
                  }
                })
            }}
          </ErrorBoundary>
        </Suspense>
        <Pager name="page" last />
      </section>
    }
}

#[component]
pub fn Edit() -> impl IntoView {
    let run = Memo::new(|_| {
        use_params_map()
            .get()
            .get("run")
            .map(|r| serde_qs::from_str::<Run>(&r).ok())
            .flatten()
            .ok_or(ApiError::InvalidInput)
    });
    let delete = expect_context::<ServerAction<Delete>>();
    let update = expect_context::<ServerAction<UpdateRun>>();
    let result = Signal::derive(move || delete.value().get());
    view! {
      <Modal>
        <h1>"Edit Run"</h1>
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
        <Dialogue action=update>
          <Select
            name="layout"
            indicator="Layout"
            options=[("1", "Layout 1"), ("2", "Layout 2"), ("3", "Layout 3"), ("4", "Layout 4"), ("5", "Layout 5")]
          />
          <Select name="category" indicator="Category" options=[("Standard", "Standard"), ("Gravspeed", "Gravspeed")] />
          <div>
            <label for="map" class="indicator">
              "Map"
            </label>
            <input class="select" list="maps" name="map" id="map" />
            <datalist id="maps">
              <ErrorBoundary fallback=|_| ()>
                <Await future=get_maps() let:maps>
                  {
                    let maps = maps.clone();
                    maps
                      .map(|v| {
                        v.into_iter()
                          .map(|m| {
                            view! { <option value=m.map.clone()>{m.map.clone()}</option> }
                          })
                          .collect_view()
                      })
                  }
                </Await>
              </ErrorBoundary>
            </datalist>
          </div>
          <input
            type="text"
            name="redirect"
            hidden
            value=|| format!("user/@me/manage{}", use_query_map().get().to_query_string())
          />
        </Dialogue>
      </Modal>
    }
}

#[component]
pub fn ManageSections() -> impl IntoView {
    view! { <h1>"Placeholder"</h1> }
}

#[component]
pub fn ManageUsers() -> impl IntoView {
    view! { <h1>"Placeholder"</h1> }
}
