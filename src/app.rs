use crate::{
    api::{get_runs_category, MapRuns},
    auth::{get_user, Login, Logout, Register},
    error_template::{AppError, ErrorTemplate},
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use wasm_bindgen::{prelude::Closure, JsCast};

#[component]
pub fn App() -> impl IntoView {
    let login = create_server_action::<Login>();
    let logout = create_server_action::<Logout>();
    let register = create_server_action::<Register>();
    let user = create_resource(
        move || {
            (
                login.version().get(),
                register.version().get(),
                logout.version().get(),
            )
        },
        move |_| get_user(),
    );

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    create_effect(|_| {
        document()
            .document_element()
            .unwrap()
            .set_class_name("dark")
    });

    create_effect(|_| {
        let a = Closure::<dyn FnMut()>::new(|| {
            let _ = document()
                .body()
                .unwrap()
                .style()
                .set_property("--scroll", &(window().page_y_offset().unwrap()).to_string());
        });
        let _ = window().add_event_listener_with_callback_and_bool(
            "scroll",
            a.as_ref().unchecked_ref(),
            false,
        );
        a.forget();
    });

    view! {
        <Html lang="en-US" dir="ltr" />
        <Stylesheet id="leptos" href="/pkg/lsl-website.css" />
        <Link rel="preconnect" href="https://fonts.googleapis.com" />
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="" />
        <Link
            href="https://fonts.googleapis.com/css2?family=Roboto+Flex:opsz,wght@8..144,100..1000&display=swap"
            rel="stylesheet"
        />

        // sets the document title
        <Title text="Welcome to Leptos" />

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors /> }.into_view()
        }>
            <header>
                <nav class="split-row-nav">
                    <div class="left-row-nav">
                        <A href="/home">"Home"</A>
                        <A href="/leaderboard/2.00/1/Standard">"Leaderboard"</A>
                        <A href="/ranking">"Ranking"</A>
                        <a href="https://discord.com/invite/G9QBCDY" rel="external">
                            "Discord"
                        </a>
                    </div>
                    <div class="right-row-nav">
                        <Transition fallback=move || {
                            view! { <span>"Loading..."</span> }
                        }>
                            {move || {
                                user.get()
                                    .map(|user| match user {
                                        Err(e) => {
                                            view! {
                                                <A href="/register">"Register"</A>
                                                <A href="/login">"Login"</A>
                                                <span>{format!("Login error: {}", e)}</span>
                                            }
                                                .into_view()
                                        }
                                        Ok(None) => {
                                            view! { <A href="/login">"Login"</A> }.into_view()
                                        }
                                        Ok(Some(user)) => {
                                            view! {
                                                <A href="/settings">"Settings"</A>
                                                <span>
                                                    {format!("Logged in as: {} ({})", user.username, user.id)}
                                                </span>
                                            }
                                                .into_view()
                                        }
                                    })
                            }}
                        </Transition>
                    </div>
                </nav>
            </header>
            <main>
                <AppRouter register=register login=login logout=logout />
            </main>
        </Router>
    }
}

#[component(transparent)]
fn AppRouter(
    register: Action<Register, Result<(), ServerFnError>>,
    login: Action<Login, Result<(), ServerFnError>>,
    logout: Action<Logout, Result<(), ServerFnError>>,
) -> impl IntoView {
    view! {
        <Routes>
            <Route path="" view=|| view! { <Redirect path="home" /> } />
            <Route path="home" view=HomePage />
            <Route
                path="discord"
                view=|| {
                    create_effect(|_| {
                        window().location().set_href("https://discord.com/invite/G9QBCDY")
                    });
                    ""
                }
            />
            <Route path="register" view=move || view! { <Register action=register /> } />
            <Route path="login" view=move || view! { <Login action=login /> } />
            <Route
                path="settings"
                view=move || {
                    view! {
                        <h1>"Settings"</h1>
                        <Logout action=logout />
                    }
                }
            />
            <LeaderboardRouter />
        </Routes>
    }
}

