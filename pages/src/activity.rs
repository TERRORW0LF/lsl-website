use chrono::{Local, NaiveDateTime, TimeZone};
use components::{Collapsible, Filter, Pager, Select, Table};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
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
    let offset = Signal::derive(move || params.get().get("page").unwrap_or(String::from("0")).parse::<i32>().unwrap());
    let activities = Resource::new(move || (filters.get(), offset.get()), move |f| get_activity(f.0, f.1 * 50));
    let last = Signal::derive(move || {
        let mut last = true;
        activities.map(|res| {
            let _ = res.as_ref().inspect(|v| last = v.len() < 50);
        });
        last
    });

    view! {
      <section id="filter-list" class="activity">
        <Collapsible id="filter" class="filter" header=|| "Show Filters">
          <Filter attr:class="filter">
            <Select name="sort" indicator="Sort By" options=[("date", "Date"), ("section", "Section")] />
            <Select
              name="order"
              indicator="Order By"
              selected=1
              options=[("asc", "Ascending"), ("desc", "Descending")]
            />
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
            <Select
              name="event"
              indicator="Event"
              options=[("", "All"), ("join", "User Joined"), ("rank", "Rank changed"), ("title", "Title changed")]
            />
            <div class="input-box">
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
          </Filter>
        </Collapsible>
        <Suspense fallback=|| { "Fetching Activities" }>
          <ErrorBoundary fallback=|_| {
            view! { <div class="error-display">"Failed to load activities"</div> }
          }>
            {move || {
              activities
                .and_then(|activity| {
                  let activity = activity.clone();
                  view! {
                    <Table headers=vec![
                      "date".into(),
                      "user".into(),
                      "patch".into(),
                      "layout".into(),
                      "category".into(),
                      "old title".into(),
                      "new title".into(),
                      "old rank".into(),
                      "new rank".into(),
                    ]>
                      {activity
                        .into_iter()
                        .map(|r| {
                          view! {
                            <span>{format!("{}", r.created_at.format("%d/%m/%Y %H:%M"))}</span>
                            <span>{r.user.username}</span>
                            <span>{r.patch.map(|v| format!("Patch {v}")).unwrap_or("-".into())}</span>
                            <span>{r.layout.map(|v| format!("Layout {v}")).unwrap_or("-".into())}</span>
                            <span>{r.category.unwrap_or("-".into())}</span>
                            <span class=format!(
                              "{} color",
                              r.title_old.map(|v| v.to_string()).unwrap_or_default(),
                            )>{r.title_old.clone().map(|v| v.to_string()).unwrap_or("-".into())}</span>
                            <span class=format!(
                              "{} color",
                              r.title_new.map(|v| v.to_string()).unwrap_or_default(),
                            )>{r.title_new.clone().map(|v| v.to_string()).unwrap_or("-".into())}</span>
                            <span class=format!(
                              "rank-{} color",
                              r.rank_old.map(|v| v.to_string()).unwrap_or_default(),
                            )>{r.rank_old.map(|v| format!("#{v}")).unwrap_or("-".into())}</span>
                            <span class=format!(
                              "rank-{} color",
                              r.rank_new.map(|v| v.to_string()).unwrap_or_default(),
                            )>{r.rank_new.map(|v| format!("#{v}")).unwrap_or("-".into())}</span>
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
