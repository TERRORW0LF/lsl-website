use leptos::prelude::*;
use leptos_router::components::Form;
use types::leptos::SelectFragment;

#[component]
pub fn Filter(children: ChildrenFragment) -> impl IntoView {
    view! {
        <Form method="GET" action="">
            <div class="row">
                {children()
                    .nodes
                    .into_iter()
                    .map(|v| view! { <div class="input-box">{v}</div> })
                    .collect_view()}
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
    view! {
        <label for=name.clone() class="indicator">
            {indicator}
        </label>
        <select class="select" name id=name.clone()>
            {options
                .nodes
                .into_iter()
                .enumerate()
                .map(|(i, v)| {
                    view! {
                        <option value=v.0 selected=move || (i == selected).then_some("")>
                            {v.1}
                        </option>
                    }
                })
                .collect_view()}
        </select>
    }
}