#[component(transparent)]
fn LeaderboardRouter() -> impl IntoView {
    let (patch, set_patch) = create_signal(String::new());
    let (layout, set_layout) = create_signal(String::new());
    let (category, set_category) = create_signal(String::new());

    view! {
        <Route
            path="leaderboard"
            view=move || view! { <Section patch=patch layout=layout category=category /> }
        >
            <Route
                path=":patch/:layout/:category"
                view=move || {
                    view! {
                        <Leaderboard patch=set_patch layout=set_layout category=set_category />
                    }
                }
            />
            <Route path="" view=|| view! { <Redirect path="2.00/1/Standard" /> } />
            <Route
                path=":patch"
                view=|| {
                    view! { <Redirect path="1/Standard" /> }
                }
            />
            <Route
                path=":patch/:layout"
                view=|| {
                    view! { <Redirect path="Standard" /> }
                }
            />
        </Route>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <section id="welcome">
            <h1>"Welcome to the Lucio Surf League!"</h1>
            <p>
                "Since its inception in 2019 "<b>"Lucio Surf"</b>
                " has been a staple in Overwatch's custom game browser. The "
                <b>"Lucio Surf League"</b>
                " has been the proud maintainer and arbitrator of Lucio Surf since the beginning. "
                "Updating the game mode and leaderboards, connecting the community, and holding tournaments, the Lucio Surf League "
                "is your place for anything related to Lucio Surf. We welcome you as another frog in our ranks."
            </p>
        </section>
        <section id="potd">
            <h2>"Player of the day"</h2>
            <div>
                <img src="" />
                <p>
                    "Test is an exceptional player combining superb routing with steady movement. This allows them to top "
                    "any leaderboard that don't involve boosts. While occasionaly dabbling in Gravspeed their main category "
                    "is Standard, prefering the challenge over the speed."
                </p>
            </div>
        </section>
    }
}

#[component]
pub fn Section(
    patch: ReadSignal<String>,
    layout: ReadSignal<String>,
    category: ReadSignal<String>,
) -> impl IntoView {
    let layouts = move || match patch.get().as_str() {
        "1.00" => vec![1, 2],
        "1.41" => vec![1, 2, 3, 4],
        "1.50" => vec![1, 2, 3, 4, 5],
        "2.00" => vec![1, 2, 3, 4, 5],
        _ => vec![],
    };

    view! {
        <section id="leaderboard">
            <details>
                <summary>
                    <span role="term" aria-details="filters">
                        "advanced"
                    </span>
                </summary>
            </details>
            <header id="lb_header">
                <nav class="split-row-nav">
                    <div class="left-row-nav">
                        <For
                            each=layouts
                            key=|l| l.to_owned()
                            children=move |l| {
                                view! {
                                    <A href=move || {
                                        patch.get() + "/" + &l.to_string() + "/" + &category.get()
                                    }>
                                        <span class="text">"Layout " {l}</span>
                                    </A>
                                }
                            }
                        />
                    </div>
                    <div class="right-row-nav">
                        <A href=move || patch.get() + "/" + &layout.get() + "/Standard">
                            <span class="text">"Standard"</span>
                        </A>
                        <A href=move || patch.get() + "/" + &layout.get() + "/Gravspeed">
                            <span class="text">"Gravspeed"</span>
                        </A>
                    </div>
                </nav>
            </header>
            <div role="definition" id="filters" class="content">
                <Form method="GET" action="">
                    <div class="group">
                        <h6>"Patch"</h6>
                        <div class="options">
                            <A href="1.00">"1.00"</A>
                            <A href="1.41">"1.41"</A>
                            <A href="1.50">"1.50"</A>
                            <A href="2.00">"Current"</A>
                        </div>
                    </div>
                    <div class="group">
                        <h6>"Sort by"</h6>
                        <div class="options">
                            <input type="radio" name="sort" value="time" id="time" />
                            <label for="time">"Time"</label>

                            <input type="radio" name="sort" value="date" id="date" />
                            <label for="time">"Date"</label>
                        </div>
                    </div>
                    <div class="group">
                        <h6>"Filter"</h6>
                        <div class="options">
                            <input type="radio" name="filter" value="none" id="none" />
                            <label for="none">"None"</label>
                            <input type="radio" name="filter" value="is_pb" id="is_pb" />
                            <label for="is_pb">"Is PB"</label>
                            <input type="radio" name="filter" value="is_wr" id="is_wr" />
                            <label for="is_wr">"Is WR"</label>
                            <input type="radio" name="filter" value="was_pb" id="was_pb" />
                            <label for="was_pb">"Was PB"</label>
                            <input type="radio" name="filter" value="was_wr" id="was_wr" />
                            <label for="was_wr">"Was WR"</label>
                        </div>
                    </div>
                    <input type="submit" class="button" value="apply" />
                </Form>
            </div>
            <Outlet />
        </section>
    }
}

