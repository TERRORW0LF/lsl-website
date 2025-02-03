use crate::{
    app::UserResource,
    server::{
        api::{get_runs_user, get_user, RunFilters},
        auth::{Delete, User},
    },
};
use leptos::prelude::*;
use leptos_router::{
    components::A,
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
    let (filters, set_filters) = signal(RunFilters::default());
    let offset = Signal::derive(|| {
        use_query_map()
            .get()
            .get("page")
            .unwrap_or(String::from("0"))
            .parse::<i32>()
            .unwrap()
    });
    let delete = ServerAction::<Delete>::new();
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
            <div>"filters"</div>
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
                                                    <span>"layout " {r.layout}</span>
                                                    <span>{r.category}</span>
                                                    <span>{r.map}</span>
                                                    <span>
                                                        <a href=r.proof>"link"</a>
                                                    </span>
                                                    <span>{r.time.to_string()} " sec"</span>
                                                    <ActionForm action=delete>
                                                        <input type="text" hidden readonly name="id" value=r.id />
                                                        <label class="delete">
                                                            <input hidden type="submit" />
                                                            <div></div>
                                                        </label>
                                                    </ActionForm>
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
                <A
                    href=move || {
                        if offset.read() == 0 {
                            String::new()
                        } else {
                            format!("?page={}", offset.get() - 1)
                        }
                    }
                    class:disabled=move || offset.read() == 0
                    class:arrow=true
                >
                    "<"
                </A>
                <div class="page">{move || offset.get() + 1}</div>
                <A href=move || format!("?page={}", offset.get() + 1) class:arrow=true>
                    ">"
                </A>
            </div>
        </section>
    }
}
