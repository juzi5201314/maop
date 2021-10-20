use rocket::Route;

pub fn routes() -> impl Into<Vec<Route>>  {
    rocket::routes![
        crate::index::index::index_api
    ]
}

