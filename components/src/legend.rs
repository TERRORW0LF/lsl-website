use leptos::prelude::*;
use types::api::Title;

#[component]
pub fn RankingLegend(#[prop(into)] titles: Vec<(Title, Signal<String>)>) -> impl IntoView {
    view! {
        <div class="legend row narrow">
            {titles
                .into_iter()
                .enumerate()
                .map(|(i, (t, s))| {
                    view! {
                        <div class="tooltip-box">
                            <span
                                class=format!("{} bg title", t.to_string())
                                aria-describedby=format!("tooltip-{i}")
                            >
                                {t.to_string()}
                            </span>
                            <span role="tooltip" id=format!("tooltip-{i}") class="tooltip">
                                {s}
                            </span>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}
