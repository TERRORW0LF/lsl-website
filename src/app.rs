use std::{cmp::Ordering, collections::HashMap};

use crate::{
    api::{get_runs_category, MapRuns, PartialRun},
    auth::{get_user, Login, Logout, Register, User},
    error_template::{AppError, ErrorTemplate},
};
use leptos::{either::*, prelude::*};
use leptos_meta::MetaTags;
use leptos_meta::*;
use leptos_router::{
    components::{Form, Outlet, ParentRoute, ProtectedRoute, Redirect, Route, Router, Routes, A},
    hooks::{use_params_map, use_query_map},
    path, MatchNestedRoutes,
};
use rust_decimal::Decimal;
use wasm_bindgen::{prelude::Closure, JsCast};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" dir="ltr">
            <head>
                <Link rel="preconnect" href="https://fonts.googleapis.com" />
                <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="" />
                <Link
                    href="https://fonts.googleapis.com/css2?family=Roboto+Flex:opsz,wght@8..144,100..1000&display=swap"
                    rel="stylesheet"
                />
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let login = ServerAction::<Login>::new();
    let logout = ServerAction::<Logout>::new();
    let register = ServerAction::<Register>::new();
    let user = Resource::new(
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

    Effect::new(|_| {
        document()
            .document_element()
            .unwrap()
            .set_class_name("dark")
    });

    Effect::new(|_| {
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
        <Stylesheet id="leptos" href="/pkg/lsl-website.css" />
        // sets the document title
        <Title text="Lucio Surf League" />
        // content for this welcome page
        <Router>
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
                                            EitherOf3::A(
                                                view! {
                                                    <A href="/login">"Login"</A>
                                                    <span>{format!("Login error: {}", e)}</span>
                                                },
                                            )
                                        }
                                        Ok(None) => {
                                            EitherOf3::B(view! { <A href="/login">"Login"</A> })
                                        }
                                        Ok(Some(user)) => {
                                            EitherOf3::C(
                                                view! {
                                                    <A href="/settings">
                                                        <img src=format!("/cdn/users/{}.jpg", user.id) />
                                                    </A>
                                                },
                                            )
                                        }
                                    })
                            }}
                        </Transition>
                    </div>
                </nav>
            </header>
            <main>
                <AppRouter register=register login=login logout=logout user=user />
            </main>
        </Router>
    }
}

