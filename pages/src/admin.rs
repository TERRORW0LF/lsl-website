use chrono::{Local, NaiveDateTime, TimeZone};
use components::{Collapsible, Dialogue, Filter, Modal, Pager, Select, Table, TableLine};
use leptos::prelude::*;
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
    api::{ApiError, Permissions, Run, RunFilters, UserPermissions},
    leptos::UserResource,
};
use util::OptResToBool;

#[component]
pub fn Admin() -> impl IntoView {
    let user = expect_context::<UserResource>();
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
            <Show when=move || {
              user
                .and_then(|u| u.has(&Permissions::ManageSections) || u.has(&Permissions::ManageRuns))
                .opt_res_to_bool()
            }>
              <li>
                <A href="sections">"Sections"</A>
              </li>
            </Show>
            <Show when=move || { user.and_then(|u| u.has(&Permissions::ManageUsers)).opt_res_to_bool() }>
              <li>
                <A href="users">"Users"</A>
              </li>
            </Show>
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
    let user = expect_context::<UserResource>();
    let perm = Signal::derive(move || {
        user.and_then(|u| u.has(&Permissions::Verify) || u.has(&Permissions::ManageRuns)).opt_res_to_bool()
    });
    let params = use_query_map();
    let user_filter = Signal::derive(move || {
        let id = params.read().get("user").map(|v| v.parse::<i64>().ok()).flatten();
        let del_perm = user.and_then(|u| u.has(&Permissions::Delete)).opt_res_to_bool();
        let u_id = if del_perm {
            Some(user.read().as_ref().map(|r| r.as_ref().map(|u| u.id).ok()).flatten().unwrap_or(-1))
        } else {
            Some(-1)
        };
        if *perm.read() { id } else { u_id }
    });
    let filters = Signal::derive(move || {
        params.with(|p| RunFilters {
            user: user_filter.get(),
            patch: Some("2.13".into()),
            layout: p.get("layout").filter(|v| !v.is_empty()),
            category: p.get("category").filter(|v| !v.is_empty()),
            map: p.get("map").filter(|v| !v.is_empty()),
            status: p.get("status").filter(|v| !v.is_empty()),
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
            <Show when=move || *perm.read()>
              <div>
                <label for="user" class="indicator">
                  "User ID"
                </label>
                <input class="select" type="number" name="user" id="user" min="1" step="1" />
              </div>
            </Show>
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
                  <Await future=get_maps("1".into(), "Standard".into()) let:maps>
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
            <Select
              name="status"
              indicator="Status"
              options=[("", "All"), ("verified", "Verified"), ("pb", "Personal Best"), ("wr", "World Record")]
            />
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
                    <Table
                      attr:class="button-2"
                      headers=vec![
                        "id".into(),
                        "date".into(),
                        "layout".into(),
                        "category".into(),
                        "map".into(),
                        "proof".into(),
                        "time".into(),
                        "".into(),
                      ]
                    >
                      {runs
                        .into_iter()
                        .map(|r| {
                          let id = r.id;
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
                                  format!("edit{}", params.to_query_string())
                                }
                              >
                                <div></div>
                              </A>
                              <A
                                attr:class="delete"
                                href=move || format!("{}{}", id.to_string(), params.get().to_query_string())
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
pub fn Edit(#[prop(into)] redirect: Signal<String>) -> impl IntoView {
    let run = Memo::new(|_| {
        use_params_map()
            .get()
            .get("run")
            .map(|r| serde_qs::from_str::<Run>(&r).ok())
            .flatten()
            .ok_or(ApiError::InvalidInput)
    });
    let maps = Resource::new(
        move || (run.get().map(|r| r.section.category), run.get().map(|r| r.section.category)),
        move |_| async move {
            if let Ok(run) = run.get() {
                get_maps(run.section.layout, run.section.category).await
            } else {
                Err(ApiError::NotFound)
            }
        },
    );
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
        <Dialogue action=update redirect>
          <Select
            options=vec![("true", "Yes").into(), ("false", "No").into()]
            indicator="Verified"
            name="verified"
            selected=!run.read().as_ref().map(|r| r.verified).unwrap_or(false) as usize
          />
          <div>
            <label for="map" class="indicator">
              "Map"
            </label>
            <input class="select" list="maps" name="section_id" id="map" />
            <datalist id="maps">
              <Suspense>
                {maps
                  .and_then(|v| {
                    v.into_iter()
                      .map(|m| {
                        view! { <option value=m.id>{m.map.clone()}</option> }
                      })
                      .collect_view()
                  })}
              </Suspense>
            </datalist>
          </div>
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
