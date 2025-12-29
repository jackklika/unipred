use super::Kalshi;
use super::kalshi_error::KalshiError;
use reqwest::{Method, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;

impl Kalshi {
    /// Helper to add auth headers and create a request builder.
    fn prepare_request(&self, method: Method, url: &Url) -> Result<reqwest::RequestBuilder, KalshiError> {
        let mut req = self.client.request(method.clone(), url.clone());
        
        if self.has_api_key() {
             let mut path_and_query = url.path().to_string();
             if let Some(query) = url.query() {
                 path_and_query.push('?');
                 path_and_query.push_str(query);
             }
             
             let headers = self.get_api_key_headers(method.as_str(), &path_and_query)?;
             req = req.headers(headers);
        } else if let Some(token) = self.get_user_token() {
            req = req.header("Authorization", token);
        }
        
        Ok(req)
    }

    pub async fn http_get<T: DeserializeOwned>(&self, url: Url) -> Result<T, KalshiError> {
        let req = self.prepare_request(Method::GET, &url)?;
        let resp = req.send().await?;
        self.process_response("GET", &url, resp).await
    }

    pub async fn http_post<B, T>(&self, url: Url, body: &B) -> Result<T, KalshiError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let mut req = self.prepare_request(Method::POST, &url)?;
        req = req.json(body);
        let resp = req.send().await?;
        self.process_response::<T>("POST", &url, resp).await
    }

    pub async fn http_delete<T: DeserializeOwned>(&self, url: Url) -> Result<T, KalshiError> {
        let req = self.prepare_request(Method::DELETE, &url)?;
        let resp = req.send().await?;
        self.process_response::<T>("DELETE", &url, resp).await
    }
    
    pub async fn http_put<B, T>(&self, url: Url, body: &B) -> Result<T, KalshiError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let mut req = self.prepare_request(Method::PUT, &url)?;
        req = req.json(body);
        let resp = req.send().await?;
        self.process_response::<T>("PUT", &url, resp).await
    }

    async fn process_response<T: DeserializeOwned>(
        &self,
        method: &str,
        url: &Url,
        resp: reqwest::Response,
    ) -> Result<T, KalshiError> {
        let status = resp.status();
        let bytes = resp.bytes().await.map_err(|e| KalshiError::InternalError(e.to_string()))?;

        if !status.is_success() {
             let body_str = String::from_utf8_lossy(&bytes);
             eprintln!("HTTP {} {} failed: status={}, body={}", method, url, status, body_str);
             
             return Err(KalshiError::InternalError(format!(
                "Non-success status {}. Body: {}",
                status,
                body_str
            )));
        }

        serde_json::from_slice::<T>(&bytes).map_err(|e| {
            let body_str = String::from_utf8_lossy(&bytes);
            KalshiError::InternalError(format!(
                "Deserialize error: {}. Body: {}",
                e,
                body_str
            ))
        })
    }

    pub fn build_url_with_params(
        &self,
        base_path: &str,
        params: Vec<(&str, String)>,
    ) -> Result<Url, KalshiError> {
        let base_url_str = format!("{}{}", self.base_url, base_path);
        Url::parse_with_params(&base_url_str, &params).map_err(|err| {
            KalshiError::InternalError(format!("URL Parse Error: {}", err))
        })
    }
    
    pub fn build_url(&self, base_path: &str) -> Result<Url, KalshiError> {
        let base_url_str = format!("{}{}", self.base_url, base_path);
        Url::parse(&base_url_str).map_err(|err| {
            KalshiError::InternalError(format!("URL Parse Error: {}", err))
        })
    }
}