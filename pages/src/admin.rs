use chrono::{Local, NaiveDateTime, TimeZone};
use components::{Collapsible, Filter, Pager, Select, Table, TableLine};
use leptos::{either::Either, prelude::*};
use leptos_router::{
    components::{A, Outlet},
    hooks::use_query_map,
};
use server::api::{get_maps, get_runs};
use types::api::RunFilters;

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
    let offset = Signal::derive(move || params.get().get("page").unwrap_or(String::from("0")).parse::<i32>().unwrap());
    let runs =
        Resource::new(move || (filters.get(), offset.get()), move |f| async move { get_runs(f.0, f.1 * 50).await });
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
              <input class="select" type="number" name="faster" id="faster" min="0" step="0.001" />
            </div>
            <div>
              <label for="slower" class="indicator">
                "Slower Than"
              </label>
              <input class="select" type="number" name="slower" id="slower" min="0" step="0.001" />
            </div>
            <div>
              <label for="user" class="indicator">
                "User ID"
              </label>
              <input class="select" type="number" name="user" id="user" min="1" step="1" />
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
                            view! { <option value=m.map.clone()>{m.map.clone()}</option> }
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
        <Suspense fallback=|| { "Fetching Runs" }>
          <ErrorBoundary fallback=|_| {
            view! { <div class="error-display">"Failed to load submits"</div> }
          }>
            {move || {
              runs
                .and_then(|runs| {
                  let runs = runs.clone();
                  view! {
                    <Table headers=vec![
                      "date".into(),
                      "user".into(),
                      "patch".into(),
                      "layout".into(),
                      "category".into(),
                      "map".into(),
                      "proof".into(),
                      "time".into(),
                    ]>
                      {runs
                        .into_iter()
                        .map(|r| {
                          view! {
                            <TableLine>
                              {format!("{}", r.created_at.format("%d/%m/%Y %H:%M"))} {r.user.username}
                              {format!("Patch {}", r.section.patch)} {format!("Layout {}", r.section.layout)}
                              {r.section.category} {r.section.map} <a href=r.proof>"link"</a> {r.time.to_string()}
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
pub fn ManageSections() -> impl IntoView {
    view! { <h1>"Placeholder"</h1> }
}

#[component]
pub fn ManageUsers() -> impl IntoView {
    view! { <h1>"Placeholder"</h1> }
}
