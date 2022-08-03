#![allow(clippy::unwrap_used)]
use std::net::IpAddr;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{
    de::{self, IntoDeserializer},
    Deserialize, Deserializer,
};
use time::{Date, Month};

use crate::{
    database::{backup::BackupType, Person},
    helpers::genesis_date,
};

use super::{ij::LimitKey, incoming_json::ij};

pub struct IncomingDeserializer;

lazy_static! {
    static ref REGEX_EMAIL: Regex = Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#).unwrap();
}

impl IncomingDeserializer {
    /// Is a given string the length given, and also only uses hex chars [a-zA-Z0-9]
    pub fn is_hex(input: &str, len: usize) -> bool {
        input.chars().count() == len && input.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Check if given &str is an email, return Some(lowercase email) or None
    pub fn valid_email(parsed: &str) -> Option<String> {
        if parsed.is_empty() || !parsed.contains('@') {
            return None;
        }
        let email = parsed.to_owned().to_lowercase();

        if let true = REGEX_EMAIL.is_match(&email) {
            Some(email)
        } else {
            None
        }
    }
    // yyyy-mm-dd_[D/J] - for uploading an image, name is set in clientside code
    pub fn parse_photo_name(file_name: &str) -> bool {
        let as_chars = || file_name.chars();

        if as_chars().count() != 12 {
            return false;
        }

        // Validate data correctly here
        let year = as_chars().take(4).collect::<String>();
        let month = as_chars().skip(5).take(2).collect::<String>();
        let day = as_chars().skip(8).take(2).collect::<String>();
        let person = as_chars().skip(11).take(1).collect::<String>();

        if Self::valid_year(&year).is_none()
            || Self::valid_month(&month).is_none()
            || Self::valid_day(&day).is_none()
            || !Self::valid_person_initial(&person)
        {
            return false;
        }

        format!("{}-{}-{}_{}", year, month, day, person) == file_name
    }

    /// Validate all parts, then validate as an acutal date (31 February fails etc)
    fn valid_date(year: i32, month: Month, day: u8) -> Option<Date> {
        match Date::from_calendar_date(year, month, day) {
            Ok(data) => Some(data),
            Err(_) => None,
        }
    }

    // Years only valid if => genesis date, up until whatever current year is
    fn valid_year(x: &str) -> Option<i32> {
        match x.parse::<i32>() {
            Ok(year) => {
                if (genesis_date().year()..=time::OffsetDateTime::now_utc().year()).contains(&year)
                {
                    Some(year)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// 01-12 to Month enum
    fn valid_month(x: &str) -> Option<Month> {
        match x.parse::<u8>() {
            Ok(month) => match Month::try_from(month) {
                Ok(data) => Some(data),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    /// Deosn't account for month, do that with `valid_date`
    fn valid_day(x: &str) -> Option<u8> {
        match x.parse::<u8>() {
            Ok(day) => {
                if (1..=31).contains(&day) {
                    Some(day)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn valid_hour(x: &str) -> Option<u8> {
        match x.parse::<u8>() {
            Ok(hour) => {
                if (0..=23).contains(&hour) {
                    Some(hour)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn valid_minute_second(x: &str) -> Option<u8> {
        match x.parse::<u8>() {
            Ok(m_s) => {
                if (0..=59).contains(&m_s) {
                    Some(m_s)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn valid_person_initial(x: &str) -> bool {
        x == "J" || x == "D"
    }

    fn valid_photo_type(x: &str) -> bool {
        x == "O" || x == "C"
    }

    // yyyy-mm-dd_[D/J]_[C/O]_[a-z0-1{16}].jpg
    fn parse_photo_name_with_hex(file_name: &str) -> bool {
        let as_chars = || file_name.chars();

        if as_chars().count() != 35 {
            return false;
        }

        if !file_name.ends_with(".jpg") {
            return false;
        }

        let start = as_chars().take(12).collect::<String>();
        let photo_type = as_chars().skip(13).take(1).collect::<String>();
        let hex = as_chars().skip(15).take(16).collect::<String>();

        if !Self::parse_photo_name(&start)
            || !Self::valid_photo_type(&photo_type)
            || !Self::is_hex(&hex, 16)
        {
            return false;
        }
        format!("{}_{}_{}.jpg", start, photo_type, hex) == file_name
    }

    // mealpedant_yyyy-mm-dd_hh.mm.ss_[NAME]_[a-f0-9]{8}.tar.gz.gpg
    pub fn parse_backup_name(file_name: &str) -> bool {
        let as_chars = || file_name.chars();

        if !file_name.ends_with(".tar.gpg") {
            return false;
        }

        if !file_name.starts_with("mealpedant_") {
            return false;
        }

        let op_backup_type = if as_chars().count() == 62 {
            Some(BackupType::SqlOnly)
        } else if as_chars().count() == 69 {
            Some(BackupType::Full)
        } else {
            None
        };

        if let Some(backup_type) = op_backup_type {
            // Validate date
            let date = as_chars().skip(11).take(10).collect::<String>();
            let year = date.chars().take(4).collect::<String>();
            let month = date.chars().skip(5).take(2).collect::<String>();
            let day = date.chars().skip(8).take(2).collect::<String>();

            // Validate time
            let time = as_chars().skip(22).take(8).collect::<String>();
            let hour = time.chars().take(2).collect::<String>();
            let minute = time.chars().skip(3).take(2).collect::<String>();
            let second = time.chars().skip(6).take(2).collect::<String>();

            let hex_skip = match backup_type {
                BackupType::Full => 53,
                BackupType::SqlOnly => 46,
            };

            let hex = as_chars().skip(hex_skip).take(8).collect::<String>();

            if Self::valid_year(&year).is_none()
                || Self::valid_month(&month).is_none()
                || Self::valid_day(&day).is_none()
                || Self::valid_hour(&hour).is_none()
                || Self::valid_minute_second(&minute).is_none()
                || Self::valid_minute_second(&second).is_none()
                || !Self::is_hex(&hex, 8)
            {
                return false;
            }

            let valid = format!("mealpedant_{year}-{month}-{day}_{hour}.{minute}.{second}_{backup_type}_{hex}.tar.gpg");
            valid == file_name
        } else {
            false
        }
    }

    /// Parse a string, custom error if failure
    fn parse_string<'de, D>(deserializer: D, name: &str) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer) {
            Ok(parsed) => Ok(parsed),
            Err(_) => Err(de::Error::custom(name)),
        }
    }

    /// Parse an i64, custom error if failure
    fn parse_i64<'de, D>(deserializer: D, name: &str) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        match i64::deserialize(deserializer) {
            Ok(parsed) => Ok(parsed),
            Err(_) => Err(de::Error::custom(name)),
        }
    }

    /// Check valid 2fa token, either hex 16, or six digits
    fn valid_token(token: &str) -> bool {
        Self::is_hex(token, 16)
            || token.chars().count() == 6 && token.chars().all(|c| c.is_ascii_digit())
    }

    /// Only allows string.len() > 12 && string.len() < 100 (counting chars!)
    fn string_range<'de, D>(deserializer: D, name: &str) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed = Self::parse_string(deserializer, name)?;

        let allowed_len = 12..=99;

        if !allowed_len.contains(&parsed.chars().count()) {
            return Err(de::Error::custom(name));
        }
        Ok(parsed)
    }

    /// Check email isn't empty, lowercase it, constains an '@' sign, and matches a 99.9% email regex
    pub fn email<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "email";
        let parsed = Self::parse_string(deserializer, name)?;

        if let Some(email) = Self::valid_email(&parsed) {
            Ok(email)
        } else {
            Err(de::Error::custom(name))
        }
    }
    /// Check email isn't empty, lowercase it, constains an '@' sign, and matches a 99.9% email regex
    pub fn vec_email<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "vec_email";
        let parsed: Vec<String> = Vec::deserialize(deserializer)?;

        if !parsed.is_empty() && parsed.iter().all(|i| Self::valid_email(i).is_some()) {
            Ok(parsed
                .iter()
                .map(|i| i.to_lowercase())
                .collect::<Vec<String>>())
        } else {
            Err(de::Error::custom(name))
        }
    }

    /// Only allows strings > 0 & alpha/or space, and also trims result
    /// So "John", "John ", "John Smith" "John Smith " are valid & then trimmed
    pub fn name<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "name";
        let parsed = Self::parse_string(deserializer, name)?;
        if parsed.chars().count() < 1
            || parsed.trim().chars().count() < 1
            || !parsed.chars().all(|x| x.is_alphabetic() || x == ' ')
        {
            return Err(de::Error::custom(name));
        }
        Ok(parsed.trim().into())
    }

    /// Only allow tokens in either format 000 000 (with/without space)
    /// or a backup token 0123456789abcedf, again spaces get removed, will be uppercased
    pub fn token<'de, D>(deserializer: D) -> Result<ij::Token, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "token";
        let mut parsed = Self::parse_string(deserializer, name)?;

        // Remove any spaces from the token string and lowercase it
        parsed = parsed.replace(' ', "");

        if Self::valid_token(&parsed) {
            if parsed.chars().count() == 6 {
                Ok(ij::Token::Totp(parsed))
            } else {
                Ok(ij::Token::Backup(parsed.to_uppercase()))
            }
        } else {
            Err(de::Error::custom(name))
        }
    }

    // TEST ME
    pub fn option_photo_name_hex<'de, D>(deserializer: D) -> Result<Option<ij::PhotoName>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Some(x) = Option::<String>::deserialize(deserializer)? {
            Ok(Some(Self::photo_name_hex(x.into_deserializer())?))
        } else {
            Ok(None)
        }
    }

    /// Only allow tokens in either format 000 000 (with/without space)
    /// or a backup token 0123456789abcedf, again spaces get removed, will be uppercased
    pub fn photo_name_hex<'de, D>(deserializer: D) -> Result<ij::PhotoName, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "photo_name";
        let parsed = Self::parse_string(deserializer, name)?;

        if Self::parse_photo_name_with_hex(&parsed) {
            if parsed.contains('O') {
                Ok(ij::PhotoName::Original(parsed))
            } else {
                Ok(ij::PhotoName::Converted(parsed))
            }
        } else {
            Err(de::Error::custom(name))
        }
    }

    /// Only allow tokens in either format 000 000 (with/without space)
    /// or a backup token 0123456789abcedf, again spaces get removed, will be uppercased
    pub fn backup_name<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "backup_name";
        let parsed = Self::parse_string(deserializer, name)?;

        if Self::parse_backup_name(&parsed) {
            Ok(parsed)
        } else {
            Err(de::Error::custom(name))
        }
    }

    // TEST ME
    pub fn option_token<'de, D>(deserializer: D) -> Result<Option<ij::Token>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Some(x) = Option::<String>::deserialize(deserializer)? {
            Ok(Some(Self::token(x.into_deserializer())?))
        } else {
            Ok(None)
        }
    }

    /// Only allows strings > 12 && string < 100
    pub fn password<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::string_range(deserializer, "password")
    }

    // TEST ME
    pub fn option_password<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Some(x) = Option::<String>::deserialize(deserializer)? {
            Ok(Some(Self::password(x.into_deserializer())?))
        } else {
            Ok(None)
        }
    }

    /// Parse as IP addr, or Email, else error
    pub fn limit<'de, D>(deserializer: D) -> Result<LimitKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "limit";
        let mut parsed = Self::parse_string(deserializer, name)?;

        parsed = parsed.to_lowercase();

        match parsed.trim().parse::<IpAddr>() {
            Ok(ip) => Ok(LimitKey::Ip(ip)),
            Err(_) => {
                if let Some(email) = Self::valid_email(&parsed) {
                    Ok(LimitKey::Email(email))
                } else {
                    Err(de::Error::custom(name))
                }
            }
        }
    }

    /// Only allows "Dave" or "Jack"
    pub fn person<'de, D>(deserializer: D) -> Result<Person, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "person";
        let parsed = Self::parse_string(deserializer, name)?;

        match Person::new(&parsed) {
            Ok(person) => Ok(person),
            Err(_) => Err(de::Error::custom(name)),
        }
    }

    /// Only allows dates, yyyy-mm-dd, that are equal to, or greater than, the genesis date
    pub fn date<'de, D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "date";
        let parsed = Self::parse_string(deserializer, name)?;

        let as_chars = || parsed.chars();
        if as_chars().count() != 10 {
            return Err(de::Error::custom(name));
        }

        let op_year = Self::valid_year(&as_chars().take(4).collect::<String>());
        let op_month = Self::valid_month(&as_chars().skip(5).take(2).collect::<String>());
        let op_day = Self::valid_day(&as_chars().skip(8).take(2).collect::<String>());

        if let (Some(year), Some(month), Some(day)) = (op_year, op_month, op_day) {
            if let Some(date) = Self::valid_date(year, month, day) {
                return Ok(date);
            }
        }
        Err(de::Error::custom(name))
    }

