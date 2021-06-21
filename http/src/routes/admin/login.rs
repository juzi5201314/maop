use rocket::{post, get, State};

#[get("/login")]
pub async fn login_page() {

}

/*#[post("/login")]
pub async fn login<'a>(resp_type: Option<RespType<'a>>, req: Request<'a>, db: &'a State<Arc<Database>>) -> crate::Result<'a> {
    let data = IndexData::new(db).await?;

    let base = req.route.map(|r| r.uri.base());
    match base {
        Some("/api") => index_api(resp_type, data).await,
        _ => {
            Ok(Response::new()
                .text("hello world"))
        }
    }
}
*/
