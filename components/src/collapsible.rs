use leptos::{either::Either, prelude::*};

#[component]
pub fn Collapsible<C>(
    #[prop(into)] id: String,
    #[prop(optional, into)] header: ViewFnOnce,
    #[prop(optional)] swapped: bool,
    children: TypedChildren<C>,
) -> impl IntoView
where
    C: IntoView + 'static,
{
    let children = children.into_inner();

    if !swapped {
        Either::Left(view! {
            <details>
                <summary class="row narrow" aria-details=id.clone()>
                    <span role="term" class="icon"></span>
                    {header.run()}
                </summary>
            </details>
            <div role="definition" id=id.clone() class="content">
                <div>
                    <div class="inner">{children()}</div>
                </div>
            </div>
        })
    } else {
        Either::Right(view! {
            <div role="definition" id=id.clone() class="content">
                <div>
                    <div class="inner">{children()}</div>
                </div>
            </div>
            <details>
                <summary class="row narrow" aria-details=id>
                    <span role="term" class="icon"></span>
                    {header.run()}
                </summary>
            </details>
        })
    }
}
