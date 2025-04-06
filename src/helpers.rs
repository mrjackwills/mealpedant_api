use crate::api_error::ApiError;
use jiff::{Timestamp, Zoned, civil::Date, tz::TimeZone};
use rand::Rng;
use std::time::SystemTime;

const HEX_CHARS: &[u8; 16] = b"ABCDEF0123456789";

#[cfg(not(test))]
use crate::S;
#[cfg(not(test))]
use sha1::{Digest, Sha1};
#[cfg(not(test))]
use tracing::error;
#[cfg(not(test))]
const HIBP: &str = "https://api.pwnedpasswords.com/range/";

/// Day 1 of Meal Pedant, no meal can exist before this date
pub const fn genesis_date() -> Date {
    jiff::civil::Date::constant(2015, 5, 9)
}

/// Get the current UTC time
pub fn now_utc() -> Zoned {
    Timestamp::now().to_zoned(TimeZone::UTC)
}
/// use app_env.start_time to work out how long the application has been running for, in seconds
pub fn calc_uptime(start_time: SystemTime) -> u64 {
    std::time::SystemTime::now()
        .duration_since(start_time)
        .map_or(0, |value| value.as_secs())
}
/// Generate a random, uppercase, hex string of length output_len
pub fn gen_random_hex(output_len: u8) -> String {
    let mut rng = rand::thread_rng();
    (0..output_len)
        .map(|_| {
            let idx = rng.gen_range(0..HEX_CHARS.len());
            HEX_CHARS[idx] as char
        })
        .collect::<String>()
}

/// Check if two byte arrays match, rather than ==
pub fn xor(input_1: &[u8], input_2: &[u8]) -> bool {
    if input_1.len() != input_2.len() {
        return false;
    }
    std::iter::zip(input_1, input_2)
        .map(|x| (x.0 ^ x.1) as usize)
        .sum::<usize>()
        == 0
}

/// Check if a given password in is HIBP using K-Anonymity
#[cfg(not(test))]
pub async fn pwned_password(password: &str) -> Result<bool, ApiError> {
    let mut sha_digest = Sha1::default();
    sha_digest.update(password.as_bytes());
    let password_hex = hex::encode(sha_digest.finalize()).to_uppercase();
    let split_five = password_hex.split_at(5);
    match reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_millis(10000))
        .gzip(true)
        .brotli(true)
        .build()?
        .get(format!("{HIBP}{}", split_five.0))
        .send()
        .await
    {
        Ok(data) => {
            let response = data.text().await.unwrap_or_default();
            Ok(response.lines().any(|line| {
                let result_split = line.split_once(':').unwrap_or_default();
                // Check not "0", as some results get padded with a "0" response, if don't meet minimum number (think currently 381)
                result_split.0 == split_five.1 && result_split.1 != "0"
            }))
        }
        Err(e) => {
            error!(%e);
            Err(ApiError::Internal(S!("hibp request error")))
        }
    }
}

#[cfg(test)]
#[allow(clippy::unused_async)]
/// When in test config, return true if password is "ILOVEYOU1234", else false
/// So that tests can be run without network connectivity
pub async fn pwned_password(password: &str) -> Result<bool, ApiError> {
    Ok(password.to_uppercase() == "ILOVEYOU1234")
}

/// cargo watch -q -c -w src/ -x 'test helpers_ -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Probably pointless, as we're now not checking against the live service when testing
    /// "ILOVEYOU1234" will be a pwned password, anything else is fine
    #[tokio::test]
    async fn helpers_pwned_password() {
        let result = pwned_password("ILOVEYOU1234").await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        let result = pwned_password("iloveyou1234").await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        let result = pwned_password("this_shouldn't_be_in_hibp_¯;ë±¨ÛdëzF=êÆVÜ;Ê_a¤ª<ý*;3¼z#±~xæ9áSÀ4õaJõò)*p'~fL¯se/)D¡½þ¡Kãß¢").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn helpers_random_hex() {
        let len = 16;
        let result = gen_random_hex(len);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(result.chars().count() == len as usize);

        let len = 64;
        let result = gen_random_hex(len);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(result.chars().count() == len as usize);

        let len = 128;
        let result = gen_random_hex(len);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(result.chars().count() == len as usize);
    }

    #[test]
    fn helpers_xor() {
        let s1 = gen_random_hex(16);
        let result = xor(s1.as_bytes(), s1.as_bytes());
        assert!(result);

        let s1 = gen_random_hex(16);
        let s2 = gen_random_hex(17);
        let result = xor(s1.as_bytes(), s2.as_bytes());
        assert!(!result);

        let s1 = gen_random_hex(16);
        let result = xor(s1.as_bytes(), s1.to_lowercase().as_bytes());
        assert!(!result);

        let s1 = gen_random_hex(16);
        let s2 = gen_random_hex(16);
        let result = xor(s1.as_bytes(), s2.as_bytes());
        assert!(!result);
    }
}
