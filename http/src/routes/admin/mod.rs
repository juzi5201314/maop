utils::pub_mods!(
    login
);


use rocket::Route;

pub fn routes() -> impl Into<Vec<Route>>  {
    rocket::routes![
        login::login_page,
    ]
}

