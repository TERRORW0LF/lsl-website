use leptos::prelude::*;
use leptos_router::{components::Form, hooks::use_query_map};
use types::leptos::SelectFragment;

#[component]
pub fn Filter(children: ChildrenFragment) -> impl IntoView {
    view! {
        <Form method="GET" action="">
            <div class="row">
                {children().nodes.into_iter().map(|v| view! { <div class="input-box">{v}</div> }).collect_view()}
            </div>
            <input type="submit" class="button" value="Apply" />
        </Form>
    }
}

#[component]
pub fn Select(
    #[prop(into)] name: String,
    #[prop(into)] indicator: String,
    #[prop(optional)] selected: usize,
    #[prop(into)] options: SelectFragment,
) -> impl IntoView {
    let val = name.clone();
    let key = Memo::new(move |_| use_query_map().read().get(&val));
    view! {
        <label for=name.clone() class="indicator">
            {indicator}
        </label>
        <select class="select" name=name.clone() id=name autocomplete="off">
            {options
                .nodes
                .into_iter()
                .enumerate()
                .map(|(i, v)| {
                    let key_val = v.0.clone();
                    view! {
                        <option
                            value=v.0
                            selected=move || {
                                leptos::logging::warn!("{:?}", key.get());
                                ((i == selected && key.read().is_none()) || key.read() == Some(key_val.clone()))
                                    .then_some("")
                            }
                        >
                            {v.1}
                        </option>
                    }
                })
                .collect_view()}
        </select>
    }
}
