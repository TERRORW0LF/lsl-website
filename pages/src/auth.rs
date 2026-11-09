use components::{Button, ButtonType, Checkbox, Dialogue, Freeform, Redirect, Select};
use leptos::prelude::*;
use leptos_meta::Title;
use server::{
    api::get_maps,
    auth::{Login, Register, Submit},
};
use types::api::{ApiError, Section};
use util::escape_regex;
use web_sys::Event;

#[component]
pub fn Login() -> impl IntoView {
    let action = expect_context::<ServerAction<Login>>();
    let result = Signal::derive(move || action.value().get().unwrap_or(Ok(())));
    view! {
      <Title text="Log In" />
      <section id="box">
        <h1>"Sign In"</h1>
        <ErrorBoundary fallback=|e| {
          view! {
            <span class="error">
              {move || {
                let e = e.get().into_iter().next().unwrap().1;
                if e.is::<ApiError>() {
                  let e = e.downcast_ref::<ApiError>().unwrap();
                  match e {
                    ApiError::InvalidCredentials => "🛈 Incorrect username or password",
                    _ => "🛈 Something went wrong. Try again",
                  }
                } else {
                  "🛈 Something went wrong. Try again"
                }
              }}
            </span>
          }
        }>
          <div class="hidden">{result}</div>
        </ErrorBoundary>
        <Dialogue action>
          <Redirect slot />
          <Freeform name="username" indicator="Username" {..} required="" />
          <Freeform name="password" indicator="Password" {..} type="password" required=""></Freeform>
          <Checkbox name="remember" indicator="Remember me?" />
          <Button slot kind=ButtonType::Link("/register".into()) text="Create Account" />
          <Button slot kind=ButtonType::Primary text="Sign In" />
        </Dialogue>
      </section>
    }
}

#[component]
pub fn Register() -> impl IntoView {
    let action = expect_context::<ServerAction<Register>>();
    let result = Signal::derive(move || action.value().get().unwrap_or(Ok(())));
    let (password, set_password) = signal(String::new());
    view! {
      <Title text="Sign Up" />
      <section id="box">
        <h1>"Sign Up"</h1>
        <ErrorBoundary fallback=|e| {
          view! {
            <span class="error">
              {move || {
                let e = e.get().into_iter().next().unwrap().1;
                if e.is::<ApiError>() {
                  let e = e.downcast_ref::<ApiError>().unwrap();
                  match e {
                    ApiError::AlreadyExists => "🛈 Username already exists",
                    _ => "🛈 Something went wrong. Try again",
                  }
                } else {
                  "🛈 Something went wrong. Try again"
                }
              }}
            </span>
          }
        }>
          <div class="hidden">{result}</div>
        </ErrorBoundary>
        <Dialogue action>
          <Freeform
            name="username"
            indicator="Username"
            error="Username must adhere to [a-zA-Z0-9_-.]{2,32}."
            {..}
            type="text"
            required=""
            minlength="2"
            maxlength="32"
            pattern="[a-zA-Z0-9_\\-\\.]*"
          />
          <Freeform
            name="password"
            indicator="Password"
            error="Password must be between 8 and 256 characters long."
            on:input=move |e| set_password(event_target_value::<Event>(&e))
            {..}
            type="password"
            required=""
            minlength="8"
            maxlength="256"
          />
          <Freeform
            name="password_confirm"
            indicator="Repeat password"
            error="Passwords must match."
            {..}
            type="password"
            required=""
            pattern=move || escape_regex(&password.read())
          />
          <Checkbox name="remember" indicator="Remember me?" />
          <Button slot kind=ButtonType::Primary text="Sign Up" />
        </Dialogue>
      </section>
    }
}