#[component(transparent)]
fn AppRouter(
    register: ServerAction<Register>,
    login: ServerAction<Login>,
    logout: ServerAction<Logout>,
    user: Resource<Result<Option<User>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <Routes fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors /> }
        }>
            <Route path=path!("") view=|| view! { <Redirect path="home" /> } />
            <Route path=path!("home") view=HomePage />
            <Route
                path=path!("discord")
                view=|| {
                    Effect::new(|_| {
                        window().location().set_href("https://discord.com/invite/G9QBCDY")
                    });
                    ""
                }
            />
            <Route path=path!("register") view=move || view! { <Register action=register /> } />
            <Route path=path!("login") view=move || view! { <Login action=login /> } />
            <ProtectedRoute
                path=path!("settings")
                condition=move || user.get().map(|n| n.map_or(false, |u| u.map_or(false, |_| true)))
                redirect_path=|| "/login"
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
fn LeaderboardRouter() -> impl MatchNestedRoutes + Clone {
    let (patch, set_patch) = signal(String::new());
    let (layout, set_layout) = signal(String::new());
    let (category, set_category) = signal(String::new());

    view! {
        <ParentRoute
            path=path!("leaderboard")
            view=move || view! { <Section patch=patch layout=layout category=category /> }
        >
            <Route
                path=path!(":patch/:layout/:category")
                view=move || {
                    view! {
                        <Leaderboard patch=set_patch layout=set_layout category=set_category />
                    }
                }
            />
            <Route path=path!("") view=|| view! { <Redirect path="2.00/1/standard" /> } />
            <Route
                path=path!(":patch")
                view=|| {
                    view! { <Redirect path="1/standard" /> }
                }
            />
            <Route
                path=path!(":patch/:layout")
                view=|| {
                    view! { <Redirect path="standard" /> }
                }
            />
        </ParentRoute>
    }
    .into_inner()
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <Title text="Home" />
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

    let query = use_query_map();

    view! {
        <Title text="Leaderboard" />
        <section id="leaderboard">
            <details>
                <summary>
                    <span role="term" aria-details="filters">
                        ""
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
                                            + &query.get().to_query_string()
                                    }>
                                        <span class="text">"Layout " {l}</span>
                                    </A>
                                }
                            }
                        />
                    </div>
                    <div class="right-row-nav">
                        <A href=move || {
                            patch.get() + "/" + &layout.get() + "/standard"
                                + &query.get().to_query_string()
                        }>
                            <span class="text">"Standard"</span>
                        </A>
                        <A href=move || {
                            patch.get() + "/" + &layout.get() + "/gravspeed"
                                + &query.get().to_query_string()
                        }>
                            <span class="text">"Gravspeed"</span>
                        </A>
                    </div>
                </nav>
            </header>
            <div role="definition" id="filters" class="content">
                <Form
                    method="GET"
                    action=move || patch.get() + "/" + &layout.get() + "/" + &category.get()
                >
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
                            <label for="date">"Date"</label>
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
                            <input type="radio" name="filter" value="verified" id="verified" />
                            <label for="verified">"Verified"</label>
                        </div>
                    </div>
                    <input type="submit" class="button" value="Apply" />
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
    Effect::new(move |_| {
        params.with(|params| {
            patch.set(params.get("patch").unwrap());
            layout.set(params.get("layout").unwrap());
            category.set(params.get("category").unwrap());
        })
    });
    let selection = Memo::new(move |_| {
        params.with(|params| {
            (
                params.get("patch").unwrap(),
                params.get("layout").unwrap(),
                params.get("category").unwrap(),
            )
        })
    });
    let maps = Resource::new(selection, |mut s| {
        get_runs_category(s.0, s.1, format!("{}{}", s.2.remove(0).to_uppercase(), s.2))
    });

    view! {
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            {move || {
                maps.get()
                    .map(|data| match data {
                        Err(e) => Either::Right(view! { <p>{e.to_string()}</p> }),
                        Ok(maps) => {
                            Either::Left(
                                view! {
                                    <div id="lb">
                                        {maps
                                            .into_iter()
                                            .map(|map| {
                                                view! { <LeaderboardEntry map=map /> }
                                            })
                                            .collect_view()}
                                    </div>
                                },
                            )
                        }
                    })
            }}
        </Transition>
    }
}

