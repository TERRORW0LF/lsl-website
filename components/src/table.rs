use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_params_map};

#[component]
pub fn Table(#[prop(into)] headers: Signal<Vec<String>>, children: ChildrenFragment) -> impl IntoView {
    view! {
        <div class="grid">
            {move || headers.get().into_iter().map(|v| view! { <span class="heading">{v}</span> }).collect_view()}
            <div class="divider header"></div>
            {children()
                .nodes
                .into_iter()
                .map(|r| {
                    view! {
                        {r}
                        <div class="divider"></div>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
pub fn TableLine(children: ChildrenFragment) -> impl IntoView {
    children().nodes.into_iter().map(|r| view! { <span>{r}</span> }).collect_view()
}

#[component]
pub fn Pager(#[prop(into)] name: Signal<String>, #[prop(into)] last: Signal<bool>) -> impl IntoView {
    let params = use_params_map();
    let offset =
        Signal::derive(move || params.get().get(&*name.read()).unwrap_or(String::from("0")).parse::<i32>().unwrap());
    view! {
        <div class="pages row">
            <Show when=move || offset.read() != 0 fallback=|| view! { <div class="arrow disabled">"<"</div> }>
                <A
                    class:arrow=true
                    href=move || {
                        let mut map = params.get();
                        map.replace(name.get(), (offset.get() - 1).to_string());
                        map.to_query_string()
                    }
                >
                    "<"
                </A>
            </Show>
            <div class="page">{move || offset.get() + 1}</div>
            <Suspense fallback=|| view! { <div class="arrow disabled">">"</div> }>
                <Show when=move || !*last.read() fallback=|| view! { <div class="arrow disabled">">"</div> }>
                    <A
                        class:arrow=true
                        href=move || {
                            let mut map = params.get();
                            map.replace(name.get(), (offset.get() + 1).to_string());
                            map.to_query_string()
                        }
                    >
                        ">"
                    </A>
                </Show>
            </Suspense>
        </div>
    }
}
