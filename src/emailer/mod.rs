#![allow(unused)]
mod template;

use crate::{
    C,
    parse_env::{AppEnv, RunMode},
};

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    address::AddressError,
    message::{Mailbox, MultiPart, SinglePart, header},
    transport::smtp::{AsyncSmtpTransportBuilder, authentication::Credentials},
};
use tracing::{error, info, trace};

use self::template::create_html_string;

pub use self::template::{CustomEmail, EmailTemplate};

/// Store secrets in here, and then use getters methods to get them
#[derive(Debug, Clone)]
pub struct EmailerEnv {
    domain: String,
    from_address: String,
    host: String,
    name: String,
    password: String,
    port: u16,
    run_mode: RunMode,
}

impl EmailerEnv {
    pub fn new(app_env: &AppEnv) -> Self {
        Self {
            domain: C!(app_env.domain),
            from_address: C!(app_env.email_from_address),
            host: C!(app_env.email_host),
            name: C!(app_env.email_name),
            password: C!(app_env.email_password),
            port: app_env.email_port,
            run_mode: app_env.run_mode,
        }
    }
    fn get_from_mailbox(&self) -> Result<Mailbox, AddressError> {
        format!("{} <{}>", self.name, self.from_address).parse::<Mailbox>()
    }

    fn get_credentials(&self) -> Credentials {
        Credentials::new(C!(self.from_address), C!(self.password))
    }

    fn get_mailer(&self) -> Result<AsyncSmtpTransportBuilder, lettre::transport::smtp::Error> {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&self.host)
    }

    const fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_domain(&self) -> &str {
        self.domain.as_str()
    }

    pub const fn get_production(&self) -> bool {
        self.run_mode.is_production()
    }
}

#[derive(Debug, Clone)]
pub struct Email {
    name: String,
    address: String,
    template: EmailTemplate,
    emailer: EmailerEnv,
}

// make an emailer that has al the secrets?
// then don't need to pass on at any other bits

impl Email {
    pub fn new(name: &str, address: &str, template: EmailTemplate, email_env: &EmailerEnv) -> Self {
        Self {
            name: name.to_owned(),
            address: address.to_owned(),
            template,
            emailer: C!(email_env),
        }
    }

    /// Send email on it's own thread, as not to slow down any api responses
    /// And assume that it succeeds, and inform the user that it was succeeded
    pub fn send(&self) {
        tokio::spawn(Self::_send(C!(self)));
    }

    /// Handle all errors in this function, just trace on any issues
    #[cfg(test)]
    #[expect(clippy::unwrap_used, clippy::unused_async)]
    async fn _send(email: Self) {
        use crate::tmp_file;

        let to_box = format!("{} <{}>", email.name, email.address).parse::<Mailbox>();
        if let (Ok(from), Ok(to)) = (email.emailer.get_from_mailbox(), to_box) {
            let subject = email.template.get_subject();
            if let Some(html_string) = create_html_string(&email) {
                let message_builder = Message::builder()
                    .from(from)
                    .to(to)
                    .subject(subject)
                    .multipart(
                        MultiPart::alternative() // This is composed of two parts.
                            .singlepart(
                                SinglePart::builder()
                                    .header(header::ContentType::TEXT_PLAIN)
                                    .body(email.template.get_fallback()),
                            )
                            .singlepart(
                                SinglePart::builder()
                                    .header(header::ContentType::TEXT_HTML)
                                    .body(C!(html_string)),
                            ),
                    );

                message_builder.map_or_else(
                    |_| {
                        error!("unable to build message with Message::builder");
                    },
                    |message| {
                        std::fs::write(
                            tmp_file!("email_headers.txt"),
                            message.headers().to_string(),
                        )
                        .unwrap();
                        std::fs::write(tmp_file!("email_body.txt"), html_string).unwrap();
                        info!("Would be sending email if on production");
                    },
                );
            }
        } else {
            error!("unable to parse from_box or to_box");
        }
    }

    /// Handle all errors in this function, just trace on any issues
    #[cfg(not(test))]
    #[expect(clippy::unwrap_used)]
    async fn _send(email: Self) {
        let to_box = format!("{} <{}>", email.name, email.address).parse::<Mailbox>();
        if let (Ok(from), Ok(to)) = (email.emailer.get_from_mailbox(), to_box) {
            let subject = email.template.get_subject();
            if let Some(html_string) = create_html_string(&email) {
                let message_builder = Message::builder()
                    .from(from)
                    .to(to)
                    .subject(C!(subject))
                    .multipart(
                        MultiPart::alternative()
                            .singlepart(
                                SinglePart::builder()
                                    .header(header::ContentType::TEXT_PLAIN)
                                    .body(email.template.get_fallback()),
                            )
                            .singlepart(
                                SinglePart::builder()
                                    .header(header::ContentType::TEXT_HTML)
                                    .body(C!(html_string)),
                            ),
                    );

                if let Ok(message) = message_builder {
                    // Only send emails on production, should probably copy + paste for testing only as well
                    if email.emailer.get_production() {
                        let creds = email.emailer.get_credentials();
                        match email.emailer.get_mailer() {
                            Ok(sender) => {
                                let transport = sender
                                    .credentials(creds)
                                    .port(email.emailer.get_port())
                                    .build();

                                match transport.send(message).await {
                                    Ok(_) => trace!("Email sent successfully!"),
                                    Err(e) => {
                                        error!(%e);
                                        error!("mailer.send error");
                                    }
                                }
                            }
                            Err(e) => {
                                error!(%e);
                                info!("Mailer relay error");
                            }
                        }
                    } else {
                        std::fs::write(
                            "/ramdrive/mealpedant/email_headers.txt",
                            message.headers().to_string(),
                        )
                        .unwrap();
                        std::fs::write("/ramdrive/mealpedant/email_body.txt", html_string).unwrap();
                        info!("Would be sending email if on production");
                    }
                } else {
                    error!("unable to build message with Message::builder");
                }
            }
        } else {
            error!("unable to parse from_box or to_box");
        }
    }
}

/// cargo watch -q -c -w src/ -x 'test emailer_mod -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::pedantic, clippy::unwrap_used)]
mod tests {

    use super::*;
    use crate::{parse_env, sleep, tmp_file};

    /// Make sure emailer sends correctly, just save onto disk and check against that, rather than sending actual email!
    #[tokio::test]
    async fn emailer_mod_send_to_disk() {
        let app_env = parse_env::AppEnv::get_env();
        let emailer = EmailerEnv::new(&app_env);

        let email = Email::new(
            "john smith",
            "email@example.com",
            EmailTemplate::PasswordChanged,
            &emailer,
        );
        email.send();

        // Need to sleep, as the email.send() function spawns onto it's own thread, 1ms should be enough to do everything it needs to
        sleep!(1);

        let result = std::fs::read_to_string(tmp_file!("email_body.txt")).unwrap();
        // assert!(result.starts_with("<!doctype html><html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:v=\"urn:schemas-microsoft-com:vml\" xmlns:o=\"urn:schemas-microsoft-com:office:office\"><head><title>"));
        // assert!(result.contains("john smith"));

        // let result = std::fs::read_to_string(tmp_file!("email_headers.txt")).unwrap();
        // assert!(result.contains("From: \"Meal Pedant\" <no-reply@mealpedant.com>"));
        // assert!(result.contains("To: \"john smith\" <email@example.com>"));
        // assert!(result.contains("Subject: Password Changed"));

        // std::fs::remove_file(tmp_file!("email_headers.txt")).unwrap();
        // std::fs::remove_file(tmp_file!("email_body.txt")).unwrap();
    }
}
