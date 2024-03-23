use std::{
    collections::{
        hash_map::{DefaultHasher, Entry},
        HashMap,
    },
    hash::{Hash, Hasher},
};

use chrono::Utc;

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

    pub fn get_user_mut(&mut self, username: &str) -> Option<&mut User> {
        self.users.get_mut(username)
    }

    pub fn get_user_by_email(&self, email: &str) -> Option<&User> {
        match self.usernames.get(email) {
            Some(username) => self.users.get(username),
            None => None,
        }
    }

    pub fn get_user_by_email_mut(&mut self, email: &str) -> Option<&mut User> {
        match self.usernames.get(email) {
            Some(username) => self.users.get_mut(username),
            None => None,
        }
    }

    pub fn add_user(&mut self, user: User) -> Result<(), String> {
        let User {
            username, email, ..
        } = &user;
        match self.users.entry(username.clone()) {
            Entry::Occupied(_) => Err(format!("User named '{}' already exists", username)),
            Entry::Vacant(vacant_user) => match self.usernames.entry(email.clone()) {
                Entry::Occupied(_) => Err(format!("Email '{}' already registered", username)),
                Entry::Vacant(vacant_email) => {
                    vacant_email.insert(username.clone());
                    vacant_user.insert(user);
                    Ok(())
                }
            },
        }
    }
}

impl User {
    pub fn new(username: &str, password: &str, email: &str) -> (Self, String) {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let password_hash = hasher.finish();
        Utc::now().hash(&mut hasher);
        let key = format!("{:0width$x}", hasher.finish(), width = 16);
        (
            Self {
                username: username.to_owned(),
                password_hash,
                email: email.to_owned(),
                key: key.clone(),
                enabled: false,
            },
            key,
        )
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