#[component]
pub fn Submit() -> impl IntoView {
    let action = ServerAction::<Submit>::new();
    let result = Signal::derive(move || action.value().get().unwrap_or(Ok(())));
    let (code, set_code) = signal(String::new());
    let (maps, set_maps) = signal::<Option<Vec<Section>>>(None);
    view! {
      <Title text="Submit" />
      <section id="box">
        <h1>"Submit"</h1>
        <ErrorBoundary fallback=|e| {
          view! {
            <span class="error">
              {move || {
                let e = e.get().into_iter().next().unwrap().1;
                if e.is::<ApiError>() {
                  let e = e.downcast_ref::<ApiError>().unwrap();
                  match e {
                    ApiError::InvalidSection => "🛈 Invalid map name",
                    ApiError::InvalidYtId => "🛈 YT video id does not exist",
                    _ => "🛈 Something went wrong. Try again",
                  }
                } else {
                  "🛈 Something went wrong. Try again"
                }
              }}
            </span>
          }
        }>
          <div class="hidden">{result}</div>
        </ErrorBoundary>
        <Dialogue action>
          <div class="row no-wrap top">
            <div class="input-box">
              <Freeform
                name="code"
                indicator="Code *"
                error="Invalid code."
                on:input=move |e| set_code(event_target_value::<Event>(&e))
                {..}
                minlength="4"
                maxlength="4"
                pattern="[1-5][sgSG][a-zA-Z0-9]{2}"
              />
            </div>
            <div class="input-box">
              <Freeform
                name="time"
                indicator="Time"
                error="Invalid time."
                {..}
                required=""
                type="number"
                min="0"
                step="0.001"
              />
            </div>
          </div>
          <div class="row no-wrap top">
            <div class="input-box">
              <Select
                name="layout"
                indicator="Layout"
                options=[("1", "Layout 1"), ("2", "Layout 2"), ("3", "Layout 3"), ("4", "Layout 4"), ("5", "Layout 5")]
                prop:selectedIndex=move || {
                  code.get().chars().next().map_or(0, |c| c.to_digit(10).unwrap_or(0) as i32 - 1)
                }
                attr:required=""
              />
            </div>
            <div class="input-box">
              <Select
                name="category"
                indicator="Category"
                options=[("Standard", "Standard"), ("Gravspeed", "Gravspeed")]
                prop:selectedIndex=move || {
                  code
                    .get()
                    .chars()
                    .nth(1)
                    .map_or(
                      0,
                      |c| match c.to_ascii_lowercase() {
                        's' => 0,
                        'g' => 1,
                        _ => -1,
                      },
                    )
                }
                attr:required=""
              />
            </div>
          </div>
          <div>
            <input
              list="maps"
              name="map"
              id="map"
              prop:value=move || {
                match maps.get() {
                  Some(ms) => {
                    let id: String = code
                      .read()
                      .chars()
                      .into_iter()
                      .skip(2)
                      .take(2)
                      .collect::<String>()
                      .to_ascii_uppercase();
                    match ms.into_iter().find(|m| m.code[2..4] == id) {
                      Some(m) => m.map,
                      None => String::new(),
                    }
                  }
                  None => String::new(),
                }
              }
            />
            <datalist id="maps">
              <Await future=get_maps("1".into(), "Standard".into()) let:maps>
                {
                  let maps = maps.clone();
                  let maps = maps.map(|v| v.into_iter().filter(|m| m.submittable).collect());
                  set_maps(maps.clone().ok());
                  maps
                    .map(|v| {
                      v.into_iter()
                        .map(|m| {
                          view! { <option value=m.map.clone()>{m.map.clone()}</option> }
                        })
                        .collect_view()
                    })
                }
              </Await>
            </datalist>
            <label for="map" class="placeholder">
              "Map"
            </label>
          </div>
          <Freeform
            name="yt_id"
            indicator="Video ID"
            error="Invalid video ID."
            {..}
            required=""
            minlength="11"
            maxlength="11"
            pattern="[a-zA-Z0-9_\\-]*"
          />
          <div class="text">"∗ Fields are optional."</div>
          <Button slot kind=ButtonType::Primary text="Submit" />
        </Dialogue>
      </section>
    }
}
