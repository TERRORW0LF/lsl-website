use crate::server::api::get_user;
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
        <Suspense fallback=|| { "Fetching User" }>
            <section id="profile">
                {move || {
                    user.get()
                        .map(|res| {
                            res.map(|u| {
                                view! {
                                    <h1>{u.username}</h1>
                                    <div class="row">
                                        <img
                                            class="avatar"
                                            src=format!("/cdn/users/{}.jpg", u.pfp)
                                        />
                                        <p>
                                            {u.bio.unwrap_or("This user has no about me.".into())}
                                        </p>
                                    </div>
                                }
                            })
                        })
                }}
            </section>
        </Suspense>
    }
}
