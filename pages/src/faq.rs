use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{components::A, hooks::use_location};

#[component]
pub fn FAQ() -> impl IntoView {
    let hash = use_location().hash;

    view! {
        <Title text="FAQ" />
        <h1>"FAQ"</h1>
        <section id="faq">
            <section id="codes">
                <details open=move || {
                    leptos::logging::log!("{}", hash.get());
                    leptos::logging::log!("{}", (hash.get() == "#codes"));
                    (hash.get() == "#codes").then_some("")
                }>
                    <summary>
                        <h2 role="term" aria-details="filters" class="icon">
                            "Workshop Codes"
                        </h2>
                    </summary>
                </details>
                <div role="definition" id="filters" class="content">
                    <div>
                        <div class="inner">
                            <p>
                                "The main code for Lucio Surf is "<strong>"KVKKR"</strong>
                                ". To play Lucio Surf copy the code and import it in your custom game lobby."
                            </p>
                            <div class="row">
                                <img src="/cdn/faq/lobby_import.jpg" alt="Lobby import button" />
                                <img src="/cdn/faq/code_import.jpg" alt="Code import screen" />
                            </div>
                            <p>
                                "If you save the code as a preset, be aware that those do not update when the code "
                                "receives update. We heavily recommend you to save the code as a favorite instead. "
                                "Favorites receive updates to the code immediately. Should you save the code as a preset "
                                "please make sure to check for code udate regularly. Submits on old versions of the code "
                                "will not be accepted as outlined in the "
                                <a href="#rules">"submit rules"</a>"."
                            </p>
                            <p>
                                "If you want to test out changes early, we recommend the beta code "
                                <strong>"048YZ"</strong>
                                ". Feedback is welcome and can be provided over our "
                                <A href="/discord">"discord server"</A>"."
                            </p>
                        </div>
                    </div>
                </div>
            </section>
        </section>
    }
}
