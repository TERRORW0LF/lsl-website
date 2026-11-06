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
                  ((i == selected && key.read().is_none()) || key.read() == Some(key_val.clone())).then_some("")
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

#[component]
pub fn Freeform(
    #[prop(into)] name: String,
    #[prop(into)] indicator: String,
    #[prop(optional, into)] error: Option<String>,
) -> impl IntoView {
    view! {
      <input name=name.clone() id=name.clone() placeholder=" " />
      <label for=name.clone() class="placeholder">
        {indicator}
      </label>
      <label for=name class="error">
        {error}
      </label>
    }
}

#[component]
pub fn MultiLine(
    #[prop(into)] name: String,
    #[prop(into)] indicator: String,
    #[prop(optional, into)] error: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
      <textarea name=name.clone() id=name.clone()>
        {children()}
      </textarea>
      <label for=name.clone() class="indicator">
        {indicator}
      </label>
      <label for=name class="error">
        {error}
      </label>
    }
}

#[component]
pub fn Checkbox(
    #[prop(into)] name: String,
    #[prop(into)] indicator: String,
    #[prop(optional, into)] checked: Signal<bool>,
) -> impl IntoView {
    view! {
      <label class="descriptor row">
        <span>{indicator}</span>
        <input type="checkbox" name=name checked=checked value="true" />
      </label>
    }
}
