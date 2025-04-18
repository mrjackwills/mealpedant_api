use jiff::civil::Date;
use regex::Regex;
use serde::{
    Deserialize, Deserializer,
    de::{self, IntoDeserializer},
};
use std::{net::IpAddr, sync::LazyLock};
use ulid::Ulid;

use crate::{
    database::{Person, backup::BackupType},
    helpers::genesis_date,
};

use super::{ij::LimitKey, incoming_json::ij};

pub struct IncomingDeserializer;

#[expect(clippy::unwrap_used)]
static REGEX_EMAIL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#).unwrap()
});

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

        if REGEX_EMAIL.is_match(&email) {
            Some(email)
        } else {
            None
        }
    }
    // D/J - for uploading an image, name is set in clientside code
    pub fn parse_photo_name(file_name: &str) -> bool {
        if file_name.chars().count() != 1 {
            return false;
        }

        // Validate data correctly here
        Self::valid_person_initial(file_name)
    }

    /// Validate a date as being valid
    fn valid_meal_date(year: i16, month: i8, day: i8) -> Option<Date> {
        Date::new(year, month, day).ok()
    }

    // Years only valid if => genesis date, up until whatever current year is
    fn valid_year(x: &str) -> Option<i16> {
        x.parse::<i16>().map_or(None, |year| {
            if year >= genesis_date().year() {
                Some(year)
            } else {
                None
            }
        })
    }

    /// 01-12 to Month enum
    fn valid_month(x: &str) -> Option<i8> {
        x.parse::<i8>().map_or(None, |month| {
            if (1..=12).contains(&month) {
                Some(month)
            } else {
                None
            }
        })
    }

    /// Doesn't account for month, do that with `valid_date`
    fn valid_day(x: &str) -> Option<i8> {
        x.parse::<i8>().map_or(None, |day| {
            if (1..=31).contains(&day) {
                Some(day)
            } else {
                None
            }
        })
    }

    fn valid_person_initial(x: &str) -> bool {
        x == "J" || x == "D"
    }

    /// mealpedant_yyyy-mm-dd_hh.mm.ss_[NAME]_[a-f0-9]{8}.tar.gz.age
    pub fn parse_backup_name(file_name: &str) -> bool {
        let as_chars = || file_name.chars();

        if !file_name.ends_with(".tar.age") {
            return false;
        }

        if !file_name.starts_with("mealpedant_") {
            return false;
        }

        let backup_type = if as_chars().count() == 62 {
            Some(BackupType::SqlOnly)
        } else if as_chars().count() == 69 {
            Some(BackupType::Full)
        } else {
            None
        };

        let Some(backup_type) = backup_type else {
            return false;
        };

        let date = as_chars().skip(11).take(10).collect::<String>();
        let Ok(parsed_date) = date.parse::<jiff::civil::Date>() else {
            return false;
        };

        if parsed_date < genesis_date() {
            return false;
        }

        let time = as_chars()
            .skip(22)
            .take(8)
            .collect::<String>()
            .replace('.', ":");
        let Ok(parsed_time) = time.parse::<jiff::civil::Time>() else {
            return false;
        };

        let hex_skip = match backup_type {
            BackupType::Full => 53,
            BackupType::SqlOnly => 46,
        };

        let hex = as_chars().skip(hex_skip).take(8).collect::<String>();
        if !Self::is_hex(&hex, 8) {
            return false;
        }

        let valid = format!(
            "mealpedant_{a:02}-{b:02}-{c:02}_{d:02}.{e:02}.{f:02}_{backup_type}_{hex}.tar.age",
            a = parsed_date.year(),
            b = parsed_date.month(),
            c = parsed_date.day(),
            d = parsed_time.hour(),
            e = parsed_time.minute(),
            f = parsed_time.second()
        );
        valid == file_name
    }

    /// Parse a string, custom error if failure
    fn parse_string<'de, D>(deserializer: D, name: &str) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map_or(Err(de::Error::custom(name)), Ok)
    }

    /// Parse an i64, custom error if failure
    fn parse_i64<'de, D>(deserializer: D, name: &str) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        i64::deserialize(deserializer).map_or(Err(de::Error::custom(name)), Ok)
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

    /// Check email isn't empty, lowercase it, contains an '@' sign, and matches a 99.9% email regex
    pub fn email<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "email";
        let parsed = Self::parse_string(deserializer, name)?;

        Self::valid_email(&parsed).ok_or_else(|| de::Error::custom(name))
    }
    /// Check email isn't empty, lowercase it, contains an '@' sign, and matches a 99.9% email regex
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
        match Option::<String>::deserialize(deserializer)? {
            Some(x) => Ok(Some(Self::photo_name_hex(x.into_deserializer())?)),
            _ => Ok(None),
        }
    }

    /// Only allow photo names in the format: [ulid][1/0][1/0].jpg
    pub fn photo_name_hex<'de, D>(deserializer: D) -> Result<ij::PhotoName, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "photo_name";
        let parsed = Self::parse_string(deserializer, name)?;
        ij::PhotoName::try_from(parsed).map_err(|()| de::Error::custom(name))
    }

    /// Only allow ulid
    pub fn ulid<'de, D>(deserializer: D) -> Result<Ulid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "ulid";
        let parsed = Self::parse_string(deserializer, name)?;
        Ulid::from_string(&parsed).map_or_else(|_| Err(de::Error::custom(name)), Ok)
    }

    /// Only allow backup names in the format: mealpedant_yyyy-mm-dd_hh.mm.ss_[NAME]_[a-f0-9]{8}.tar.gz.age
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
        match Option::<String>::deserialize(deserializer)? {
            Some(x) => Ok(Some(Self::token(x.into_deserializer())?)),
            _ => Ok(None),
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
        match Option::<String>::deserialize(deserializer)? {
            Some(x) => Ok(Some(Self::password(x.into_deserializer())?)),
            _ => Ok(None),
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

        parsed.trim().parse::<IpAddr>().map_or(
            Self::valid_email(&parsed).map_or(Err(de::Error::custom(name)), |email| {
                Ok(LimitKey::Email(email))
            }),
            |ip| Ok(LimitKey::Ip(ip)),
        )
    }

    /// Only allows "Dave" or "Jack"
    pub fn person<'de, D>(deserializer: D) -> Result<Person, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "person";
        let parsed = Self::parse_string(deserializer, name)?;

        Person::try_from(parsed.as_str()).map_or(Err(de::Error::custom(name)), Ok)
    }

    /// Only allow strings, and trim said string
    pub fn trimmed<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = "trimmed";
        let parsed = Self::parse_string(deserializer, name)?;

        Ok(parsed.trim().to_owned())
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
            if let Some(date) = Self::valid_meal_date(year, month, day) {
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

    pub fn option_id<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<i64>::deserialize(deserializer)? {
            Some(x) => Ok(Some(Self::id(x.into_deserializer())?)),
            _ => Ok(None),
        }
    }
}

