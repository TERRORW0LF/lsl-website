use leptos::prelude::*;

#[component]
pub fn Collapsible<C>(
    #[prop(into)] id: String,
    #[prop(optional, into)] header: ViewFnOnce,
    children: TypedChildren<C>,
) -> impl IntoView
where
    C: IntoView + 'static,
{
    let children = children.into_inner();

    view! {
        <details>
            <summary class="row narrow" aria-details=id>
                <span role="term" class="icon"></span>
                {header.run()}
            </summary>
        </details>
        <div role="definition" id class="content">
            <div>
                <div class="inner">{children()}</div>
            </div>
        </div>
    }
}