    /// Only allows strings > 12 && string < 100
    pub fn invite<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::string_range(deserializer, "invite")
    }

    /// Allow only positive i64, due to sql id issues
    pub fn id<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "id";
        let parsed = Self::parse_i64(deserializer, name)?;
        if parsed < 1 {
            return Err(de::Error::custom(name));
        }
        Ok(parsed)
    }

    // COPY ME
    pub fn option_id<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Some(x) = Option::<i64>::deserialize(deserializer)? {
            Ok(Some(Self::id(x.into_deserializer())?))
        } else {
            Ok(None)
        }
    }
}

/// incoming_serializer
///
/// cargo watch -q -c -w src/ -x 'test incoming_serializer -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use serde::de::value::{Error as ValueError, SeqDeserializer, StringDeserializer};
    use serde::de::{value::I64Deserializer, IntoDeserializer};

    use rand::{distributions::Alphanumeric, Rng};

    use crate::api::api_tests::{ANON_EMAIL, TEST_EMAIL};
    use crate::helpers::gen_random_hex;

    use super::*;

    fn ran_s(x: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(x)
            .map(char::from)
            .collect()
    }

    fn ran_n() -> i64 {
        rand::thread_rng().gen_range(0..2500)
    }

    fn ran_token(backup: bool) -> String {
        if backup {
            let charset = b"abcdef0123456789";
            let token_len = 16;
            let mut rng = rand::thread_rng();

            (0..token_len)
                .map(|_| {
                    let idx = rng.gen_range(0..charset.len());
                    charset[idx] as char
                })
                .collect()
        } else {
            let digit = || rand::thread_rng().gen_range(0..=9);
            format!(
                "{}{}{}{}{}{}",
                digit(),
                digit(),
                digit(),
                digit(),
                digit(),
                digit()
            )
        }
    }

    #[test]
    fn helpers_is_hex() {
        let len = 16;
        let result = gen_random_hex(len);

        assert!(IncomingDeserializer::is_hex(&result, len.into()));

        let len = 16;
        let result = gen_random_hex(len);
        assert!(IncomingDeserializer::is_hex(
            &result.to_lowercase(),
            len.into()
        ));

        let len = 128;
        let result = gen_random_hex(len);
        assert!(IncomingDeserializer::is_hex(&result, len.into()));

        let len = 128;
        let result = gen_random_hex(len);
        assert!(IncomingDeserializer::is_hex(
            &result.to_lowercase(),
            len.into()
        ));

        let len = 16;
        let result = format!("{}g", gen_random_hex(len));
        assert!(!IncomingDeserializer::is_hex(&result, 17));

        let len = 16;
        let result = format!("{}%", gen_random_hex(len));
        assert!(!IncomingDeserializer::is_hex(&result, 17));

        let len = 16;
        let result = gen_random_hex(len);
        assert!(!IncomingDeserializer::is_hex(&result.to_lowercase(), 17));
    }

    #[test]
    fn incoming_serializer_date_err() {
        let test = |name: String| {
            let deserializer: StringDeserializer<ValueError> = name.into_deserializer();
            let result = IncomingDeserializer::date(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "date");
        };

        // before genesis date
        test("2014-01-01".to_owned());

        // in the future
        test("2100-01-01".to_owned());

        // invalid month
        test("2020-20-01".to_owned());

        // invalid day
        test("2020-01-40".to_owned());

        // missing parts
        test("2020-01".to_owned());
        test("01-2020-01".to_owned());
        test("2020-30-04".to_owned());

        // Random
        test(gen_random_hex(10));
    }

    #[test]
    fn incoming_serializer_date_ok() {
        let test = |name: String| {
            let deserializer: StringDeserializer<ValueError> = name.into_deserializer();
            let result = IncomingDeserializer::date(deserializer);
            assert!(result.is_ok());
        };

        // after genesis date
        test("2015-10-01".to_owned());

        // Before today
        test("2019-01-02".to_owned());
    }

    #[test]
    fn incoming_serializer_photo_with_hex() {
        // Valid
        assert!(IncomingDeserializer::parse_photo_name_with_hex(
            "2020-01-22_J_C_abcdef1234567890.jpg"
        ));
        assert!(IncomingDeserializer::parse_photo_name_with_hex(
            "2020-01-22_D_O_abcdef1234567890.jpg"
        ));

        // Invalid
        assert!(!IncomingDeserializer::parse_photo_name_with_hex(
            "2020-1-22_J_C_abcdef1234567890.jpg"
        ));
        assert!(!IncomingDeserializer::parse_photo_name_with_hex(
            "2020-01-32_J_C_abcdef1234567890.jpg"
        ));
        assert!(!IncomingDeserializer::parse_photo_name_with_hex(
            "2020-01-22_R_C_abcdef1234567890.jpg"
        ));
        assert!(!IncomingDeserializer::parse_photo_name_with_hex(
            "2020-01-22_J_I_abcdef1234567890.jpg"
        ));
        assert!(!IncomingDeserializer::parse_photo_name_with_hex(
            "2020-01-22_D_O_abcdef123456789z.jpg"
        ));
        assert!(!IncomingDeserializer::parse_photo_name(&gen_random_hex(35)));
    }

    #[test]
    fn incoming_serializer_photo() {
        // Valid
        assert!(IncomingDeserializer::parse_photo_name("2020-01-22_J"));
        assert!(IncomingDeserializer::parse_photo_name("2020-01-22_D"));

        // Invalid
        assert!(!IncomingDeserializer::parse_photo_name("2020-01-22_P"));
        assert!(!IncomingDeserializer::parse_photo_name("2020-01-22"));
        assert!(!IncomingDeserializer::parse_photo_name("2020-01-32_D"));
        assert!(!IncomingDeserializer::parse_photo_name("2020-22-22_D"));
        assert!(!IncomingDeserializer::parse_photo_name(&gen_random_hex(12)));
    }

    #[test]
    fn incoming_serializer_photo_name_invalid() {
        let test = |name: String| {
            let deserializer: StringDeserializer<ValueError> = name.into_deserializer();
            let result = IncomingDeserializer::photo_name_hex(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "photo_name");
        };

        test(String::from("0"));
        test(String::from("123"));

        test(String::from("2020-1-22_J_C_abcdef1234567890.jpg"));
        test(String::from("2020-01-32_J_C_abcdef1234567890.jpg"));
        test(String::from("2020-01-22_R_C_abcdef1234567890.jpg"));
        test(String::from("2020-01-22_J_I_abcdef1234567890.jpg"));
        test(String::from("2020-01-22_D_O_abcdef123456789z.jpg"));
        test(gen_random_hex(35));
    }

    #[test]
    fn incoming_serializer_photo_name_valid() {
        let test = |name: String| {
            let deserializer: StringDeserializer<ValueError> = name.into_deserializer();
            let result = IncomingDeserializer::photo_name_hex(deserializer);
            assert!(result.is_ok());
        };

        test(String::from("2020-10-22_J_C_abcdef1234567890.jpg"));
        test(String::from("2020-01-20_J_C_abcdef1234567890.jpg"));
        test(String::from("2020-01-22_D_C_abcdef1234567890.jpg"));
        test(String::from("2020-01-22_J_O_abcdef1234567890.jpg"));
        test(String::from("2020-01-22_D_O_abcdef1234567891.jpg"));
    }

    #[test]
    fn incoming_serializer_backup_name_invalid() {
        let test = |name: String| {
            let deserializer: StringDeserializer<ValueError> = name.into_deserializer();
            let result = IncomingDeserializer::backup_name(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "backup_name");
        };

        // missing prefix
        test(String::from(
            "xxxxxxxxxx_2022-07-03_03.01.00_LOGS_REDIS_SQL_8159c23e.tar.gpg",
        ));

        // missing suffix
        test(String::from(
            "2022-07-03_03.01.00_LOGS_REDIS_SQL_8159c23e.tar.xxx",
        ));
        // invalid date
        test(String::from(
            "mealpedant_1999-07-03_03.01.00_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.gpg",
        ));
        test(String::from(
            "mealpedant_2020-14-03_03.01.00_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.gpg",
        ));
        test(String::from(
            "mealpedant_2020-12-34_03.01.00_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.gpg",
        ));

        // invalid time
        test(String::from(
            "mealpedant_2022-07-03_24.01.00_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.gpg",
        ));
        test(String::from(
            "mealpedant_2022-07-03_03.63.00_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.gpg",
        ));
        test(String::from(
            "mealpedant_2022-07-03_03.01.72_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.gpg",
        ));

        // invalid name
        test(String::from(
            "mealpedant_2022-07-03_03.01.00_lOGS_REDIS_SQL_8159c23e.tar.gpg",
        ));
        test(String::from(
            "mealpedant_2022-07-03_03.01.00_LOGS_PHOTO_REDIS_SQL_8159c23e.tar.gpg",
        ));
        test(String::from(
            "mealpedant_2022-07-03_03.01.00_lOGS_REDIS_8159c23e.tar.gpg",
        ));

        // invalid hex
        test(String::from(
            "mealpedant_2022-07-03_03.01.00_LOGS_REDIS_SQL_8159c23.tar.gpg",
        ));
        test(String::from(
            "mealpedant_2022-07-03_03.01.00_LOGS_REDIS_SQL_8159c23K.tar.gpg",
        ));

        // /random
        test(gen_random_hex(10));
        test(gen_random_hex(62));
        test(gen_random_hex(69));
    }

    #[test]
    fn incoming_serializer_backup_name_valid() {
        let test = |name: String| {
            let deserializer: StringDeserializer<ValueError> = name.into_deserializer();
            let result = IncomingDeserializer::backup_name(deserializer);
            assert!(result.is_ok());
        };

        test(String::from(
            "mealpedant_2022-07-03_03.01.00_LOGS_REDIS_SQL_8159c23e.tar.gpg",
        ));
        test(String::from(
            "mealpedant_2022-07-03_03.01.00_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.gpg",
        ));
    }

    #[test]
    fn incoming_serializer_id_err() {
        let test = |id: String| {
            let deserializer: StringDeserializer<ValueError> = id.into_deserializer();
            let result = IncomingDeserializer::id(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "id");
        };

        test(String::from("0"));
        test(String::from("123"));

        let deserializer: I64Deserializer<ValueError> = 0i64.into_deserializer();
        let result = IncomingDeserializer::id(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "id");
    }

    #[test]
    fn incoming_serializer_id_ok() {
        // add one, just to make sure 0 doesn't get used
        let id = ran_n() + 1;
        let deserializer: I64Deserializer<ValueError> = id.into_deserializer();
        let result = IncomingDeserializer::id(deserializer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), id);
    }

    #[test]
    fn incoming_serializer_token_ok() {
        // Should split tests, match as totp, or match as backup
        let test = |token: String| {
            let deserializer: StringDeserializer<ValueError> = token.clone().into_deserializer();
            let result = IncomingDeserializer::token(deserializer);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap().to_string(),
                token.replace(' ', "").to_uppercase()
            );
        };

        test(String::from("111111"));
        test(String::from("111 111"));
        test(String::from(" 111 111 "));
        test(ran_token(false));
        test(String::from("aaaaaabbbbbb1234"));
        test(String::from("aaaaa abbbbbb1 234"));
        test(ran_token(true));
        test(ran_token(true).to_uppercase());
    }

    #[test]
    fn incoming_serializer_token_err() {
        let test = |token: String| {
            let deserializer: StringDeserializer<ValueError> = token.into_deserializer();
            let result = IncomingDeserializer::token(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "token");
        };

        test(String::from("12345"));
        test(String::from("1234567"));
        test(String::from("12345a"));
        test(String::from("aaaabbbbccccdddd1"));
        test(String::from("zzzzzzzzzzzzzzzz"));
        test(format!("{}z", ran_token(true)));
    }

    #[test]
    fn incoming_serializer_limit_err() {
        let test = |x: String| {
            let deserializer: StringDeserializer<ValueError> = x.into_deserializer();
            let result = IncomingDeserializer::limit(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "limit");
        };
        let p = || rand::thread_rng().gen_range(255..455);
        test(format!("{}.{}.{}.{}", p(), p(), p(), p()));

        test(String::from("email@email"));
        test(String::from("emailemail.com"));
        test(String::from("@emailemail.com"));
        test(String::from("127.127.127"));
        test(format!("127.127.127.{}", p()));
        test(String::from(".127.127.127"));
    }

    #[test]
    fn incoming_serializer_limit_ok() {
        let test = |x: String| {
            let deserializer: StringDeserializer<ValueError> = x.clone().into_deserializer();
            let result = IncomingDeserializer::limit(deserializer);
            assert!(result.is_ok());
            match result.unwrap() {
                LimitKey::Email(e) => assert_eq!(e, x.to_lowercase()),
                LimitKey::Ip(i) => assert_eq!(i.to_string(), x),
            }
        };
        let p = || rand::thread_rng().gen_range(0..255);

        test(String::from("email@email.com"));
        test(String::from("email@email.com").to_uppercase());
        test(format!("{}@{}.{}", ran_s(10), ran_s(10), ran_s(3)));
        test(format!("{}@{}.{}", ran_s(10), ran_s(10), ran_s(3)).to_uppercase());
        test(format!(
            "{}@{}.{}.{}",
            ran_s(10),
            ran_s(10),
            ran_s(2),
            ran_s(2)
        ));

        test(String::from("127.0.0.1"));
        test(String::from("255.255.255.255"));
        test(format!("{}.{}.{}.{}", p(), p(), p(), p()));
    }

    #[test]
    fn incoming_serializer_email_ok() {
        let test = |email: String| {
            let deserializer: StringDeserializer<ValueError> = email.clone().into_deserializer();
            let result = IncomingDeserializer::email(deserializer);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), email.to_lowercase());
        };

        test(String::from("email@email.com"));
        test(String::from("email@email.com").to_uppercase());
        test(format!("{}@{}.{}", ran_s(10), ran_s(10), ran_s(3)));
        test(format!("{}@{}.{}", ran_s(10), ran_s(10), ran_s(3)).to_uppercase());
        test(format!(
            "{}@{}.{}.{}",
            ran_s(10),
            ran_s(10),
            ran_s(2),
            ran_s(2)
        ));
    }

    #[test]
    fn incoming_serializer_email_err() {
        let test = |email: String| {
            let deserializer: StringDeserializer<ValueError> = email.into_deserializer();
            let result = IncomingDeserializer::email(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "email");
        };

        test(String::from("emailemail.com"));
        test(String::from(""));
        test(String::from(" "));
        test(String::from(" @ . "));
        test(String::from(" @.com"));
        test(String::from(" @ .com"));
        test(String::from("email@"));
        test(String::from("@email.com"));
        test(String::from("email@email"));
        test(String::from("email@email."));

        let deserializer: I64Deserializer<ValueError> = ran_n().into_deserializer();
        let result = IncomingDeserializer::email(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "email");
    }

    #[test]
    fn incoming_serializer_vec_email_ok() {
        let test = |x: Vec<String>| {
            let deserializer: SeqDeserializer<std::vec::IntoIter<String>, ValueError> =
                x.clone().into_deserializer();
            let result = IncomingDeserializer::vec_email(deserializer);
            assert!(result.is_ok());
            assert_eq!(result.as_ref().unwrap().len(), x.len());
            assert_eq!(result.unwrap()[0], x[0].to_lowercase());
        };

        test(vec![
            "email@email.com".to_string(),
            "email@abc.com".to_string(),
            ANON_EMAIL.to_string(),
            TEST_EMAIL.to_string(),
        ]);
        test(vec![
            "EMAIL@EMAIL.COM".to_string(),
            "email@abc.com".to_string(),
            ANON_EMAIL.to_string(),
            TEST_EMAIL.to_string(),
        ]);
    }

    #[test]
    fn incoming_serializer_vec_email_err() {
        let test = |x: Vec<String>| {
            let deserializer: SeqDeserializer<std::vec::IntoIter<String>, ValueError> =
                x.into_deserializer();
            let result = IncomingDeserializer::vec_email(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "vec_email");
        };
        test(vec![]);
        test(vec![
            "emailemail.com".to_string(),
            "email@abc.com".to_string(),
            ANON_EMAIL.to_string(),
            TEST_EMAIL.to_string(),
        ]);
        test(vec![
            "email@email".to_string(),
            "email@abc.com".to_string(),
            ANON_EMAIL.to_string(),
            TEST_EMAIL.to_string(),
        ]);
        test(vec![
            "email@.com".to_string(),
            "email@abc.com".to_string(),
            ANON_EMAIL.to_string(),
            TEST_EMAIL.to_string(),
        ]);
        test(vec![gen_random_hex(12)]);
    }

    #[test]
    fn incoming_serializer_name_ok() {
        let test = |name: String| {
            let deserializer: StringDeserializer<ValueError> = name.clone().into_deserializer();
            let result = IncomingDeserializer::name(deserializer);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), name.trim());
        };

        test(String::from("aabbccd"));
        test(String::from("sdfsdf "));
        test(String::from("sdfsdf "));
        test(String::from("sdfsdf bakaks"));
        test(String::from(" sdfsdf bakaks "));
    }

    #[test]
    fn incoming_serializer_name_err() {
        let test = |name: String| {
            let deserializer: StringDeserializer<ValueError> = name.into_deserializer();
            let result = IncomingDeserializer::name(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "name");
        };

        test(String::from("invalid.name"));
        test(String::from("invalid1name"));
        test(String::from("John 1 Smith"));
        test(String::from(""));
        test(String::from(" "));
        test(String::from("        "));

        let deserializer: I64Deserializer<ValueError> = ran_n().into_deserializer();
        let result = IncomingDeserializer::name(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "name");
    }

    #[test]
    fn incoming_serializer_password() {
        let test = |password: String| {
            let deserializer: StringDeserializer<ValueError> = password.clone().into_deserializer();
            let result = IncomingDeserializer::password(deserializer);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), password);
        };

        test(ran_s(12));

        test(String::from("            "));

        test(ran_s(40));

        test(ran_s(99));
    }

    #[test]
    fn incoming_serializer_password_err() {
        let test = |password: String| {
            let deserializer: StringDeserializer<ValueError> = password.into_deserializer();
            let result = IncomingDeserializer::password(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "password");
        };

        test(String::from(""));

        test(ran_s(11));

        test(ran_s(100));

        let deserializer: I64Deserializer<ValueError> = ran_n().into_deserializer();
        let result = IncomingDeserializer::password(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "password");
    }

    #[test]
    fn incoming_serializer_invite() {
        let test = |invite: String| {
            let deserializer: StringDeserializer<ValueError> = invite.clone().into_deserializer();
            let result = IncomingDeserializer::invite(deserializer);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), invite);
        };

        test(ran_s(12));

        test(String::from("            "));

        test(ran_s(40));

        test(ran_s(99));
    }

    #[test]
    fn incoming_serializer_invite_err() {
        let test = |invite: String| {
            let deserializer: StringDeserializer<ValueError> = invite.into_deserializer();
            let result = IncomingDeserializer::invite(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "invite");
        };

        test(String::from(""));

        test(ran_s(11));

        test(ran_s(100));

        let deserializer: I64Deserializer<ValueError> = ran_n().into_deserializer();
        let result = IncomingDeserializer::invite(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "invite");
    }
}
