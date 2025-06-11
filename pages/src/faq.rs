use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{components::A, hooks::use_location};

#[component]
pub fn FAQ() -> impl IntoView {
    let hash_memo = use_location().hash;
    let (hash, set_hash) = signal(String::new());
    Effect::new(move |_| set_hash.set(hash_memo.get()));

    view! {
        <Title text="FAQ" />
        <section id="faq">
            <h1>"FAQ"</h1>
            <hr />
            <section id="codes">
                <details open=move || (hash.get() == "#play").then_some("")>
                    <summary>
                        <span role="term" aria-details="filters" class="icon">
                            "How can I join or host a Lucio Surf lobby?"
                        </span>
                    </summary>
                </details>
                <div role="definition" id="filters" class="content">
                    <div>
                        <div class="inner">
                            <h5>"Favoriting the workshop code"</h5>
                            <p>
                                "The main code for Lucio Surf is "<strong>"KVKKR"</strong>
                                ". To favorite the code click the "<span class="red">"star"</span>
                                " button on a Lucio Surf lobby in your " "recently played tab."
                            </p>
                            <img src="/cdn/faq/favorite.jpg" alt="Favorite button" />
                            <p>
                                "After favoriting the code go to the Favorites tab and select "
                                <strong>"Lucio Surf"</strong>". To join a lobby use the "
                                <span class="red">"Quick Join"</span>" button " "or double click "
                                <strong>"Lucio Surf"</strong>
                                " and select a specific lobby and click "
                                <span class="red">"Join"</span>"."
                                "To create your own lobby instead, click the "
                                <span class="blue">"Create New Lobby"</span>" button."
                            </p>
                            <div class="row">
                                <img src="/cnd/faq/favorite_tab.jpg" alt="Favorite tab" />
                                <img src="lobby_select.jpg" alt="Lobby select" />
                            </div>
                            <div class="spacing"></div>
                            <h5>"Importing the workshop code"</h5>
                            <p>
                                "If you don't want to favorite the workshop code you can import it directly every time "
                                "you open a lobby. Create a custom game and click the "
                                <span class="red">"Import"</span>
                                " button and input the workshop code."
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
                                <a href="#submit">"submit rules"</a>"."
                            </p>
                            <div class="spacing"></div>
                            <h5>"Beta participation"</h5>
                            <p>
                                "If you want to test out changes early, we recommend the beta code "
                                <strong>"048YZ"</strong>
                                ". Feedback is welcome and can be provided over our "
                                <A href="/discord" target="_blank">
                                    "discord server"
                                </A>". Submits of runs achieved on the beta code "
                                "will not be accepted to the leaderboards."
                            </p>
                        </div>
                    </div>
                </div>
            </section>
            <hr />
            <section id="codes">
                <details open=move || (hash.get() == "#submit").then_some("")>
                    <summary>
                        <span role="term" aria-details="filters" class="icon">
                            "How do I submit a run?"
                        </span>
                    </summary>
                </details>
                <div role="definition" id="filters" class="content">
                    <div>
                        <div class="inner">
                            <h5>"Submitting runs"</h5>
                            <p>
                                "To submit a run to the website log in or create an account. Select submit from the "
                                " account dropdown and fill out the form. "<strong>"Code"</strong>
                                " is a shorthand for " <strong>"Layout"</strong>", "
                                <strong>"Category"</strong>", and " <strong>"Map"</strong>
                                " which will be autofilled when typing in the code. The "
                                <strong>"Video ID"</strong>
                                " field are the 11 characters indentifying the proof video on YouTube:"
                            </p>
                            <ul>
                                <li>
                                    "https://www.youtube.com/watch?v="<strong>"dQw4w9WgXcQ"</strong>
                                </li>
                                <li>"https://youtu.be/"<strong>"dQw4w9WgXcQ"</strong></li>
                            </ul>
                            <div class="spacing"></div>
                            <h5>"Submit rules"</h5>
                            <p>"The run must be"</p>
                            <ul>
                                <li>"recorded in 720p30 or higher"</li>
                                <li>
                                    "uploaded to "<a href="https://youtube.com/." target="_blank">
                                        "YouTube"
                                    </a>
                                </li>
                                <li>
                                    "trimmed to only include one attempt (that being the run itself)"
                                </li>
                                <li>"played on the newest update of the official code"</li>
                            </ul>
                        </div>
                    </div>
                </div>
            </section>
            <hr />
        </section>
    }
}
