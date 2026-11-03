use leptos::{
    either::{Either, EitherOf4},
    prelude::*,
    server_fn::{Http, ServerFn, client::Client, codec::PostUrl, request::ClientReq},
};
use leptos_router::{components::A, hooks::use_query_map};
use serde::de::DeserializeOwned;
use web_sys::FormData;

#[component]
pub fn Modal(#[prop(into)] redirect: Signal<String>, children: Children) -> impl IntoView {
    view! {
      <A attr:class="toner" href=move || redirect.get()>
        <div />
      </A>
      <section id="box">{children()}</section>
    }
}

#[component]
pub fn Dialogue<ServFn, OutputProtocol>(
    action: ServerAction<ServFn>,
    #[prop(optional)] redirect: Option<Redirect>,
    #[prop(default=vec![])] button: Vec<Button>,
    children: ChildrenFragment,
) -> impl IntoView
where
    ServFn: DeserializeOwned + ServerFn<Protocol = Http<PostUrl, OutputProtocol>> + Clone + Send + Sync + 'static,
    <<ServFn::Client as Client<ServFn::Error>>::Request as ClientReq<ServFn::Error>>::FormData: From<FormData>,
    ServFn: Send + Sync + 'static,
    ServFn::Output: Send + Sync + 'static,
    ServFn::Error: Send + Sync + 'static,
    <ServFn as ServerFn>::Client: Client<<ServFn as ServerFn>::Error>,
{
    let len = button.len();
    view! {
      <ActionForm action>
        {children().nodes.into_iter().map(|v| view! { <div class="input-box">{v}</div> }).collect_view()}
        {if let Some(redirect) = redirect {
          Either::Right(move || {
            view! {
              <input
                type="text"
                name="redirect"
                hidden
                value=move || {
                  if let Some(r) = redirect.redirect.read() {
                    Some(format!("{}{}", r, use_query_map().get().to_query_string()))
                  } else {
                    use_query_map().read().get("redirect")
                  }
                }
              />
            }
          })
        } else {
          Either::Left(())
        }}
        <div class:row=len
          .gt(
            &1,
          )>
          {button
            .into_iter()
            .map(|b| match b.kind {
              ButtonType::Primary => EitherOf4::A(view! { <input type="submit" class="button primary" value=b.text /> }),
              ButtonType::Secondary => {
                EitherOf4::B(view! { <input type="submit" class="button secondary" value=b.text /> })
              }
              ButtonType::Danger => EitherOf4::C(view! { <input type="submit" class="button danger" value=b.text /> }),
              ButtonType::Link(href) => {
                EitherOf4::D(
                  view! {
                    <A attr:class="button secondary" href=move || href.get()>
                      {b.text}
                    </A>
                  },
                )
              }
            })
            .collect_view()}
        </div>
      </ActionForm>
    }
}

#[slot]
pub struct Redirect {
    #[prop(optional, into)]
    pub redirect: Option<Signal<String>>,
}

#[slot]
pub struct Button {
    #[prop(default = ButtonType::Primary)]
    pub kind: ButtonType,
    #[prop(into)]
    pub text: String,
}

pub enum ButtonType {
    Primary,
    Secondary,
    Danger,
    Link(Signal<String>),
}
