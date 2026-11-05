use chrono::{Local, NaiveDateTime, TimeZone};
use components::{
    Button, ButtonType, Checkbox, Collapsible, Dialogue, Filter, Freeform, Modal, Pager, Redirect, Select, Table,
    TableLine,
};
use leptos::prelude::*;
use leptos_router::{
    components::{A, Outlet},
    hooks::{use_params_map, use_query_map},
};
use server::{
    admin::{
        AddSection, DeleteSection, RecalculateRankings, RecalculateRuns, UpdateRun, UpdateSection, UpdateUser,
        get_users,
    },
    api::{get_maps, get_runs, get_sections},
    auth::Delete,
};
use types::{
    api::{ApiError, Permissions, Run, RunFilters, Section, SectionFilters, UserFilters, UserPermissions},
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
        user.and_then(|u| u.has_any(&[Permissions::Verify, Permissions::ManageRuns])).opt_res_to_bool()
    });
    let user_id = Signal::derive(move || user.map(|r| r.as_ref().map(|u| u.id).ok()).flatten().unwrap_or(-1));
    let del_perm = Signal::derive(move || user.and_then(|u| u.has(&Permissions::ManageRuns)).opt_res_to_bool());
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
        move |f| get_runs(f.0, f.3 * 50),
    );
    let last = Signal::derive(move || {
        let mut last = true;
        runs.map(|res| {
            let _ = res.as_ref().inspect(|v| last = v.len() < 50);
        });
        last
    });

    view! {
      <section id="filter-list">
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
              </datalist>
            </div>
            <Select
              name="status"
              indicator="Status"
              options=[("", "All"), ("verified", "Verified"), ("pb", "Personal Best"), ("wr", "World Record")]
            />
            <Show when=move || *perm.read()>
              <div>
                <label for="user" class="indicator">
                  "User ID"
                </label>
                <input class="select" type="number" name="user" id="user" min="1" step="1" />
              </div>
            </Show>
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
                      class:button-2=true
                      class:run=true
                      headers=vec![
                        "id".into(),
                        "date".into(),
                        "layout".into(),
                        "category".into(),
                        "map".into(),
                        "user".into(),
                        "proof".into(),
                        "time".into(),
                        "".into(),
                        "".into(),
                      ]
                    >
                      {runs
                        .into_iter()
                        .map(|r| {
                          let id = r.id;
                          let u_id = r.user.user_id;
                          view! {
                            <TableLine>
                              {r.id} {format!("{}", r.created_at.format("%d/%m/%Y %H:%M"))}
                              {format!("Layout {}", r.section.layout)} {r.section.category.clone()}
                              {r.section.map.clone()}
                              <a href=format!("/user/{}", r.user.user_id)>{r.user.username.clone()}</a>
                              <a href=r.proof.clone() target="_blank">
                                "link"
                              </a> {format!("{} sec", r.time.to_string())}
                              <Show when=move || {
                                perm.get()
                              }>
                                {
                                  let r = r.clone();
                                  view! {
                                    <A
                                      class:edit=true
                                      href=move || {
                                        let mut params = params.get();
                                        params.replace_decoded("run", serde_qs::to_string(&r).unwrap_or("".into()));
                                        format!("edit{}", params.to_query_string())
                                      }
                                    >
                                      <div></div>
                                    </A>
                                  }
                                }
                              </Show> <Show when=move || del_perm.get() || user_id.get() == u_id>
                                <A
                                  class:delete=true
                                  href=move || format!("{}{}", id.to_string(), params.get().to_query_string())
                                >
                                  <div></div>
                                </A>
                              </Show>
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
pub fn EditRun(#[prop(into)] redirect: Signal<String>) -> impl IntoView {
    let user = expect_context::<UserResource>();
    let run = Memo::new(|_| {
        use_query_map()
            .get()
            .get("run")
            .map(|r| serde_qs::from_str::<Run>(&r).ok())
            .flatten()
            .ok_or(ApiError::InvalidInput)
    });
    let maps = Resource::new(
        move || run.get().map(|r| (r.section.layout, r.section.category)),
        move |combo| async move {
            if let Ok(combo) = combo { get_maps(combo.0, combo.1).await } else { Err(ApiError::NotFound) }
        },
    );
    let action = expect_context::<ServerAction<UpdateRun>>();
    let result = Signal::derive(move || action.value().get());
    view! {
      <Modal redirect>
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
        <Dialogue action>
          <Redirect slot redirect />
          <Show when=move || { run.read().as_ref().map(|r| r.verified).is_ok_and(|v| !v) }>
            <Checkbox name="verified" indicator="Verify" />
          </Show>
          <Show when=move || user.and_then(|u| u.has(&Permissions::ManageRuns)).opt_res_to_bool()>
            <div>
              <label for="map" class="indicator">
                "Map"
              </label>
              <input class="select" list="maps" name="section_id" id="map" />
              <datalist id="maps">
                <Suspense>
                  {move || {
                    maps
                      .and_then(|v| {
                        v.into_iter()
                          .map(|m| {
                            view! { <option value=m.id>{m.map.clone()}</option> }
                          })
                          .collect_view()
                      })
                  }}
                </Suspense>
              </datalist>
            </div>
          </Show>
          <input name="id" hidden value=move || run.read().as_ref().map(|r| r.id).unwrap_or(-1) />
          <Button slot kind=ButtonType::Link(redirect) text="Cancel" />
          <Button slot kind=ButtonType::Primary text="Update" />
        </Dialogue>
      </Modal>
    }
}

#[component]
pub fn DeleteRun(#[prop(into)] redirect: Signal<String>) -> impl IntoView {
    let id = Signal::derive(|| use_params_map().get().get("id").unwrap().parse::<i32>().unwrap_or(-1));
    let action = expect_context::<ServerAction<Delete>>();
    let result = Signal::derive(move || action.value().get());
    view! {
      <Modal redirect>
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
        <Dialogue action>
          <Redirect slot redirect />
          <input type="text" name="id" hidden value=id />
          <Button slot kind=ButtonType::Link(redirect) text="Cancel" />
          <Button slot kind=ButtonType::Danger text="Delete" />
        </Dialogue>
      </Modal>
    }
}

#[component]
pub fn ManageSections() -> impl IntoView {
    let params = use_query_map();
    let filters = Signal::derive(move || {
        params.with(|p| SectionFilters {
            patch: Some("2.13".into()),
            layout: p.get("layout").filter(|v| !v.is_empty()),
            category: p.get("category").filter(|v| !v.is_empty()),
            map: p.get("map").filter(|v| !v.is_empty()),
            status: p.get("status").filter(|v| !v.is_empty()),
            sort: p.get("sort").filter(|v| !v.is_empty()).unwrap_or("patch".into()),
            ascending: p.get("order").filter(|v| !v.is_empty()).is_none_or(|s| s == "asc"),
        })
    });
    let offset = Signal::derive(move || params.get().get("page").unwrap_or(String::from("0")).parse::<i32>().unwrap());
    let delete = ServerAction::<DeleteSection>::new();
    let update = ServerAction::<UpdateSection>::new();
    let add = ServerAction::<AddSection>::new();
    provide_context(delete);
    provide_context(update);
    provide_context(add);
    let sections = Resource::new(
        move || (filters.get(), update.version().get(), delete.version().get(), offset.get()),
        move |f| get_sections(f.0, f.3 * 50),
    );
    let last = Signal::derive(move || {
        let mut last = true;
        sections.map(|res| {
            let _ = res.as_ref().inspect(|v| last = v.len() < 50);
        });
        last
    });

    view! {
      <section id="filter-list">
        <Outlet />
        <div class="row padding">
          <A href="fix" attr:class="button secondary">
            "Fix Rankings"
          </A>
          <A href="add" attr:class="button primary">
            "Add Section"
          </A>
        </div>
        <Collapsible id="filter" class="filter" header=|| "Show Filters">
          <Filter attr:class="filter">
            <Select
              name="sort"
              indicator="Sort By"
              options=[("layout", "Layout"), ("category", "Category"), ("map", "Map")]
            />
            <Select name="order" indicator="Order By" options=[("asc", "Ascending"), ("desc", "Descending")] />
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
              </datalist>
            </div>
            <Select
              name="status"
              indicator="Status"
              options=[("", "All"), ("submittable", "Submittable"), ("closed", "Closed")]
            />
          </Filter>
        </Collapsible>
        <Suspense fallback=|| { "Fetching Sections" }>
          <ErrorBoundary fallback=|_| {
            view! { <div class="error-display">"You are not logged in"</div> }
          }>
            {move || {
              sections
                .and_then(|sections| {
                  let sections = sections.clone();
                  view! {
                    <Table
                      class:button-2=true
                      class:section=true
                      headers=vec![
                        "id".into(),
                        "layout".into(),
                        "category".into(),
                        "map".into(),
                        "code".into(),
                        "submittable".into(),
                        "".into(),
                        "".into(),
                      ]
                    >
                      {sections
                        .into_iter()
                        .map(|s| {
                          let id = s.id;
                          view! {
                            <TableLine>
                              {s.id} {format!("Layout {}", s.layout)} {s.category.clone()} {s.map.clone()}
                              {s.code.clone()} {if s.submittable { "Yes" } else { "No" }}
                              <A
                                class:edit=true
                                href=move || {
                                  let mut params = params.get();
                                  params.replace_decoded("section", serde_qs::to_string(&s).unwrap_or("".into()));
                                  format!("edit{}", params.to_query_string())
                                }
                              >
                                <div></div>
                              </A>
                              <A
                                class:delete=true
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
pub fn EditSection(#[prop(into)] redirect: Signal<String>) -> impl IntoView {
    let section = Memo::new(|_| {
        use_query_map()
            .get()
            .get("section")
            .map(|r| serde_qs::from_str::<Section>(&r).ok())
            .flatten()
            .ok_or(ApiError::InvalidSection)
    });
    let action = expect_context::<ServerAction<UpdateSection>>();
    let run_fix = ServerAction::<RecalculateRuns>::new();
    let result = Signal::derive(move || action.value().get());

    view! {
      <Modal redirect>
        <h1>"Edit Section"</h1>
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
                    ApiError::NotFound | ApiError::InvalidSection => "🛈 Invalid section",
                    ApiError::InvalidInput => "🛈 No updated parameters",
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
          <div class="hidden">{move || section.read().as_ref().map(|_| ()).map_err(|e| e.clone())}</div>
        </ErrorBoundary>
        <Dialogue action=run_fix class:row=true class:no-wrap=true>
          <Redirect slot redirect />
          <span>"Fix points, wr and pb flags"</span>
          <input
            type="text"
            name="section_id"
            hidden
            value=move || section.read().as_ref().map(|s| s.id).unwrap_or(-1)
          />
          <Button slot kind=ButtonType::Primary text="Fix" />
        </Dialogue>
        <Dialogue action>
          <Redirect slot redirect />
          <Freeform
            name="name"
            indicator="Map Name"
            error="Must be 4 to 32 alphanumeric characters"
            {..}
            minlength="4"
            maxlength="32"
            pattern="[a-zA-Z0-9() ]*"
          />
          <Freeform
            name="code"
            indicator="Code"
            error="Must be 4 alphanumeric characters"
            {..}
            maxlength="4"
            minlength="4"
            pattern="[A-Z0-9]*"
          />
          <Select
            name="submittable"
            indicator="Submittable"
            options=[("", "Unchanged"), ("true", "Yes"), ("false", "No")]
          />
          <input name="id" hidden value=move || section.read().as_ref().map(|s| s.id).unwrap_or(-1) />
          <Button slot kind=ButtonType::Link(redirect) text="Cancel" />
          <Button slot kind=ButtonType::Primary text="Update" />
        </Dialogue>
      </Modal>
    }
}

#[component]
pub fn DeleteSection(#[prop(into)] redirect: Signal<String>) -> impl IntoView {
    let id = Signal::derive(|| use_params_map().get().get("id").unwrap().parse::<i32>().unwrap_or(-1));
    let action = expect_context::<ServerAction<DeleteSection>>();
    let result = Signal::derive(move || action.value().get());

    view! {
      <Modal redirect>
        <h1>"Delete Section"</h1>
        <p>"Please confirm your deletion of the section with id " {id}</p>
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
                    ApiError::NotFound => "🛈 Invalid section",
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
        <Dialogue action>
          <Redirect slot redirect />
          <input type="text" name="id" hidden value=id />
          <Button slot kind=ButtonType::Link(redirect) text="Cancel" />
          <Button slot kind=ButtonType::Danger text="Delete" />
        </Dialogue>
      </Modal>
    }
}

#[component]
pub fn AddSection(#[prop(into)] redirect: Signal<String>) -> impl IntoView {
    let action = expect_context::<ServerAction<AddSection>>();
    let result = Signal::derive(move || action.value().get());

    view! {
      <Modal redirect>
        <h1>"Add Section"</h1>
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
                    ApiError::AlreadyExists => "🛈 Section already exists",
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
        <Dialogue action>
          <Redirect slot redirect />
          <Select
            name="layout"
            indicator="Layout"
            options=[("1", "Layout 1"), ("2", "Layout 2"), ("3", "Layout 3"), ("4", "Layout 4"), ("5", "Layout 5")]
            attr:required
          />
          <Select
            name="category"
            indicator="Category"
            options=[("Standard", "Standard"), ("Gravspeed", "Gravspeed")]
            attr:required
          />
          <Freeform
            name="map"
            indicator="Map"
            error="Must be 4 to 32 alphanumeric characters"
            {..}
            minlength="4"
            maxlength="32"
            pattern="[a-zA-Z0-9 ]*"
            required=true
          />
          <Freeform
            name="code"
            indicator="Code"
            error="Must be 4 uppercase alphanumeric characters"
            {..}
            minlength="4"
            maxlength="4"
            pattern="[A-Z0-9]*"
            required=true
          />
          <Checkbox name="submittable" indicator="Submittable" />
          <Button slot kind=ButtonType::Link(redirect) text="Cancel" />
          <Button slot kind=ButtonType::Primary text="Add" />
        </Dialogue>
      </Modal>
    }
}

#[component]
pub fn FixRankings(#[prop(into)] redirect: Signal<String>) -> impl IntoView {
    let action = ServerAction::<RecalculateRankings>::new();
    let result = Signal::derive(move || action.value().get());

    view! {
      <Modal redirect>
        <h1>"Add Section"</h1>
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
                    ApiError::InvalidInput => "🛈 Invalid combo",
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
        <Dialogue action>
          <Redirect slot redirect />
          <Select
            name="layout"
            indicator="Layout"
            options=[
              ("", "Combined"),
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
            options=[("", "Combined"), ("Standard", "Standard"), ("Gravspeed", "Gravspeed")]
          />
          <Button slot kind=ButtonType::Link(redirect) text="Cancel" />
          <Button slot kind=ButtonType::Primary text="Recalculate" />
        </Dialogue>
      </Modal>
    }
}

#[component]
pub fn ManageUser() -> impl IntoView {
    let params = use_query_map();
    let filters = Signal::derive(move || {
        params.with(|p| UserFilters {
            id: p.get("id").filter(|v| !v.is_empty()).map(|v| v.parse::<i64>().ok()).flatten(),
            name: p.get("name").filter(|v| !v.is_empty()),
            permission: p
                .get_all("permission")
                .map(|v| v.iter().filter_map(|v| serde_qs::from_str::<Permissions>(v).ok()).collect())
                .unwrap_or(vec![]),
            sort: p.get("sort").filter(|v| !v.is_empty()).unwrap_or("name".into()),
            ascending: p.get("order").filter(|v| !v.is_empty()).is_none_or(|s| s == "asc"),
        })
    });
    let offset = Signal::derive(move || params.get().get("page").unwrap_or(String::from("0")).parse::<i32>().unwrap());
    let update = ServerAction::<UpdateUser>::new();
    provide_context(update);
    let users =
        Resource::new(move || (filters.get(), update.version().get(), offset.get()), move |f| get_users(f.0, f.2 * 50));
    let last = Signal::derive(move || {
        let mut last = true;
        users.map(|res| {
            let _ = res.as_ref().inspect(|v| last = v.len() < 50);
        });
        last
    });

    view! {
      <section id="filter-list">
        <Outlet />
        <Collapsible id="filter" class="filter" header=|| "Show Filters">
          <Filter attr:class="filter">
            <Select name="sort" indicator="Sort By" options=[("name", "Name"), ("created_at", "Join Date")] />
            <Select name="order" indicator="Order By" options=[("asc", "Ascending"), ("desc", "Descending")] />
            <Freeform name="id" indicator="ID" {..} type="number" />
            <Freeform name="name" indicator="Name" />
          </Filter>
        </Collapsible>
        <Suspense fallback=|| { "Fetching Sections" }>
          <ErrorBoundary fallback=|_| {
            view! { <div class="error-display">"You are not logged in"</div> }
          }>
            {move || {
              users
                .and_then(|users| {
                  let users = users.clone();
                  view! {
                    <Table
                      class:user=true
                      headers=vec!["id".into(), "name".into(), "join date".into(), "".into(), "".into()]
                    >
                      {users
                        .into_iter()
                        .map(|u| {
                          let id = u.id;
                          view! {
                            <TableLine>
                              {u.id} {u.username.clone()} {format!("{}", u.created_at.format("%d/%m/%Y %H:%M"))}
                              <A
                                class:edit=true
                                href=move || {
                                  let mut params = params.get();
                                  params.replace_decoded("section", serde_qs::to_string(&u).unwrap_or("".into()));
                                  format!("edit{}", params.to_query_string())
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
