use tracing::error;

use super::Email;

#[derive(Debug, Clone)]
pub struct CustomEmail {
    title: String,
    line_one: String,
    line_two: Option<String>,
    button: Option<EmailButton>,
}

impl CustomEmail {
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(
        title: String,
        line_one: String,
        line_two: Option<String>,
        hyper_link: Option<String>,
        button_text: Option<String>,
    ) -> Self {
        let button = if let (Some(link), Some(text)) = (hyper_link, button_text) {
            Some(EmailButton { link, text })
        } else {
            None
        };

        Self {
            title,
            line_one,
            line_two,
            button,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EmailTemplate {
    /// secret, will handle to secret-to-link in enum
    Verify(String),
    AccountLocked,
    PasswordChanged,
    /// secret, will handle to secret-to-link in enum
    PasswordResetRequested(String),
    TwoFAEnabled,
    TwoFADisabled,
    TwoFABackupEnabled,
    TwoFABackupDisabled,
    Custom(CustomEmail),
}

impl EmailTemplate {
    pub fn get_fallback(&self) -> String {
        format!(
            "{},\n{}\n{}\n",
            self.get_subject(),
            self.get_line_one(),
            self.get_line_two().unwrap_or_default()
        )
    }

    pub fn get_subject(&self) -> String {
        match self {
            Self::Verify(_) => "Verify Email Address".to_owned(),
            Self::AccountLocked => "Security Alert".to_owned(),
            Self::PasswordChanged => "Password Changed".to_owned(),
            Self::PasswordResetRequested(_) => "Password Reset Requested".to_owned(),
            Self::TwoFAEnabled => "Two-Factor Enabled".to_owned(),
            Self::TwoFADisabled => "Two-Factor Disabled".to_owned(),
            Self::TwoFABackupEnabled => "Two-Factor Backup Enabled".to_owned(),
            Self::TwoFABackupDisabled => "Two-Factor Backup Disabled".to_owned(),
            Self::Custom(custom_email) => custom_email.title.clone(),
        }
    }

    pub fn get_button(&self) -> Option<EmailButton> {
        match self {
            Self::PasswordResetRequested(link) => Some(EmailButton {
                link: format!("/user/reset/{link}"),
                text: "RESET PASSWORD".to_owned(),
            }),
            Self::Verify(link) => Some(EmailButton {
                link: format!("/user/verify/{link}"),
                text: "VERIFY EMAIL ADDRESS".to_owned(),
            }),
            Self::TwoFAEnabled => Some(EmailButton {
                link: String::from("/user/settings/"),
                text: "GENERATE BACKUP CODES".to_owned(),
            }),
            Self::Custom(custom_email) => custom_email.button.as_ref().map(|button| EmailButton {
                link: button.link.clone(),
                text: button.text.clone(),
            }),
            _ => None,
        }
    }

    pub fn get_line_one(&self) -> String {
        match self {
			Self::Custom(custom_email) => custom_email.line_one.clone(),
            Self::AccountLocked => "Due to multiple failed login attempts your account has been locked.".to_owned(),
            Self::PasswordChanged => "The password for your Meal Pedant account has been changed.".to_owned(),
            Self::PasswordResetRequested(_) => "This password reset link will only be valid for one hour".to_owned(),
            Self::TwoFABackupDisabled => "You have removed the Two-Factor Authentication backup codes for your Meal Pedant account. New backup codes can be created at any time from the user settings page.".to_owned(),
            Self::TwoFABackupEnabled => "You have created Two-Factor Authentication backup codes for your Meal Pedant account. The codes should be stored somewhere secure".to_owned(),
            Self::TwoFADisabled => "You have disabled Two-Factor Authentication for your Meal Pedant account.".to_owned(),
            Self::TwoFAEnabled => "You have enabled Two-Factor Authentication for your Meal Pedant account, it is recommended to create and save backup codes, these can be generated in the user settings area.".to_owned(),
            Self::Verify(_) => "Welcome to Meal Pedant, before you start we just need you to verify this email address.".to_owned(),
        }
    }

    pub fn get_line_two(&self) -> Option<String> {
        let contact_support =
            "If you did not enable this setting, please contact support as soon as possible."
                .to_owned();
        match self {
            Self::TwoFAEnabled | Self::TwoFADisabled | Self::PasswordChanged => {
                Some(contact_support)
            }
            // Self::TwoFADisabled => Some(contact_support),
            // Self::PasswordChanged => Some(contact_support),
            Self::AccountLocked => {
                Some("Please contact support in order to unlock your account".to_owned())
            }
            Self::PasswordResetRequested(_) => Some(
                "If you did not request a password reset then please ignore this email".to_owned(),
            ),
            Self::Custom(custom_email) => custom_email.line_two.as_ref().cloned(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmailButton {
    link: String,
    text: String,
}

fn create_template(input: &Email, domain: &str) -> String {
    let full_domain = format!("https://www.{domain}");

    let mut template = format!(
        r"
<mjml>
	<mj-head>
		<mj-title>
			{title}
		</mj-title>
		<mj-attributes>
			<mj-all font-family='Open Sans, Tahoma, Arial, sans-serif'></mj-all>
		</mj-attributes>
		<mj-style inline='inline'>
			.link-nostyle {{ color: inherit; text-decoration: none }}
		</mj-style>
	</mj-head>
	<mj-body background-color='#929892'>
		<mj-section padding-top='30px'></mj-section>
		<mj-section background-color='#212121' border-radius='10px' text-align='center'>
		<mj-column vertical-align='middle' width='100%'>
			<mj-image width='320px' src='https://static.mealpedant.com/email_header.png'></mj-image>
			<mj-spacer height='15px'></mj-spacer>
			<mj-text line-height='1.2' color='#ffffff' font-weight='500' font-size='20px'>
				Hi {name},
			</mj-text>
			<mj-text line-height='1.2' color='#ffffff' font-weight='500' font-size='20px'>
				{line_one}
			</mj-text>",
        title = input.template.get_subject(),
        name = input.name,
        line_one = input.template.get_line_one()
    );

    if let Some(line_two) = input.template.get_line_two() {
        let line_two_section = format!(
            r"
			<mj-text line-height='1.2' color='#ffffff' font-weight='500' font-size='20px'>
				{line_two}
			</mj-text>"
        );
        template.push_str(&line_two_section);
    }
    if let Some(mut button) = input.template.get_button() {
        // This is dirty, need to come up with a better solution
        if !button.link.starts_with("http") {
            button.link = format!("{full_domain}{}", button.link);
        }

        let button_section = format!(
            r"
			<mj-button href='{link}' border-radius='10px' background-color='#7ca1b2' font-size='20px'>
				{text}
			</mj-button>
			<mj-text line-height='1.2' align='center' color='#ffffff' font-size='13px'>
				or copy and paste this address into the browser address bar
			</mj-text>
			<mj-text line-height='1.2' align='center' color='#ffffff' font-size='13px'>
				<a class='link-nostyle' href='{link}'>
					{link}
				</a>
			</mj-text>",
            link = button.link,
            text = button.text
        );
        template.push_str(&button_section);
    }
    let end_section = format!(
        r"
		</mj-column>
		<mj-column vertical-align='middle' width='100%' padding-top='40px'>
			<mj-text line-height='1.2' align='center' color='#ffffff' font-size='12px'>
				This is an automated email - replies sent to this email address are not read
				<br></br>
				<a class='link-nostyle' href='{full_domain}'>
					{full_domain}
				</a>
				<br></br>
				© 2015 -
			</mj-text>
		</mj-column>
	</mj-section>
	<mj-section padding-bottom='30px'></mj-section>
</mj-body>
</mjml>"
    );
    template.push_str(&end_section);
    template
}

/// Use a EmailTemplate to create a parsed mjml html string
#[allow(clippy::cognitive_complexity)]
pub fn create_html_string(input: &Email) -> Option<String> {
    let template = create_template(input, input.emailer.get_domain());

    match mrml::parse(template) {
        Ok(root) => {
            let opts = mrml::prelude::render::Options::default();
            match root.render(&opts) {
                Ok(email_string) => Some(email_string),
                Err(e) => {
                    error!("{:?}", e);
                    error!("email render error");
                    None
                }
            }
        }
        Err(e) => {
            error!("{:?}", e);
            error!("mrml parsing error");
            None
        }
    }
}

/// cargo watch -q -c -w src/ -x 'test emailer_template -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery, clippy::unwrap_used)]
mod tests {

    use crate::{emailer::EmailerEnv, parse_env};

    use super::*;

    #[test]
    fn emailer_template_create_template() {
        let app_env = parse_env::AppEnv::get_env();
        let emailer = EmailerEnv::new(&app_env);

        let secret = "test_secret";

        let create_input = |template: EmailTemplate| {
            Email::new("john smith", "email@example.com", template, &emailer)
        };

        let input = create_input(EmailTemplate::AccountLocked);
        let result = create_template(&input, &app_env.domain);
        //title
        assert!(result.contains("Security Alert"));
        // name
        assert!(result.contains("Hi john smith,"));
        // line one
        assert!(
            result.contains("Due to multiple failed login attempts your account has been locked.")
        );
        // no button
        assert!(!result.contains("<mj-button"));
        assert!(!result.contains("or copy and paste this address into the browser address bar"));

        let input = create_input(EmailTemplate::PasswordChanged);
        let result = create_template(&input, &app_env.domain);
        assert!(result.contains("Hi john smith,"));
        assert!(result.contains("The password for your Meal Pedant account has been changed."));
        assert!(result.contains(
            "If you did not enable this setting, please contact support as soon as possible."
        ));
        assert!(!result.contains("<mj-button"));
        assert!(!result.contains("or copy and paste this address into the browser address bar"));

        let input = create_input(EmailTemplate::PasswordResetRequested(secret.to_owned()));
        let result = create_template(&input, &app_env.domain);
        // title
        assert!(result.contains("Password Reset Requested"));
        // name
        assert!(result.contains("Hi john smith,"));
        // line one
        assert!(result.contains("This password reset link will only be valid for one hour"));
        // line two
        assert!(result
            .contains("If you did not request a password reset then please ignore this email"));
        // button
        assert!(result.contains("<mj-button"));
        assert!(result.contains("or copy and paste this address into the browser address bar"));
        let link = format!(
            "<a class='link-nostyle' href='https://www.{}/user/reset/test_secret'>",
            app_env.domain
        );

        assert!(result.contains(&link));
        assert!(result.contains("RESET PASSWORD"));

        let input = create_input(EmailTemplate::TwoFABackupEnabled);
        let result = create_template(&input, &app_env.domain);
        // title
        assert!(result.contains("Two-Factor Backup Enabled"));
        // name
        assert!(result.contains("Hi john smith,"));
        // line one
        assert!(result.contains("You have created Two-Factor Authentication backup codes for your Meal Pedant account. The codes should be stored somewhere secure"));
        // button
        assert!(!result.contains("<mj-button"));
        assert!(!result.contains("or copy and paste this address into the browser address bar"));

        let input = create_input(EmailTemplate::TwoFABackupDisabled);
        let result = create_template(&input, &app_env.domain);
        // title
        assert!(result.contains("Two-Factor Backup Disabled"));
        // name
        assert!(result.contains("Hi john smith,"));
        // line one
        assert!(result.contains("You have removed the Two-Factor Authentication backup codes for your Meal Pedant account. New backup codes can be created at any time from the user settings page."));
        // button
        assert!(!result.contains("<mj-button"));
        assert!(!result.contains("or copy and paste this address into the browser address bar"));

        let input = create_input(EmailTemplate::TwoFAEnabled);
        let result = create_template(&input, &app_env.domain);
        // title
        assert!(result.contains("Two-Factor Enabled"));
        // name
        assert!(result.contains("Hi john smith,"));
        // line one
        assert!(result.contains("You have enabled Two-Factor Authentication for your Meal Pedant account, it is recommended to create and save backup codes, these can be generated in the user settings area."));
        // button
        assert!(result.contains(
            "If you did not enable this setting, please contact support as soon as possible."
        ));
        assert!(result.contains("<mj-button"));
        assert!(result.contains("or copy and paste this address into the browser address bar"));
        let link = format!(
            "<a class='link-nostyle' href='https://www.{}/user/settings/'>",
            app_env.domain
        );
        assert!(result.contains(&link));
        assert!(result.contains("GENERATE BACKUP CODES"));

        let input = create_input(EmailTemplate::TwoFADisabled);
        let result = create_template(&input, &app_env.domain);
        // title
        assert!(result.contains("Two-Factor Disabled"));
        // name
        assert!(result.contains("Hi john smith,"));
        // line one
        assert!(result
            .contains("You have disabled Two-Factor Authentication for your Meal Pedant account"));
        // button
        assert!(result.contains(
            "If you did not enable this setting, please contact support as soon as possible."
        ));
        assert!(!result.contains("<mj-button"));
        assert!(!result.contains("or copy and paste this address into the browser address bar"));

        let input = create_input(EmailTemplate::Verify(secret.to_string()));
        let result = create_template(&input, &app_env.domain);
        // title
        assert!(result.contains("Verify Email Address"));
        // name
        assert!(result.contains("Hi john smith,"));
        // line one
        assert!(result.contains("Welcome to Meal Pedant, before you start we just need you to verify this email address."));
        // button
        assert!(result.contains("<mj-button"));
        assert!(result.contains("or copy and paste this address into the browser address bar"));
        let link = format!(
            "<a class='link-nostyle' href='https://www.{}/user/verify/{}'>",
            app_env.domain, secret
        );
        assert!(result.contains(&link));
        assert!(result.contains("VERIFY EMAIL ADDRESS"));
    }

    #[test]
    fn emailer_template() {
        let app_env = parse_env::AppEnv::get_env();
        let emailer = &EmailerEnv::new(&app_env);

        let secret = "test_reset_secret";

        let input = Email::new(
            "john smith",
            "email@example.com",
            EmailTemplate::PasswordResetRequested(secret.to_owned()),
            emailer,
        );
        let result = create_html_string(&input);
        assert!(result.is_some());

        let result = result.unwrap();

        assert!(result.starts_with("<!doctype html><html xmlns=\"http://www.w3.org/1999/xhtml\""));
        let link = format!(
            "href=\"https://www.{}/user/reset/{}\"",
            app_env.domain, secret
        );
        assert!(result.contains(&link));
    }
}
