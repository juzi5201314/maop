utils::pub_mods!(
    index
);

use rocket::Route;

pub fn routes() -> impl Into<Vec<Route>>  {
    rocket::routes![
        index::index_page,
    ]
}

pub fn api_routes() -> impl Into<Vec<Route>>  {
    rocket::routes![
        index::index_api
    ]
}

