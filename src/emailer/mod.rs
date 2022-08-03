#![allow(unused)]
mod template;

use crate::parse_env::AppEnv;
use anyhow::Result;

use lettre::{
    address::AddressError,
    message::{header, Mailbox, MultiPart, SinglePart},
    transport::smtp::{authentication::Credentials, AsyncSmtpTransportBuilder},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
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
    production: bool,
}

impl EmailerEnv {
    pub fn new(app_env: &AppEnv) -> Self {
        Self {
            domain: app_env.domain.clone(),
            from_address: app_env.email_from_address.clone(),
            host: app_env.email_host.clone(),
            name: app_env.email_name.clone(),
            password: app_env.email_password.clone(),
            port: app_env.email_port,
            production: app_env.production,
        }
    }
    fn get_from_mailbox(&self) -> Result<Mailbox, AddressError> {
        format!("{} <{}>", self.name, self.from_address).parse::<Mailbox>()
    }

    fn get_credentials(&self) -> Credentials {
        Credentials::new(self.from_address.clone(), self.password.clone())
    }

    fn get_mailer(&self) -> Result<AsyncSmtpTransportBuilder, lettre::transport::smtp::Error> {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&self.host)
    }

    fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_domain(&self) -> &str {
        self.domain.as_str()
    }

    pub fn get_production(&self) -> bool {
        self.production
    }
}

#[derive(Debug, Clone)]
pub struct Email {
    name: String,
    email_address: String,
    template: EmailTemplate,
    emailer: EmailerEnv,
}

// make an emailer that has al the secrets?
// then don't need to pass on at any other bits

impl Email {
    pub fn new(
        name: &str,
        email_address: &str,
        template: EmailTemplate,
        email_env: &EmailerEnv,
    ) -> Self {
        Self {
            name: name.to_owned(),
            email_address: email_address.to_owned(),
            template,
            emailer: email_env.clone(),
        }
    }

    /// Send email on it's own thread, as not to slow down any api responses
    /// And assume that it succeeds, and inform the user that it was succeeded
    pub fn send(&self) {
        tokio::spawn(Self::_send(self.clone()));
    }

    /// Handle all errors in this function, just trace on any issues
    /// not(release) instead?
    #[cfg(test)]
	#[allow(clippy::unwrap_used)]
    async fn _send(email: Email) {
        let to_box = format!("{} <{}>", email.name, email.email_address).parse::<Mailbox>();
        if let (Ok(from), Ok(to)) = (email.emailer.get_from_mailbox(), to_box) {
            let subject = email.template.get_subject();
            if let Some(html_string) = create_html_string(&email) {
                // let fallback = format!("{}\n{}\n{}", email.template.get_subject();
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
                                    .body(html_string.clone()),
                            ),
                    );

                if let Ok(message) = message_builder {
                    std::fs::write("/dev/shm/email_headers.txt", message.headers().to_string())
                        .unwrap();
                    std::fs::write("/dev/shm/email_body.txt", html_string).unwrap();
                    info!("Would be sending email if on production");
                } else {
                    error!("unable to build message with Message::builder");
                }
            }
        } else {
            error!("unable to parse from_box or to_box");
        }
    }

    /// Handle all errors in this function, just trace on any issues
    #[cfg(not(test))]
	#[allow(clippy::unwrap_used)]
    async fn _send(email: Email) {
        let to_box = format!("{} <{}>", email.name, email.email_address).parse::<Mailbox>();
        if let (Ok(from), Ok(to)) = (email.emailer.get_from_mailbox(), to_box) {
            let subject = email.template.get_subject();
            if let Some(html_string) = create_html_string(&email) {
                let message_builder = Message::builder()
                    .from(from)
                    .to(to)
                    .subject(subject.clone())
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
                                    .body(html_string.clone()),
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
                        std::fs::write("/dev/shm/email_headers.txt", message.headers().to_string())
                            .unwrap();
                        std::fs::write("/dev/shm/email_body.txt", html_string).unwrap();
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
#[allow(clippy::unwrap_used)]
mod tests {

    use super::*;
    use crate::parse_env;

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
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;

        let result = std::fs::read_to_string("/dev/shm/email_body.txt").unwrap();
        assert!(result.starts_with("<!doctype html><html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:v=\"urn:schemas-microsoft-com:vml\" xmlns:o=\"urn:schemas-microsoft-com:office:office\"><head><title>"));
        assert!(result.contains("john smith"));

        let result = std::fs::read_to_string("/dev/shm/email_headers.txt").unwrap();
        assert!(result.contains("From: \"Meal Pedant\" <no-reply@mealpedant.com>"));
        assert!(result.contains("To: \"john smith\" <email@example.com>"));
        assert!(result.contains("Subject: Password Changed"));

        std::fs::remove_file("/dev/shm/email_headers.txt").unwrap();
        std::fs::remove_file("/dev/shm/email_body.txt").unwrap();
    }
}
