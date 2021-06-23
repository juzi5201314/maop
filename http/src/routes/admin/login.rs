use std::fs::{File, OpenOptions, read_to_string};
use std::io::Read;
use std::io::Write;
use std::ops::Deref;

use argon2::{PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand_core::OsRng;
use rocket::{get, post, State};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;

use crate::api_format::{Api, RespType};
use crate::request::Request;
use crate::response::Response;

#[get("/login")]
pub async fn login_page(req: Request<'_>) -> crate::Result<'_> {
    todo!()
}

#[post("/login", data = "<req_data>")]
pub async fn login<'a>(
    _api: Api,
    resp_t: RespType<'_>,
    req_data: Json<LoginReqData>,
) -> crate::Result<'a> {
    if let Some(password_hash) = get_password_from_local()? {
        let argon2 = new_argon2()?;
        let is_ok = argon2
            .verify_password(
                req_data.password.as_bytes(),
                &argon2::PasswordHash::new(&password_hash)
                    .map_err(|e| anyhow::Error::msg(e.to_string()))?,
            )
            .is_ok();
        if is_ok {
            Ok(Response::new().format(
                &LoginRespData {
                    password: password_hash,
                },
                resp_t,
            )?)
        } else {
            Ok(Response::new().status(Status::Unauthorized).text(
                utils::i18n!("errors.http.authentication_failed"),
            ))
        }
    } else {
        logger::error!(utils::i18n!(
            "errors.http.password_file_missing"
        ));
        Ok(Response::new()
            .status(Status::Unauthorized)
            .text(utils::i18n!("errors.http.authentication_failed")))
    }
}

pub fn generate_password_if_no_exists() -> anyhow::Result<()> {
    if let None = get_password_from_local()? {
        let (pwd_hash, pwd) = generate_password()?;
        println!("auto generate password: {}", pwd);
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(config::DATA_PATH.join(".password"))?;
        file.write_all(pwd_hash.as_bytes())?;
        file.sync_all()?;
    }

    Ok(())
}

fn generate_password() -> anyhow::Result<(String, String)> {
    let argon2: argon2::Argon2 = new_argon2()?;
    let password = passwords::PasswordGenerator::new()
        .length(8)
        .strict(true)
        .numbers(true)
        .lowercase_letters(true)
        .uppercase_letters(true)
        .spaces(false)
        .symbols(true)
        .generate_one()
        .map_err(|str| anyhow::Error::msg(str))?;
    let salt = SaltString::generate(&mut OsRng);
    Ok((
        argon2
            .hash_password_simple(password.as_bytes(), &salt)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?
            .to_string(),
        password
    ))
}

fn get_password_from_local() -> anyhow::Result<Option<String>> {
    let file_path = config::DATA_PATH.join(".password");
    Ok(if file_path.exists() {
        let hash = read_to_string(&file_path)?;
        Some(hash)
    } else {
        None
    })
}

fn new_argon2<'a>() -> anyhow::Result<argon2::Argon2<'a>> {
    Ok(argon2::Argon2::new(
        Option::None,
        10,
        2048,
        1,
        argon2::Version::V0x13,
    )
        .map_err(|e| anyhow::Error::msg(e.to_string()))?)
}

#[derive(serde::Deserialize)]
pub struct LoginReqData {
    password: String,
}

#[derive(serde::Serialize)]
struct LoginRespData {
    password: String,
}
