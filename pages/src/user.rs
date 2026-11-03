use leptos::prelude::*;
use server::api::get_user;

#[component]
pub fn Profile(id: Signal<i64>) -> impl IntoView {
    let user = Resource::new(move || id(), |s| get_user(s));
    view! {
      <section id="profile">
        <Suspense fallback=|| {
          "Fetching User"
        }>
          {move || {
            user
              .and_then(|u| {
                view! {
                  <h1>{u.username.clone()}</h1>
                  <div class="row">
                    <div>
                      <img class="avatar" src=format!("/cdn/users/{}.jpg", u.pfp) />
                      <p>{u.bio.clone().unwrap_or("This user has no about me.".into())}</p>
                    </div>
                    <div class="medals"></div>
                  </div>
                  <div class="rankings"></div>
                }
              })
          }}
        </Suspense>
      </section>
    }
}