#[component]
pub fn Leaderboard(
    patch: WriteSignal<String>,
    layout: WriteSignal<String>,
    category: WriteSignal<String>,
) -> impl IntoView {
    let params = use_params_map();
    create_effect(move |_| {
        params.with(|params| {
            patch.set(params.get("patch").cloned().unwrap());
            layout.set(params.get("layout").cloned().unwrap());
            category.set(params.get("category").cloned().unwrap());
        })
    });
    let selection = create_memo(move |_| {
        params.with(|params| {
            (
                params.get("patch").cloned().unwrap(),
                params.get("layout").cloned().unwrap(),
                params.get("category").cloned().unwrap(),
            )
        })
    });
    let maps = create_resource(selection, |s| get_runs_category(s.0, s.1, s.2));

    view! {
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || {
                maps.get()
                    .map(|data| match data {
                        Err(e) => view! { <p>{e.to_string()}</p> }.into_view(),
                        Ok(maps) => {
                            view! {
                                <div id="lb">
                                    {maps
                                        .into_iter()
                                        .map(|map| {
                                            view! { <LeaderboardEntry map=map /> }
                                        })
                                        .collect_view()}
                                </div>
                            }
                                .into_view()
                        }
                    })
            }}
        </Transition>
    }
}

#[component]
pub fn LeaderboardEntry(map: MapRuns) -> impl IntoView {
    view! {
        <div class="lb_entry">
            <div class="header">
                <h2>{map.map.clone()}</h2>
                {match map.runs.get(0) {
                    Some(r) => {
                        view! {
                            <h5>{r.username.clone()}</h5>
                            <h5>{r.time.to_string()} " seconds"</h5>
                        }
                            .into_view()
                    }
                    None => {}.into_view(),
                }}
            </div>
            <div class="content">
                <img
                    src=format!("/cdn/maps/{}.jpg", map.map)
                    alt=format!("Picture of {}", map.map)
                />
                <div class="lb_entry_ranks">
                    {map
                        .runs
                        .into_iter()
                        .filter(|r| r.is_pb && r.verified)
                        .enumerate()
                        .map(|(i, r)| {
                            view! {
                                <div class="lb_entry_rank">
                                    <span>"#"{i + 1}</span>
                                    <span>{r.username}</span>
                                    <span>{r.time.to_string()} " s"</span>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Login(action: Action<Login, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <section id="box">
            <h1>"Sign In"</h1>
            <ActionForm action=action>
                <div class="input-box">
                    <input
                        type="text"
                        name="username"
                        id="username"
                        required
                        value=""
                        onkeyup="this.setAttribute('value', this.value);"
                    />
                    <label for="username" class="placeholder">
                        "Username"
                    </label>
                </div>
                <div class="input-box">
                    <input
                        type="password"
                        name="password"
                        id="password"
                        required
                        value=""
                        onkeyup="this.setAttribute('value', this.value);"
                    />
                    <label for="password" class="placeholder">
                        "Password"
                    </label>
                </div>
                <div class="remember">
                    <input type="checkbox" name="remember" id="remember" />
                    <label for="remember">"Remember me?"</label>
                </div>
                <div class="line">
                    <A href="/register" class="link">
                        "Create account"
                    </A>
                    <input type="submit" class="button" value="Sign In" />
                </div>
            </ActionForm>
        </section>
    }
}

#[component]
pub fn Register(action: Action<Register, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <section id="box">
            <h1>"Sign Up"</h1>
            <ActionForm action=action>
                <div class="input-box">
                    <input
                        type="text"
                        maxlength="32"
                        name="username"
                        id="username"
                        required
                        minlength="2"
                        maxlength="32"
                        pattern="[a-zA-Z_\\-\\.]*"
                        value=""
                        onkeyup="this.setAttribute('value', this.value);"
                    />
                    <label for="username" class="placeholder">
                        "Username"
                    </label>
                    <label for="username" class="error">
                        "Username must adhere to [a-zA-Z_-.]{2,32}."
                    </label>
                </div>
                <div class="input-box">
                    <input
                        type="password"
                        name="password"
                        id="password"
                        required
                        minlength="8"
                        maxlength="256"
                        value=""
                        onkeyup="this.setAttribute('value', this.value);"
                    />
                    <label for="password" class="placeholder">
                        "Password"
                    </label>
                    <label for="password" class="error">
                        "Password must be between 8 and 256 characters long."
                    </label>
                </div>
                <div class="input-box">
                    <input
                        type="password"
                        name="password"
                        id="password-repeat"
                        required
                        value=""
                        onkeyup="this.setAttribute('value', this.value);"
                    />
                    <label for="password-repeat" class="placeholder">
                        "Repeat password"
                    </label>
                </div>
                <div class="remember">
                    <input type="checkbox" name="remember" id="remember" />
                    <label for="remember">"Remember me?"</label>
                </div>
                <input type="submit" class="button" value="Sign Up" />
            </ActionForm>
        </section>
    }
}

#[component]
pub fn Logout(action: Action<Logout, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <div id="loginbox">
            <ActionForm action=action>
                <button type="submit" class="button">
                    "Log Out"
                </button>
            </ActionForm>
        </div>
    }
}
