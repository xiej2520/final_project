use std::error::Error;

use reqwest::{RequestBuilder, Url};

#[derive(Clone, Debug)]
pub struct HttpClient {
    base_url: Url,
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new(base_url: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            base_url: Url::parse(base_url)?,
            client: reqwest::Client::new(),
        })
    }

    pub async fn get(&self, path: &str) -> Result<RequestBuilder, Box<dyn Error>> {
        let url = self.base_url.join(path)?;
        tracing::info!("GET {}", url.as_str()); 
        Ok(self.client.get(url.as_str()))
    }

    pub async fn post(&self, path: &str) -> Result<RequestBuilder, Box<dyn Error>> {
        let url = self.base_url.join(path)?;
        tracing::info!("POST {}", url.as_str());
        Ok(self.client.post(url.as_str()))
    }

    pub async fn put(&self, path: &str) -> Result<RequestBuilder, Box<dyn Error>> {
        let url = self.base_url.join(path)?;
        tracing::info!("PUT {}", url.as_str()); 
        Ok(self.client.put(url.as_str()))
    }

    pub async fn delete(&self, path: &str) -> Result<RequestBuilder, Box<dyn Error>> {
        let url = self.base_url.join(path)?;
        tracing::info!("DELETE {}", url.as_str()); 
        Ok(self.client.delete(url.as_str()))
    }

    pub async fn patch(&self, path: &str) -> Result<RequestBuilder, Box<dyn Error>> {
        let url = self.base_url.join(path)?;
        tracing::info!("PATCH {}", url.as_str());
        Ok(self.client.patch(url.as_str()))
    }
}
