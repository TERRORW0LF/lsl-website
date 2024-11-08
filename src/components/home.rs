use leptos::prelude::*;
use leptos_meta::Title;

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Title text="Home" />
        <section id="welcome">
            <h1>"Welcome to the Lucio Surf League!"</h1>
            <p>
                "Since its inception in 2019 "<b>"Lucio Surf"</b>
                " has been a staple in Overwatch's custom game browser. The "
                <b>"Lucio Surf League"</b>
                " has been the proud maintainer and arbitrator of Lucio Surf since the beginning. "
                "Updating the game mode and leaderboards, connecting the community, and holding tournaments, the Lucio Surf League "
                "is your place for anything related to Lucio Surf. We welcome you as another frog in our ranks."
            </p>
        </section>
        <section id="potd">
            <h2>"Player of the day"</h2>
            <div>
                <img src="" />
                <p>
                    "Test is an exceptional player combining superb routing with steady movement. This allows them to top "
                    "any leaderboard that don't involve boosts. While occasionaly dabbling in Gravspeed their main category "
                    "is Standard, prefering the challenge over the speed."
                </p>
            </div>
        </section>
    }
}
