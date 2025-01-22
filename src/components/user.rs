use crate::{
    app::UserResource,
    server::{
        api::{get_runs_user, get_user, RunFilters},
        auth::User,
    },
};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

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
    let user = expect_context::<UserResource>();
    let (filters, set_filters) = signal(RunFilters::default());

    view! {
        <section id="manage-runs">
            <div>"filters"</div>
            <div class="grid">
                <span>"id"</span>
                <span>"date"</span>
                <span>"layout"</span>
                <span>"category"</span>
                <span>"map"</span>
                <span>"proof"</span>
                <span>"time"</span>
            </div>
            <Suspense fallback=|| { "Fetching Runs" }>
                <ErrorBoundary fallback=|_| {
                    view! { <div class="error-display">"You are not logged in"</div> }
                }>
                    {move || {
                        user.get()
                            .map(|res| {
                                res.map(|user| {
                                    view! { <ManageRunsList user filters /> }
                                })
                            })
                    }}
                </ErrorBoundary>
            </Suspense>
        </section>
    }
}

#[component]
fn ManageRunsList(user: User, filters: ReadSignal<RunFilters>) -> impl IntoView {
    let runs = Resource::new(move || filters.get(), move |f| get_runs_user(user.id, f, 0));
    view! {
        <Suspense fallback=|| {
            "Fetching Runs"
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
                                        <span>"layout " {r.layout}</span>
                                        <span>{r.category}</span>
                                        <span>{r.map}</span>
                                        <span>
                                            <a href=r.proof>"link"</a>
                                        </span>
                                        <span>{r.time.to_string()} " sec"</span>
                                        <div class="body">{r.id}</div>
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                    })
            }}
        </Suspense>
    }
}
