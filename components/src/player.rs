use leptos::prelude::*;
use types::internal::Proof;

#[component]
pub fn Player(proof: Signal<Option<Proof>>, cover: String) -> impl IntoView {
    let (play, set_play) = signal(false);
    Effect::new(move |old: Option<Option<String>>| {
        let url = proof.get().map(|p| p.url);
        if old.flatten() != url {
            set_play(false);
        }
        url
    });
    view! {
        <div class="video">
            <Show
                when=move || *play.read()
                fallback=move || {
                    view! {
                        <Show when=move || proof.read().is_some()>
                            <div class="buttons">
                                <Show
                                    when=move || proof.get().unwrap().yt_id.is_some()
                                    fallback=move || {
                                        view! {
                                            <a
                                                href=move || proof.get().unwrap().url
                                                class="play-wrapper"
                                                target="_blank"
                                            >
                                                <div></div>
                                            </a>
                                        }
                                    }
                                >
                                    <button
                                        class="play-wrapper"
                                        on:click=move |_| { set_play(true) }
                                    >
                                        <div></div>
                                    </button>
                                </Show>
                                <br />
                                <a
                                    class="external"
                                    href=move || proof.get().unwrap().url
                                    target="_blank"
                                >
                                    "Open in new Tab"
                                </a>
                            </div>
                        </Show>
                        <div class="no-vid">
                            <img
                                src=format!("/cdn/maps/{}.jpg", cover)
                                alt=format!("Picture of {}", cover)
                            />
                        </div>
                    }
                }
            >
                <iframe
                    src=format!(
                        "https://www.youtube-nocookie.com/embed/{}?autoplay=1&rel=0&modestbranding=1&showinfo=0",
                        proof.get().unwrap().yt_id.unwrap(),
                    )
                    allowfullscreen
                ></iframe>
            </Show>
        </div>
    }
}
