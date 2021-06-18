use rocket::fs::FileServer;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FlashMessage, FromRequest, Request};
use rocket::response::{Redirect, Flash, status};
use rocket::serde::Serialize;

use rocket_dyn_templates::Template;

use std::collections::HashMap;

#[macro_use] extern crate rocket;

mod user;
use user::{User, UserCtl};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct TemplateContext<'r> {
    title: &'r str,
    name: Option<&'r str>,
    user_id: Option<usize>,
    items: Vec<&'r str>
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

        println!("from_request: 0: user_id: {:?}", user_id);
        let user_id = match user_id {
            Some(x) => x,
            None => return request::Outcome::Forward(()),
        };

        let outcome = request.rocket().state::<UserCtl>()
            .map(|ctl| {
                let x = ctl.lock().unwrap();
                let user = x.get(user_id).unwrap();
                user
            })
            .or_forward(());
        outcome
    }
}

#[get("/")]
fn index(user: User) -> Template {
    Template::render("index", &TemplateContext {
        title: "Hello",
        name: Some(&user.username),
        user_id: Some(user.user_id),
        items: vec!["One", "Two", "Three"],
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
    let mut x = HashMap::new();
    x.insert("message", "invalid login");
    x.insert("kind", "some kind");
    Template::render("login", &x)
}

#[post("/login", data = "<login>")]
fn post_login(jar: &CookieJar<'_>, login: Form<Login<'_>>) -> Result<Redirect, Flash<Redirect>> {
    if login.username == "zbr" && login.password == "password" {
        jar.add_private(Cookie::new("user_id", 1.to_string()));
        Ok(Redirect::to(uri!(index)))
    } else {
        Err(Flash::error(Redirect::to(uri!(login_page)), "Invalid username/password."))
    }
}

#[post("/logout")]
fn logout(jar: &CookieJar<'_>) -> Flash<Redirect> {
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

#[rocket::main]
async fn main() {
    let cfg = user::config::Config{
        path: "qwe".to_owned(),
    };
    rocket::build()
        .attach(Template::fairing())
        .register("/", catchers![default_catcher, not_found])
        .mount("/static", FileServer::from("../static/"))
        .mount("/", routes![index, no_auth_index, login, login_page, post_login, logout])
        .manage(user::stage(cfg))
        .launch()
        .await
        .unwrap();
}
