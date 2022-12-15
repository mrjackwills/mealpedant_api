use argon2::password_hash::SaltString;
use argon2::{Algorithm::Argon2id, Argon2, Params, ParamsBuilder, PasswordHash, Version::V0x13};
use std::fmt;
use tracing::error;

use crate::api_error::ApiError;

/// reduce t cost for testing only, else too slow
#[cfg(not(release))]
#[allow(clippy::unwrap_used)]
fn get_params() -> Params {
    let mut params = ParamsBuilder::new();
    params.m_cost(4096).unwrap();
    params.t_cost(1).unwrap();
    params.p_cost(1).unwrap();
    params.params().unwrap()
}

// This takes 19 seconds when testing, t_cost issue!
#[cfg(release)]
#[allow(clippy::unwrap_used)]
fn get_params() -> Params {
    let mut params = ParamsBuilder::new();
    params.m_cost(4096).unwrap();
    params.t_cost(190).unwrap();
    params.p_cost(1).unwrap();
    params.params().unwrap()
}

fn get_hasher() -> Argon2<'static> {
    Argon2::new(Argon2id, V0x13, get_params())
}

// Need to look into this
#[derive(Clone, PartialEq, Eq)]
pub struct ArgonHash(pub String);

impl fmt::Display for ArgonHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// TODO impl from, check that strings starts with `$argon2id$` etc

impl ArgonHash {
    // pub fn get(&self) -> String{
    // 	*self.0
    // }

    pub async fn new(password: String) -> Result<Self, ApiError> {
        let password_hash = Self::hash_password(password).await?;
        Ok(Self(password_hash))
    }

    /// create a password hash, use blocking to run in own thread
    async fn hash_password(password: String) -> Result<String, ApiError> {
        tokio::task::spawn_blocking(move || -> Result<String, ApiError> {
            let salt = SaltString::generate(rand::thread_rng());
            match PasswordHash::generate(get_hasher(), password, salt.as_str()) {
                Ok(hash) => Ok(hash.to_string()),
                Err(e) => {
                    error!(%e);
                    Err(ApiError::Internal(String::from("password_hash generate")))
                }
            }
        })
        .await?
    }
}

/// check a password against a known password hash, use blocking to run in own thread
pub async fn verify_password(password: &str, argon_hash: ArgonHash) -> Result<bool, ApiError> {
    let password = password.to_owned();
    tokio::task::spawn_blocking(move || -> Result<bool, ApiError> {
        PasswordHash::new(&argon_hash.0).map_or(
            Err(ApiError::Internal(String::from(
                "verify_password::new_hash",
            ))),
            |hash| match hash.verify_password(&[&get_hasher()], password) {
                Ok(_) => Ok(true),
                Err(e) => match e {
                    // Could always just return false, no need to worry about internal errors?
                    argon2::password_hash::Error::Password => Ok(false),
                    _ => Err(ApiError::Internal(String::from("verify_password"))),
                },
            },
        )
    })
    .await?
}

/// http tests - ran via actual requests to a (local) server
/// cargo watch -q -c -w src/ -x 'test argon_mod -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery, clippy::unwrap_used)]
mod tests {

    use once_cell::sync::Lazy;
    use rand::{distributions::Alphanumeric, Rng};
    use regex::Regex;

    use super::*;

    static ARGON_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"^\$argon2id\$v=19\$m=4096,t=1,p=1\$[a-zA-Z0-9+/=]{22}\$[a-zA-Z0-9+/=]{43}"#)
            .unwrap()
    });

    fn ran_s(x: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(x)
            .map(char::from)
            .collect()
    }

    #[tokio::test]
    async fn argon_mod_hash() {
        let password = ran_s(20);
        let result = ArgonHash::new(password.clone()).await;
        assert!(result.is_ok());
        assert!(ARGON_REGEX.is_match(&result.unwrap().to_string()));
    }

    #[tokio::test]
    async fn argon_mod_verify_random() {
        let password = ran_s(20);
        let argon_hash = ArgonHash::new(password.clone()).await.unwrap();

        // Verify true
        let result = verify_password(&password, argon_hash).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify false
        let short_pass = password.chars().take(19).collect::<String>();
        let argon_hash = ArgonHash::new(password.clone()).await.unwrap();
        let result = verify_password(&short_pass, argon_hash).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn argon_mod_verify_known() {
        let password = "This is a known password";
        let password_hash = ArgonHash("$argon2id$v=19$m=4096,t=5,p=1$rahU5enqn3WcOo9A58Ifjw$I+7yA6+29LuB5jzPUwnxtLoH66Lng7ExWqHdivwj8Es".to_owned());

        // Verify true
        let result = verify_password(password, password_hash.clone()).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify false
        let result = verify_password("this is a known password", password_hash).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
