use leptos::{
    prelude::*,
    server_fn::{Http, ServerFn, client::Client, codec::PostUrl, request::ClientReq},
};
use leptos_router::{components::A, hooks::use_query_map};
use serde::de::DeserializeOwned;
use web_sys::FormData;

#[component]
pub fn Modal(children: Children) -> impl IntoView {
    view! {
      <A attr:class="toner" href=|| format!("../{}", use_query_map().get().to_query_string())>
        <div />
      </A>
      <section id="box">{children()}</section>
    }
}

#[component]
pub fn Dialogue<ServFn, OutputProtocol>(action: ServerAction<ServFn>, children: Children) -> impl IntoView
where
    ServFn: DeserializeOwned + ServerFn<Protocol = Http<PostUrl, OutputProtocol>> + Clone + Send + Sync + 'static,
    <<ServFn::Client as Client<ServFn::Error>>::Request as ClientReq<ServFn::Error>>::FormData: From<FormData>,
    ServFn: Send + Sync + 'static,
    ServFn::Output: Send + Sync + 'static,
    ServFn::Error: Send + Sync + 'static,
    <ServFn as ServerFn>::Client: Client<<ServFn as ServerFn>::Error>,
{
    view! {
      <ActionForm action>
        {children()} <div class="row">
          <A attr:class="button secondary" href=|| format!("../{}", use_query_map().get().to_query_string())>
            "Cancel"
          </A>
          <input type="submit" class="button primary" value="Submit" />
        </div>
      </ActionForm>
    }
}
