use crate::server::auth::get_user;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn Profile() -> impl IntoView {
    let user_id = Signal::derive(|| use_params_map().get().get("id"));
    let user = Resource::new(move || user_id.get(), |_| get_user());
    view! {
        <Suspense fallback=|| { "Fetching User" }>
            <section id="profile">
                {move || {
                    user.get()
                        .map(|res| {
                            res.map(|u| {
                                view! {
                                    <img src=u.pfp />
                                    <h1>{u.username}</h1>
                                    <p>{u.bio}</p>
                                }
                            })
                        })
                }}
            </section>
        </Suspense>
    }
}
