use argon2::password_hash::SaltString;
use argon2::{
    Algorithm, Argon2, ParamsBuilder, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use once_cell::sync::Lazy;
use rand_core::OsRng;

static ARGON2: Lazy<Argon2> = Lazy::new(|| {
    Argon2::new(Algorithm::Argon2id, Version::V0x13, {
        const ERROR_MSG: &str = "wrong argon2 params";
        let mut params = ParamsBuilder::new();
        params
            .m_cost(1024)
            .expect(ERROR_MSG)
            .t_cost(5)
            .expect(ERROR_MSG)
            .p_cost(2)
            .expect(ERROR_MSG);
        params.params().expect(ERROR_MSG)
    })
});

pub fn password_hash(
    password: String,
) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    Ok(ARGON2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

pub fn password_verify(
    password: &[u8],
    password_hash: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(password_hash)?;

    Ok(ARGON2.verify_password(password, &parsed_hash).is_ok())
}
