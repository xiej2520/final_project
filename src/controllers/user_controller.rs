use std::{
    collections::{
        hash_map::{DefaultHasher, Entry},
        HashMap,
    },
    hash::{Hash, Hasher},
};

use chrono::Utc;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use reqwest::Url;

#[derive(Debug, Default)]
pub struct UserStore {
    users: HashMap<String, User>,
    usernames: HashMap<String, String>,
}

#[derive(Debug)]
pub struct User {
    username: String,
    password_hash: u64,
    email: String,
    key: String,
    enabled: bool,
}

impl UserStore {
    pub fn get_user(&self, username: &str) -> Option<&User> {
        self.users.get(username)
    }

    pub fn verify_user(&self, email: &str, key: &str) -> Result<(), String> {
        match self.usernames.get(email) {
            Some(username) => match self.users.get_mut(username) {
                Some(user) => user.enable(key),
                None => Err("User not found".to_owned()),
            },
            None => Err("Email not found".to_owned()),
        }
    }

    pub fn add_user(&mut self, user: User, relay_ip: [u8; 4], relay_port: u16) -> Result<(), String> {
        let User {
            username, email, ..
        } = &user;
        match self.users.entry(username.clone()) {
            Entry::Occupied(_) => Err(format!("User named '{username}' already exists")),
            Entry::Vacant(vacant_user) => match self.usernames.entry(email.clone()) {
                Entry::Occupied(_) => Err(format!("Email '{email}' already registered")),
                Entry::Vacant(vacant_email) => {
                    match user.send_email(relay_ip, relay_port).await {
                        Ok(link) => {
                            vacant_email.insert(username.clone());
                            vacant_user.insert(user);
                            Ok(link)
                        }
                        Err(err) => Err(err),
                    }
                }
            },
        }
    }
}

impl User {
    pub fn new(username: &str, password: &str, email: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let password_hash = hasher.finish();
        Utc::now().hash(&mut hasher);
        let key = format!("{:0width$x}", hasher.finish(), width = 16);
        Self {
            username: username.to_owned(),
            password_hash,
            email: email.to_owned(),
            key: key.clone(),
            enabled: false,
        }
    }

    pub async fn send_email(&self, relay_ip: [u8; 4], relay_port: u16) -> Result<String, String> {
        // escape special characters in email, including '+'
        let verification_link = Url::parse_with_params(
            format!("http://{}/api/verify", CONFIG.domain).as_str(),
            &[("email", self.email.as_str()), ("key", self.key.as_str())],
        )
        .map_err(|e| e.to_string())?
        .to_string();

        if cfg!(feature = "disable_email") {
            return Ok(verification_link);
        }

        let email = Message::builder()
            .from(
                "final_project <final_project@cse356.compas.cs.stonybrook.edu>"
                    .parse()
                    .unwrap(),
            )
            .to(self.email.parse().unwrap())
            .subject(verification_link.clone())
            .body(verification_link.clone())
            .unwrap();

        let relay_ip_string = relay_ip
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(".");
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&relay_ip_string)
            .port(relay_port)
            .build();
        match mailer.send(email).await {
            Ok(_) => Ok(verification_link),
            Err(err) => Err(format!(
                "Failed to send email: {err}, verification link {verification_link}"
            )),
        }
    }

    pub fn matches_password(&self, password: &str) -> bool {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        self.password_hash == hasher.finish()
    }

    pub fn enable(&mut self, key: &str) -> Result<(), String> {
        if self.key != key {
            return Err("Invalid key".to_owned());
        } else if self.enabled {
            return Err("User already enabled".to_owned());
        }
        self.enabled = true;
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
