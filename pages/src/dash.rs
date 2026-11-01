use components::{Button, ButtonType, Dialogue, Freeform, Modal, MultiLine, Redirect};
use leptos::{either::Either, html::Input, prelude::*};
use leptos_meta::Title;
use leptos_router::components::{A, Outlet};
use server::auth::{DiscordAdd, DiscordDelete, Logout, UpdateBio, UpdateCreds, discord_list};
use types::{
    api::{ApiError, User},
    leptos::{UpdatePfpAction, UserResource},
};
use util::escape_regex;
use wasm_bindgen::JsCast;
use web_sys::{Event, FormData, HtmlFormElement, SubmitEvent};

#[component]
pub fn Dashboard() -> impl IntoView {
    let user = expect_context::<UserResource>();
    let logout = expect_context::<ServerAction<Logout>>();

    view! {
      <Title text="Dashboard" />
      <section id="dashboard">
        <Outlet />
        <Suspense fallback=|| view! { <h1>"Loading..."</h1> }>
          <h1>"Dashboard"</h1>
          <div class="section">
            <h2>"Profile"</h2>
            <div class="row">
              <A href="avatar">
                <img
                  class="pfp"
                  src=move || {
                    format!(
                      "/cdn/users/{}.jpg",
                      user
                        .read()
                        .as_ref()
                        .map(|res| { res.as_ref().unwrap_or(&User::default()).pfp.clone() })
                        .unwrap_or("default".into()),
                    )
                  }
                />
              </A>
              <A attr:class="button primary" href="avatar">
                "Change Avatar"
              </A>
            </div>
            <div class="spacer-2"></div>
            <div class="row no-wrap">
              <div class="narrow">
                <h3>"USERNAME"</h3>
                <h4>{move || { user.and_then(|user| user.username.clone()) }}</h4>
              </div>
              <A attr:class="button secondary" href="username">
                "Edit"
              </A>
            </div>
            <div class="spacer-2"></div>
            <div class="row no-wrap">
              <div class="narrow">
                <h3>"PASSWORD"</h3>
                <h4>"********"</h4>
              </div>
              <A attr:class="button secondary" href="password">
                "Edit"
              </A>
            </div>
            <div class="spacer-2"></div>
            <div class="row no-wrap">
              <div class="narrow">
                <h3>"ABOUT ME"</h3>
                <p>{move || { user.and_then(|user| { user.bio.clone().unwrap_or("No about me set.".into()) }) }}</p>
              </div>
              <A attr:class="button secondary" href="bio">
                "Edit"
              </A>
            </div>
          </div>
          <div class="section">
            <h2>"Account"</h2>
            <h3>"CONNECTIONS"</h3>
            <div>
              <h4>"Discord"</h4>
              <p>
                "Manage your connected Discord profiles or add a new one. "
                "This allows you to easily submit and manage your runs from within Discord."
              </p>

              <A attr:class="button primary" href="discord">
                "Manage"
              </A>
            </div>
            <h3>"MANAGEMENT"</h3>
            <div>
              <h4>"Log Out"</h4>
              <p>"Logging out of your account will only affect this device."</p>
              <ActionForm action=logout>
                <input type="submit" class="button secondary" value="Log Out" />
              </ActionForm>
            </div>
            <div>
              <h4>"Account Removal"</h4>
              <p>
                "Disabling your account will anonymize your account and prevent "
                "you from logging in. This action will not delete runs from the leaderboard."
              </p>
              <button class="button danger">"Disable Account"</button>
            </div>
          </div>
          <div class="section">
            <h2>"Submissions"</h2>
            <div>
              <h4>"Edit Submissions"</h4>
              <p>
                "Manage your submissions to the leaderboards. "
                "This will redirect you to the submission management page."
              </p>
              <A attr:class="button primary" href="../manage">
                "Edit"
              </A>
            </div>
            <div>
              <h4>"Claim Submissions"</h4>
              <p>
                "Claim submissions made under the old Google Sheets method "
                "to your account. Requires moderator approval."
              </p>
              <button class="button secondary">"Claim"</button>
            </div>
          </div>
        </Suspense>
      </section>
    }
}

#[component]
pub fn Username() -> impl IntoView {
    let action = expect_context::<ServerAction<UpdateCreds>>();
    let result = Signal::derive(move || action.value().get());
    view! {
      <Modal>
        <h1>"Edit Username"</h1>
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
          <Redirect slot redirect="user/@me/dashboard" />
          {[
            view! {
              <Freeform
                name="username"
                indicator="Username"
                error="Username must adhere to [a-zA-Z0-9_-.]{2,32}."
                {..}
                required=""
                minlength="2"
                maxlength="32"
                pattern="[a-zA-Z0-9_\\-\\.]*"
              />
            },
          ]}
          <Button slot kind=ButtonType::Link("../".into()) text="Cancel" />
          <Button slot kind=ButtonType::Primary text="Save" />
        </Dialogue>
      </Modal>
    }
}

