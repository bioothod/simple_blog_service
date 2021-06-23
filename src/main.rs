use rocket::fs::FileServer;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FlashMessage, FromRequest, Request};
use rocket::response::{Redirect, Flash, status};
use rocket::serde::Serialize;
use rocket::State;

use rocket_dyn_templates::Template;

use std::collections::HashMap;
use structopt::StructOpt;

#[macro_use] extern crate rocket;

mod user;
use user::{User, UserCtl};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct TemplateContext<'r> {
    title: &'r str,
    name: Option<&'r str>,
    user_id: Option<usize>,
    items: Vec<HashMap<String, String>>,
}

#[derive(FromForm)]
struct Login<'r> {
    username: &'r str,
    password: &'r str
}


#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<User, Self::Error> {
        let user_id: Option<usize> = request.cookies()
                                        .get_private("user_id")
                                        .and_then(|cookie| cookie.value().parse().ok());

        let user_id = match user_id {
            Some(x) => x,
            None => return request::Outcome::Forward(()),
        };

        request.rocket().state::<UserCtl>()
            .map(|ctl| {
                let x = ctl.read().unwrap();
                let user = x.meta.check_user_id(&user_id);
                user
            })
            .and_then(|x| x)
            .or_forward(())
    }
}

#[get("/")]
fn index(user: User, ctl: &State<UserCtl>) -> Template {
    let mut x = ctl.write().unwrap();
    let posts = x.read_latest(0, 10);

    let posts = match posts {
        Err(err_msg) => {
            println!("could not read posts: {}", err_msg);
            vec![]
        },
        Ok(x) => x,
    };
    drop(x);

    Template::render("index", &TemplateContext {
        title: "ioremap.net :: posts",
        name: Some(&user.username),
        user_id: Some(user.user_id),
        items: posts,
    })
}

#[get("/", rank = 2)]
fn no_auth_index() -> Redirect {
    Redirect::to(uri!(login_page))
}

#[get("/login")]
fn login(_user: User) -> Redirect {
    Redirect::to(uri!(index))
}

#[get("/login", rank = 2)]
fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
    println!("login_page: flash: {:?}", flash);

    let mut x = HashMap::new();
    if let Some(f) = flash {
        x.insert("kind", f.kind());
        x.insert("message", f.message());
        Template::render("login", &x)
    } else {
        x.insert("kind", "invalid kind");
        x.insert("message", "invalid message");
        Template::render("login", &x)
    }
}

#[post("/login", data = "<login>")]
fn post_login(jar: &CookieJar<'_>, login: Form<Login<'_>>, ctl: &State<UserCtl>) -> Result<Redirect, Flash<Redirect>> {
    let x = ctl.read().unwrap();
    match x.meta.check_password(login.username, login.password) {
        Ok(user) => {
            jar.add_private(Cookie::new("user_id", user.user_id.to_string()));
            Ok(Redirect::to(uri!(index)))
        },
        Err(err_msg) => {
            Err(Flash::error(Redirect::to(uri!(login_page)), err_msg))
        },
    }
}

#[post("/logout")]
fn logout(_user: User, jar: &CookieJar<'_>) -> Flash<Redirect> {
    jar.remove_private(Cookie::named("user_id"));
    Flash::success(Redirect::to(uri!(login_page)), "Successfully logged out.")
}

#[catch(404)]
fn not_found(req: &Request<'_>) -> Template {
    let mut map = HashMap::new();
    map.insert("path", req.uri().path().raw());
    Template::render("error/404", &map)
}

#[catch(default)]
fn default_catcher(status: Status, req: &Request<'_>) -> status::Custom<String> {
    let msg = format!("{} ({})", status, req.uri());
    status::Custom(status, msg)
}

#[derive(StructOpt)]
struct Cli {
    #[structopt(long="meta_path", parse(from_os_str))]
    meta_path: std::path::PathBuf,

    #[structopt(long="db_path", parse(from_os_str))]
    db_path: std::path::PathBuf,
}

#[rocket::main]
async fn main() {
    let args = Cli::from_args();

    let cfg = user::config::Config{
        db_path: &args.db_path,
        meta_path: &args.meta_path,
    };

    rocket::build()
        .attach(Template::fairing())
        .register("/", catchers![default_catcher, not_found])
        .mount("/static", FileServer::from("../static"))
        .mount("/", routes![index, no_auth_index, login, login_page, post_login, logout])
        .manage(user::init(&cfg))
        .launch()
        .await
        .unwrap();
}
