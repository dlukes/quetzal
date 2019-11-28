#![feature(proc_macro_hygiene, decl_macro)]

use std::path::PathBuf;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use rocket::response::content::{Html, JavaScript};
use rocket_contrib::json::JsonValue;
// use rocket_contrib::serve::StaticFiles;

// _path below currently doesn't capture empty paths, so we need to treat
// index specially and redirect from it manually; cf.
// https://github.com/SergioBenitez/Rocket/issues/985
#[get("/", format = "text/html")]
fn index() -> Html<String> {
    frontend_ui(None)
}

#[get("/<_path..>", format = "text/html")]
fn frontend_ui(_path: Option<PathBuf>) -> Html<String> {
    let main_html = include_str!("../../../front/src/main.html");
    Html(main_html.replace("MAIN_JS", "/main.js"))
}

// implement ?v=hash cache bypass
#[get("/main.js", format = "application/javascript")]
fn main_js() -> JavaScript<&'static str> {
    JavaScript(include_str!("../../../front/target/main.js"))
}

// this is more correct...
// #[get("/documents", format = "application/json")]
// ... but this makes it easier to test the API by sending requests from the
// browser:
#[get("/documents")]
fn documents() -> JsonValue {
    json!({
        "data": [
            { "id": "12A001N" },
            { "id": "12A002N" }
        ],
        "errors": []
    })
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, frontend_ui, main_js])
        .mount("/api", routes![documents])
        .launch();
}