#[component]
pub fn Password() -> impl IntoView {
    let action = expect_context::<ServerAction<UpdateCreds>>();
    let result = Signal::derive(move || action.value().get());
    let (password_new, set_password_new) = signal(String::new());
    view! {
      <Modal>
        <h1>"Edit Password"</h1>
        <ErrorBoundary fallback=|e| {
          view! {
            <span class="error">
              {move || {
                let e = e.get().into_iter().next().unwrap().1;
                if e.is::<ApiError>() {
                  let e = e.downcast_ref::<ApiError>().unwrap();
                  match e {
                    ApiError::InvalidCredentials => "🛈 Incorrect password",
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
          <Redirect slot redirect="user/@me/dashboard" />
          <Freeform name="password[old]" indicator="Current Password" {..} type="password" required="" />
          <Freeform
            name="password[new]"
            indicator="New Password"
            error="Password must be between 8 and 256 characters long."
            on:input=move |e| set_password_new(event_target_value::<Event>(&e))
            {..}
            type="password"
            required=""
            minlength="8"
            maxlength="256"
          />
          <Freeform
            name="password-repeat"
            indicator="Repeat Password"
            error="Passwords must match."
            {..}
            type="password"
            required=""
            pattern=move || escape_regex(&password_new.read())
          />
          <Button slot kind=ButtonType::Link("../".into()) text="Cancel" />
          <Button slot kind=ButtonType::Primary text="Save" />
        </Dialogue>
      </Modal>
    }
}

#[component]
pub fn Bio() -> impl IntoView {
    let action = expect_context::<ServerAction<UpdateBio>>();
    let user = expect_context::<UserResource>();
    let result = Signal::derive(move || action.value().get());
    view! {
      <Modal>
        <h1>"Edit About Me"</h1>
        <ErrorBoundary fallback=|e| {
          view! {
            <span class="error">
              {move || {
                let e = e.get().into_iter().next().unwrap().1;
                if e.is::<ApiError>() {
                  let e = e.downcast_ref::<ApiError>().unwrap();
                  match e {
                    ApiError::InvalidCredentials => "🛈 Incorrect password",
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
          <Redirect slot redirect="user/@me/dashboard" />
          {[
            view! {
              <MultiLine
                name="bio"
                indicator="About Me"
                error="About me must be under 512 characters."
                attr:maxlength="512"
              >
                <Suspense fallback=move || {
                  "Loading..."
                }>{move || user.and_then(|u| u.bio.clone().unwrap_or(String::new()))}</Suspense>
              </MultiLine>
            },
          ]}
          <Button slot kind=ButtonType::Link("../".into()) text="Cancel" />
          <Button slot kind=ButtonType::Primary text="Save" />
        </Dialogue>
      </Modal>
    }
}

#[component]
pub fn Avatar() -> impl IntoView {
    let action = expect_context::<UpdatePfpAction>();
    let result = Signal::derive(move || action.value().get());
    let input_ref = NodeRef::<Input>::new();
    view! {
      <A attr:class="toner" href="../">
        <div />
      </A>
      <section id="box">
        <h1>"Change Avatar"</h1>
        <ErrorBoundary fallback=|e| {
          view! {
            <span class="error">
              {move || {
                let e = e.get().into_iter().next().unwrap().1;
                if e.is::<ServerFnError<ApiError>>() {
                  let e = e.downcast_ref::<ServerFnError<ApiError>>().unwrap();
                  match e {
                    ServerFnError::Args(_) => "🛈 File must be jpg and under 4 MB",
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
        <form on:submit=move |ev: SubmitEvent| {
          ev.prevent_default();
          let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
          let form_data = FormData::new_with_form(&target).unwrap();
          action.dispatch_local(form_data);
        }>
          <div class="input-box">
            <input node_ref=input_ref type="file" name="avatar" id="avatar" required accept=".jpg" />
            <label
              for="avatar"
              class="file"
              on:dragover=move |ev| ev.prevent_default()
              on:drop=move |ev| {
                ev.prevent_default();
                if let Some(dt) = ev.data_transfer() {
                  if let Some(input) = input_ref.get_untracked() {
                    input.set_files(dt.files().as_ref());
                  }
                }
              }
            >
              <h6>"Drag and drop file here or click to open file selector"</h6>
              <p>"Accepts .jpg files under 4 MB"</p>
            </label>
            <label for="avatar" class="error">
              "File must be jpg."
            </label>
          </div>
          <div class="row">
            <A attr:class="button secondary" href="../">
              "Cancel"
            </A>
            <input type="submit" class="button primary" value="Save" />
          </div>
        </form>
      </section>
    }
}

#[component]
pub fn DiscordList() -> impl IntoView {
    let discord_del = ServerAction::<DiscordDelete>::new();
    let discord_add = ServerAction::<DiscordAdd>::new();
    let discord_list =
        Resource::new(move || (discord_del.version().get(), discord_add.version().get()), |_| discord_list());
    view! {
      <A attr:class="toner" href="../">
        <div />
      </A>
      <section id="box">
        <h1>"Discord"</h1>
        <Transition fallback=move || {
          view! { <p>"Loading..."</p> }
        }>
          <ErrorBoundary fallback=|_| {
            view! { <span class="error">"🛈 Something went wrong. Try again"</span> }
          }>
            {move || {
              discord_list
                .and_then(|conns| {
                  view! {
                    {conns
                      .clone()
                      .into_iter()
                      .map(|con| {
                        view! {
                          <div class="discord row">
                            <div class="narrow">
                              <h4>{con.name}</h4>
                              <p>{con.snowflake.clone()}</p>
                            </div>
                            <ActionForm action=discord_del>
                              <input hidden type="text" name="snowflake" value=con.snowflake.clone() />
                              <input type="submit" class="button danger" value="Remove" />
                            </ActionForm>
                          </div>
                        }
                      })
                      .collect_view()}
                    {if conns.len() < 5 {
                      Either::Left(
                        view! {
                          <ActionForm action=discord_add>
                            <input type="submit" class="discord-add" value="Add Account" />
                          </ActionForm>
                        },
                      )
                    } else {
                      Either::Right(())
                    }}
                  }
                })
            }}
          </ErrorBoundary>
        </Transition>
        <div class="row">
          <A attr:class="button secondary" href="../">
            "Cancel"
          </A>
          <A attr:class="button primary" href="../">
            "Save"
          </A>
        </div>
      </section>
    }
}