#[component]
pub fn LeaderboardEntry(map: MapRuns) -> impl IntoView {
    fn filter<'a>(
        r: &'a PartialRun,
        f: Option<String>,
        t: &'a mut Decimal,
        ts: &'a mut HashMap<i64, Decimal>,
    ) -> bool {
        match f {
            Some(f) => match f.as_str() {
                "verified" => r.verified,
                "was_pb" => {
                    if &r.time < ts.get(&r.user_id).unwrap_or(&Decimal::new(999999, 3)) {
                        ts.insert(r.user_id, r.time);
                        true
                    } else {
                        false
                    }
                }
                "is_pb" => r.is_pb,
                "was_wr" => {
                    if r.time < *t {
                        *t = r.time;
                        true
                    } else {
                        false
                    }
                }
                "is_wr" => r.is_wr,
                _ => true,
            },
            None => r.is_pb,
        }
    }
    let sort = |s: Option<String>| match s {
        Some(s) => match s.as_str() {
            "date" => move |r1: &(usize, PartialRun), r2: &(usize, PartialRun)| -> Ordering {
                if r1.1.created_at > r2.1.created_at {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            },
            _ => move |r1: &(usize, PartialRun), r2: &(usize, PartialRun)| -> Ordering {
                if r1.1.time == r2.1.time {
                    if r1.1.created_at < r2.1.created_at {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                } else {
                    if r1.1.time < r2.1.time {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
            },
        },
        None => move |r1: &(usize, PartialRun), r2: &(usize, PartialRun)| -> Ordering {
            if r1.1.time < r2.1.time {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        },
    };

    let filter_key = || use_query_map().with(|q| q.get("filter").map(|s| s.to_owned()));
    let sort_key = || use_query_map().with(|q| q.get("sort").map(|s| s.to_owned()));
    let runs = move || {
        let mut old_time = Decimal::new(999999, 3);
        let mut old_times = HashMap::<i64, Decimal>::new();
        let r = map.runs.clone();
        let mut runs: Vec<(usize, PartialRun)> = r
            .into_iter()
            .filter(|r| filter(r, filter_key(), &mut old_time, &mut old_times))
            .enumerate()
            .collect();
        runs.sort_unstable_by(sort(sort_key()));
        runs
    };
    let runs2 = runs.clone();
    let (run, set_run) = signal::<Option<PartialRun>>(None);

    view! {
        <div class="lb_entry">
            <div class="header">
                <h2>{map.map.clone()}</h2>
                {move || match runs().get(0) {
                    Some((_, r)) => {
                        set_run(Some(r.clone()));
                        Either::Left(
                            view! {
                                <h5>{r.username.clone()}</h5>
                                <h5>{r.time.to_string()} " seconds"</h5>
                            },
                        )
                    }
                    None => Either::Right(view! {}),
                }}
            </div>
            <div class="content">
                <div class="video">
                    {move || match run.get() {
                        Some(r) => {
                            Either::Left(
                                view! {
                                    <div class="buttons">
                                        <a class="play-wrapper" href=r.proof.clone()>
                                            <div></div>
                                        </a>
                                        <br />
                                        <a class="external" href=r.proof target="_blank">
                                            "Open in new Tab"
                                        </a>
                                    </div>
                                },
                            )
                        }
                        None => Either::Right(view! {}),
                    }} <div class="toner">
                        <img
                            src=format!("/cdn/maps/{}.jpg", map.map)
                            alt=format!("Picture of {}", map.map)
                        />
                    </div>
                </div>
                <div class="lb_entry_ranks">
                    <Show
                        when=move || run.with(|r| r.is_some())
                        fallback=|| view! { <span class="no-data">"No Runs Found"</span> }
                    >
                        <For
                            each=runs2.clone()
                            key=|r| r.1.id
                            children=move |(i, r)| {
                                view! {
                                    <div class="lb_entry_rank">
                                        <span class="rank">
                                            {move || match sort_key() {
                                                Some(k) => {
                                                    if k == "time" {
                                                        "#".to_string() + &(i + 1).to_string()
                                                    } else {
                                                        format!("{}", r.created_at.format("%d/%m/%y"))
                                                    }
                                                }
                                                None => "#".to_string() + &(i + 1).to_string(),
                                            }}
                                        </span>
                                        <span class="name">
                                            <A href=format!("/user/{}", r.user_id)>{r.username}</A>
                                        </span>
                                        <span class="time">{r.time.to_string()} " s"</span>
                                    </div>
                                }
                            }
                        />
                    </Show>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Login(action: ServerAction<Login>) -> impl IntoView {
    view! {
        <Title text="Log In" />
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
                    <A href="/register" attr:class="link">
                        "Create account"
                    </A>
                    <input type="submit" class="button" value="Sign In" />
                </div>
            </ActionForm>
        </section>
    }
}

#[component]
pub fn Register(action: ServerAction<Register>) -> impl IntoView {
    view! {
        <Title text="Sign Up" />
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
pub fn Logout(action: ServerAction<Logout>) -> impl IntoView {
    view! {
        <Title text="Log Out" />
        <div id="loginbox">
            <ActionForm action=action>
                <button type="submit" class="button">
                    "Log Out"
                </button>
            </ActionForm>
        </div>
    }
}