/// incoming_serializer
///
/// cargo watch -q -c -w src/ -x 'test incoming_serializer -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::pedantic, clippy::unwrap_used)]
mod tests {
    use serde::de::value::{Error as ValueError, SeqDeserializer, StringDeserializer};
    use serde::de::{IntoDeserializer, value::I64Deserializer};

    use rand::{Rng, distributions::Alphanumeric};

    use crate::helpers::gen_random_hex;
    use crate::servers::api_tests::{ANON_EMAIL, TEST_EMAIL};
    use crate::{C, S};

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
        test(S!("2014-01-01"));
        // invalid month
        test(S!("2020-20-01"));

        // invalid day
        test(S!("2020-01-40"));

        // missing parts
        test(S!("2020-01"));
        test(S!("01-2020-01"));
        test(S!("2020-30-04"));

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
        test(S!("2015-10-01"));

        // Before today
        test(S!("2019-01-02"));
    }

    #[test]
    fn incoming_serializer_photo() {
        // Valid
        assert!(IncomingDeserializer::parse_photo_name("J"));
        assert!(IncomingDeserializer::parse_photo_name("D"));

        // Invalid
        assert!(!IncomingDeserializer::parse_photo_name("j"));
        assert!(!IncomingDeserializer::parse_photo_name("d"));
        assert!(!IncomingDeserializer::parse_photo_name("1"));
        assert!(!IncomingDeserializer::parse_photo_name("p"));
    }

    #[test]
    fn incoming_serializer_ulid_valid() {
        let test = |ulid: &str| {
            let deserializer: StringDeserializer<ValueError> = ulid.to_owned().into_deserializer();
            let result = IncomingDeserializer::ulid(deserializer);
            assert!(result.is_ok());
        };
        test("01JQJ59DS59PESRRGD71994M12");
        test("01jqj59ds6wx5j268m4j0dx89b");
    }

    #[test]
    fn incoming_serializer_ulid_invalid() {
        let test = |ulid: &str| {
            let deserializer: StringDeserializer<ValueError> = ulid.to_owned().into_deserializer();
            let result = IncomingDeserializer::ulid(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "ulid");
        };
        test("01JQJ59DS59PESRRGD71994I12");
        test("01JQJ59DS59PESRRGD71994L12");
        test("01JQJ59DS59PESRRGD71994O12");
        test("01JQJ59DS59PESRRGD71994U12");
        test(r#"!@#$%^&*()-=_+[]{};':",<."#);
    }

    #[test]
    fn incoming_serializer_photo_name_invalid() {
        let test = |name: String| {
            let deserializer: StringDeserializer<ValueError> = name.into_deserializer();
            let result = IncomingDeserializer::photo_name_hex(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "photo_name");
        };

        test(S!("01b81v762g9aehkknq62qh116r02.jpg"));
        test(S!("01gqa3j0g0a15w8kcx6ab7466n20.jpg"));
        test(S!("01f7xnvn28vqngmfz0rcpz02zy22.jpg"));
        test(S!("01b81v762g9aehkknq62qh116r00.jpeg"));
        test(S!("0Lcvte2s406tkb3h10t86d14cy01.jpg"));
        test(S!("01b81v762g9aehkknq62qh116r00"));
        test(S!("01b81v762g9aehkknq62qh116r000jpg"));
        test(gen_random_hex(31));
        test(gen_random_hex(32));
        test(gen_random_hex(33));
    }

    #[test]
    fn incoming_serializer_photo_name_valid() {
        let test = |name: String| {
            let deserializer: StringDeserializer<ValueError> = name.into_deserializer();
            let result = IncomingDeserializer::photo_name_hex(deserializer);
            assert!(result.is_ok());
        };

        test(S!("01b81v762g9aehkknq62qh116r00.jpg"));
        test(S!("01gqa3j0g0a15w8kcx6ab7466n10.jpg"));
        test(S!("01f7xnvn28vqngmfz0rcpz02zy11.jpg"));
        test(S!("01cvte2s406tkb3h10t86d14cy01.jpg"));
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
            "xxxxxxxxxx_2022-07-03_03.01.00_LOGS_REDIS_SQL_8159c23e.tar.age",
        ));

        // missing suffix
        test(String::from(
            "2022-07-03_03.01.00_LOGS_REDIS_SQL_8159c23e.tar.xxx",
        ));
        // invalid date
        test(String::from(
            "mealpedant_1999-07-03_03.01.00_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.age",
        ));
        test(String::from(
            "mealpedant_2020-14-03_03.01.00_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.age",
        ));
        test(String::from(
            "mealpedant_2020-12-34_03.01.00_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.age",
        ));

        // invalid time
        test(String::from(
            "mealpedant_2022-07-03_24.01.00_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.age",
        ));
        test(String::from(
            "mealpedant_2022-07-03_03.63.00_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.age",
        ));
        test(String::from(
            "mealpedant_2022-07-03_03.01.72_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.age",
        ));

        // invalid name
        test(String::from(
            "mealpedant_2022-07-03_03.01.00_lOGS_REDIS_SQL_8159c23e.tar.age",
        ));
        test(String::from(
            "mealpedant_2022-07-03_03.01.00_LOGS_PHOTO_REDIS_SQL_8159c23e.tar.age",
        ));
        test(String::from(
            "mealpedant_2022-07-03_03.01.00_lOGS_REDIS_8159c23e.tar.age",
        ));

        // invalid hex
        test(String::from(
            "mealpedant_2022-07-03_03.01.00_LOGS_REDIS_SQL_8159c23.tar.age",
        ));
        test(String::from(
            "mealpedant_2022-07-03_03.01.00_LOGS_REDIS_SQL_8159c23K.tar.age",
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
            "mealpedant_2022-07-03_03.01.00_LOGS_REDIS_SQL_8159c23e.tar.age",
        ));
        test(String::from(
            "mealpedant_2022-07-03_03.01.00_LOGS_PHOTOS_REDIS_SQL_8159c23e.tar.age",
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

        test(S!("0"));
        test(S!("123"));

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
            let deserializer: StringDeserializer<ValueError> = C!(token).into_deserializer();
            let result = IncomingDeserializer::token(deserializer);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap().to_string(),
                token.replace(' ', "").to_uppercase()
            );
        };

        test(S!("111111"));
        test(S!("111 111"));
        test(S!(" 111 111 "));
        test(ran_token(false));
        test(S!("aaaaaabbbbbb1234"));
        test(S!("aaaaa abbbbbb1 234"));
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

        test(S!("12345"));
        test(S!("1234567"));
        test(S!("12345a"));
        test(S!("aaaabbbbccccdddd1"));
        test(S!("zzzzzzzzzzzzzzzz"));
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
        let p = || rand::thread_rng().gen_range(256..455);
        test(format!("{}.{}.{}.{}", p(), p(), p(), p()));

        test(S!("email@email"));
        test(S!("emailemail.com"));
        test(S!("@emailemail.com"));
        test(S!("127.127.127"));
        test(format!("127.127.127.{}", p()));
        test(S!(".127.127.127"));
    }

    #[test]
    fn incoming_serializer_limit_ok() {
        let test = |x: String| {
            let deserializer: StringDeserializer<ValueError> = C!(x).into_deserializer();
            let result = IncomingDeserializer::limit(deserializer);
            assert!(result.is_ok());
            match result.unwrap() {
                LimitKey::Email(e) => assert_eq!(e, x.to_lowercase()),
                LimitKey::Ip(i) => assert_eq!(i.to_string(), x),
            }
        };
        let p = || rand::thread_rng().gen_range(0..255);

        test(S!("email@email.com"));
        test(S!("email@email.com").to_uppercase());
        test(format!("{}@{}.{}", ran_s(10), ran_s(10), ran_s(3)));
        test(format!("{}@{}.{}", ran_s(10), ran_s(10), ran_s(3)).to_uppercase());
        test(format!(
            "{}@{}.{}.{}",
            ran_s(10),
            ran_s(10),
            ran_s(2),
            ran_s(2)
        ));

        test(S!("127.0.0.1"));
        test(S!("255.255.255.255"));
        test(format!("{}.{}.{}.{}", p(), p(), p(), p()));
    }

    #[test]
    fn incoming_serializer_trimmed_ok() {
        let deserializer: StringDeserializer<ValueError> = S!("abc ").into_deserializer();
        let result = IncomingDeserializer::trimmed(deserializer);
        assert!(result.is_ok());
        assert!(!result.unwrap().contains(' '));

        let deserializer: StringDeserializer<ValueError> = S!("abc\n").into_deserializer();
        let result = IncomingDeserializer::trimmed(deserializer);
        assert!(result.is_ok());
        assert!(!result.unwrap().contains('\n'));

        let deserializer: StringDeserializer<ValueError> = S!(" abc ").into_deserializer();
        let result = IncomingDeserializer::trimmed(deserializer);
        assert!(result.is_ok());
        assert!(!result.unwrap().contains(' '));

        let deserializer: StringDeserializer<ValueError> = S!("\nabc\n").into_deserializer();
        let result = IncomingDeserializer::trimmed(deserializer);
        assert!(result.is_ok());
        assert!(!result.unwrap().contains('\n'));

        let deserializer: StringDeserializer<ValueError> = S!(" abc\n").into_deserializer();
        let result = IncomingDeserializer::trimmed(deserializer);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.contains('\n'));
        assert!(!result.contains(' '));
    }

    #[test]
    fn incoming_serializer_email_ok() {
        let test = |email: String| {
            let deserializer: StringDeserializer<ValueError> = C!(email).into_deserializer();
            let result = IncomingDeserializer::email(deserializer);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), email.to_lowercase());
        };

        test(S!("email@email.com"));
        test(S!("email@email.com").to_uppercase());
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

        test(S!("emailemail.com"));
        test(S!(""));
        test(S!(" "));
        test(S!(" @ . "));
        test(S!(" @.com"));
        test(S!(" @ .com"));
        test(S!("email@"));
        test(S!("@email.com"));
        test(S!("email@email"));
        test(S!("email@email."));

        let deserializer: I64Deserializer<ValueError> = ran_n().into_deserializer();
        let result = IncomingDeserializer::email(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "email");
    }

    #[test]
    fn incoming_serializer_vec_email_ok() {
        let test = |x: Vec<String>| {
            let deserializer: SeqDeserializer<std::vec::IntoIter<String>, ValueError> =
                C!(x).into_deserializer();
            let result = IncomingDeserializer::vec_email(deserializer);
            assert!(result.is_ok());
            assert_eq!(result.as_ref().unwrap().len(), x.len());
            assert_eq!(result.unwrap()[0], x[0].to_lowercase());
        };

        test(vec![
            S!("email@email.com"),
            S!("email@abc.com"),
            ANON_EMAIL.to_string(),
            TEST_EMAIL.to_string(),
        ]);
        test(vec![
            S!("EMAIL@EMAIL.COM"),
            S!("email@abc.com"),
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
            S!("emailemail.com"),
            S!("email@abc.com"),
            ANON_EMAIL.to_string(),
            TEST_EMAIL.to_string(),
        ]);
        test(vec![
            S!("email@email"),
            S!("email@abc.com"),
            ANON_EMAIL.to_string(),
            TEST_EMAIL.to_string(),
        ]);
        test(vec![
            S!("email@.com"),
            S!("email@abc.com"),
            ANON_EMAIL.to_string(),
            TEST_EMAIL.to_string(),
        ]);
        test(vec![gen_random_hex(12)]);
    }

    #[test]
    fn incoming_serializer_name_ok() {
        let test = |name: String| {
            let deserializer: StringDeserializer<ValueError> = C!(name).into_deserializer();
            let result = IncomingDeserializer::name(deserializer);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), name.trim());
        };

        test(S!("aabbccd"));
        test(S!("sdfsdf "));
        test(S!("sdfsdf "));
        test(S!("sdfsdf bakaks"));
        test(S!(" sdfsdf bakaks "));
    }

    #[test]
    fn incoming_serializer_name_err() {
        let test = |name: String| {
            let deserializer: StringDeserializer<ValueError> = name.into_deserializer();
            let result = IncomingDeserializer::name(deserializer);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "name");
        };

        test(S!("invalid.name"));
        test(S!("invalid1name"));
        test(S!("John 1 Smith"));
        test(S!(""));
        test(S!(" "));
        test(S!("        "));

        let deserializer: I64Deserializer<ValueError> = ran_n().into_deserializer();
        let result = IncomingDeserializer::name(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "name");
    }

    #[test]
    fn incoming_serializer_password() {
        let test = |password: String| {
            let deserializer: StringDeserializer<ValueError> = C!(password).into_deserializer();
            let result = IncomingDeserializer::password(deserializer);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), password);
        };

        test(ran_s(12));

        test(S!("            "));

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

        test(S!(""));

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
            let deserializer: StringDeserializer<ValueError> = C!(invite).into_deserializer();
            let result = IncomingDeserializer::invite(deserializer);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), invite);
        };

        test(ran_s(12));

        test(S!("            "));

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

        test(S!(""));

        test(ran_s(11));

        test(ran_s(100));

        let deserializer: I64Deserializer<ValueError> = ran_n().into_deserializer();
        let result = IncomingDeserializer::invite(deserializer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "invite");
    }
}
