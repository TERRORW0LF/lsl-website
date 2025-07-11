use charming::{Chart, Echarts, WasmRenderer, theme::Theme};
use leptos::{html::Div, prelude::*};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::js_sys;

#[component]
pub fn Chart(chart: Chart) -> impl IntoView {
    let chart_elem: NodeRef<Div> = NodeRef::new();
    let (width, set_width) = signal::<i32>(0);
    let mut echart: Option<Echarts> = None;

    Effect::watch(
        move || width.get(),
        move |_, _, _| {
            if echart.is_none() {
                if let Some(ch) = WasmRenderer::new_opt(None, None)
                    .theme(Theme::Walden)
                    .render("chart", &chart)
                    .ok()
                {
                    echart = Some(ch);
                }
            }
            if let Some(echart) = echart.as_ref() {
                echart.resize(JsValue::UNDEFINED);
            }
        },
        true,
    );
    let mut obs: Option<web_sys::ResizeObserver> = None;
    let mut has_obs = false;
    Effect::new(move |_| {
        if obs.is_none() {
            let a =
                Closure::<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>::new(move |entries: js_sys::Array, _| {
                    set_width(
                        entries.to_vec()[0]
                            .clone()
                            .unchecked_into::<web_sys::ResizeObserverEntry>()
                            .content_rect()
                            .width() as i32,
                    );
                });
            obs = web_sys::ResizeObserver::new(a.as_ref().unchecked_ref()).ok();
            a.forget();
        }
        if let Some(observer) = obs.as_mut() {
            if let Some(elem) = chart_elem.get() {
                if !has_obs {
                    observer.observe(&elem.into());
                    has_obs = true;
                }
            }
        }
    });
    view! { <div id="chart" node_ref=chart_elem></div> }
}
