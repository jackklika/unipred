use super::Kalshi;
use super::kalshi_error::*;

use rsa::{pkcs1::DecodeRsaPrivateKey, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use super::crypto::sign_pss_text;
use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

impl<'a> Kalshi {
    /// Asynchronously logs a user into the Kalshi exchange.
    ///
    /// This method sends a POST request to the Kalshi exchange's login endpoint with the user's credentials.
    /// On successful authentication, it updates the current session's token and member ID.
    ///
    /// # Arguments
    /// * `user` - A string slice representing the user's email.
    /// * `password` - A string slice representing the user's password.
    ///
    /// # Returns
    /// - `Ok(())`: Empty result indicating successful login.
    /// - `Err(KalshiError)`: Error in case of a failure in the HTTP request or response parsing.
    ///
    /// # Example
    /// ```
    /// kalshi_instance.login("johndoe@example.com", "example_password").await?;
    /// ```
    pub async fn login(&mut self, user: &str, password: &str) -> Result<(), KalshiError> {
        let login_url: &str = &format!("{}/login", self.base_url.to_string());

        let login_payload = LoginPayload {
            email: user.to_string(),
            password: password.to_string(),
        };

        let result: LoginResponse = self
            .client
            .post(login_url)
            .json(&login_payload)
            .send()
            .await?
            .json()
            .await?;

        self.curr_token = Some(format!("Bearer {}", result.token));
        self.member_id = Some(result.member_id.clone());

        // Clear API Key auth
        self.private_key = None;
        self.api_key_id = None;

        return Ok(());
    }

    /// Asynchronously authenticates a user with the Kalshi exchange using an API key.
    ///
    /// This method reads a private key from a file, and stores it along with the key ID
    /// for signing future requests.
    ///
    /// # Arguments
    /// * `key_id` - A string slice representing the API key ID.
    /// * `private_key_path` - A string slice representing the path to the PEM-encoded private key file.
    ///
    /// # Returns
    /// - `Ok(())`: Empty result indicating successful authentication setup.
    /// - `Err(KalshiError)`: Error in case of a failure in reading or parsing the key.
    ///
    /// # Example
    /// ```
    /// kalshi_instance.login_apikey("your_key_id", "/path/to/your/private.key").await?;
    /// ```
    pub async fn login_apikey(
        &mut self,
        key_id: &str,
        private_key_path: &str,
    ) -> Result<(), KalshiError> {
        let pem_str = fs::read_to_string(private_key_path)?;
        let private_key = RsaPrivateKey::from_pkcs1_pem(&pem_str)?;

        self.private_key = Some(Arc::new(private_key));
        self.api_key_id = Some(key_id.to_string());

        // Clear email/password auth
        self.curr_token = None;
        self.member_id = None;

        Ok(())
    }

    /// Asynchronously logs a user out of the Kalshi exchange.
    ///
    /// Sends a POST request to the Kalshi exchange's logout endpoint. This method
    /// should be called to properly terminate the session initiated by `login`.
    ///
    /// # Returns
    /// - `Ok(())`: Empty result indicating successful logout.
    /// - `Err(KalshiError)`: Error in case of a failure in the HTTP request.
    ///
    /// # Examples
    /// ```
    /// kalshi_instance.logout().await?;
    /// ```
    pub async fn logout(&self) -> Result<(), KalshiError> {
        let logout_url: &str = &format!("{}/logout", self.base_url.to_string());

        self.client
            .post(logout_url)
            .header("Authorization", self.curr_token.clone().unwrap())
            .header("content-type", "application/json".to_string())
            .send()
            .await?;

        return Ok(());
    }

    /// Generates the required headers for API key authentication.
    ///
    /// This is a helper function that creates the timestamp, message string, and signature
    /// required for authenticating with an API key.
    ///
    /// # Arguments
    /// * `method` - The HTTP method of the request (e.g., "GET", "POST").
    /// * `path` - The request path (e.g., "/trade-api/v2/portfolio/balance").
    ///
    /// # Returns
    /// - `Ok(HeaderMap)`: A map of headers to be added to the request.
    /// - `Err(KalshiError)`: An error if signing fails or if not authenticated with an API key.
    pub fn get_api_key_headers(
        &self,
        method: &str,
        path: &str,
    ) -> Result<reqwest::header::HeaderMap, KalshiError> {
        let (api_key_id, private_key) =
            if let (Some(id), Some(key)) = (&self.api_key_id, &self.private_key) {
                (id, key)
            } else {
                // This indicates the user is trying to make a signed request without
                // having called `login_apikey` first.
                return Err(KalshiError::NotAuthenticated);
            };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        // todo: this is a hack to get the path to work, need to find a better way to pass this in
        let msg_string = format!("{}{}/trade-api/v2{}", timestamp, method.to_uppercase(), path);

        let sig_b64 = sign_pss_text(private_key, &msg_string)
            .map_err(|e| KalshiError::CryptoError(format!("Signing failed: {}", e)))?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("KALSHI-ACCESS-KEY", api_key_id.parse().unwrap());
        headers.insert("KALSHI-ACCESS-TIMESTAMP", timestamp.parse().unwrap());
        headers.insert("KALSHI-ACCESS-SIGNATURE", sig_b64.parse().unwrap());

        Ok(headers)
    }
}

// used in login method
#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    member_id: String,
    token: String,
}
// used in login method
#[derive(Debug, Serialize, Deserialize)]
struct LoginPayload {
    email: String,
    password: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::traits::PublicKeyParts;

    #[test]
    fn test_parse_rsa_private_key() {
        // test key, generated from kalshi and then invalidated
        let private_key_pem = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEAvVmhxyQUKdC8h4sEQnrvUO0J4qyDUbeLsCe+q33RvtqflCr8
DrZNry+qYJDava14xF4l1a3EbW70pkFECn4sIVOIq4ZVjTVdj9MnBJpQg2qKADMi
ahHA7jJJQbm9I/tbtEcTjAVrqti8wo4D5k9EjMwFWo5c6LMkRYEHy07NBjlDOK5+
271b7lrQENatkXEaXp2QQzpDdoQD4BAyOX25zTU0wAwVQ6uvJhyvIRKr4nz0ZzzF
WbAzMilGzKriPzdWcxDfCZvrEnxTTpa6bdWMbSwiFvs+HikB4a3RHRu5MID0z+IH
oBNJ7FtwzSMEzKcW9Rgm3FueKGzQjp/xjb6SLwIDAQABAoIBAQC8Uw+CXzHmvQMl
1HAyJs8rL/brCiW5+tHmLEGJkyQvrIWW+oGjqFHvcXsFfEzy5jv9Ip6CvcdmCDsv
uC4SOZdutgRyhLNGNNOPnrVp8Ikvi6Ehvbn2wR2gS0dtJW0nAnMdBKw+UY3aaKKk
5laelCxb1PdmL73ce4AR2NmFriRgkJE2TzwnEgBCl8ZfyNA+PTBlZVmEPRrTCoA5
qcIBn1vC3sj4ii/G69hDoXVS6t+9CNWLv8xIzaP8AvxSHDEynciZOI6nwXewq24f
V5E4vzfOvhuF84P7TqNNspGeNfk8SBFTpub9+qNb8Hm70oRlmDALegiO/R3QDDTL
TBkkPbRxAoGBANQZ7EvtsoDrdmq41hlC94DSzhhY+c8mC/n/2T7U7/TQNG5Uk4CN
4zITgV+AsbXWzol7eptDv5Nb37iAG2ZE7Vhi3Fb+L1lCHYdhjoEFylifidoHjP8Y
b9mqaWUB28Xmi2Z99Mnk6xWNXvMR+GlxcyEuCUDbzDPMSYfjvyBmDUhDAoGBAOSK
QGT478Q4yOJVR6CGUWNOOI5RpM4EXXng7fTJGXC+R35eg4GAHKZEGAUe+dLE/hiF
Ows0oyXqCeXwp+JpIEcqSXEVXC+2lAK4Bz17XXsGKIdvAV6IeVdcH4PBK8v5VsUk
A4XmEeo128jUt7A582Ta/mMiksYz+bpcl6uRYJWlAoGANvaMxFRReJUL97X0TVGM
P8bg/3A3NBYA7oT9cAnQMNmvbJPgMWUTZgul7/CynJOQrBHigM+6ml4piG8yKntc
IhZkUOrHrFK0wjtmqUqt1+9n0qc4Q3a6rWY6r6EeqZcHssSbJaJ7xPcAju6uN+zd
T9DVNwh+T0H2IA/FnIi1km8CgYAcJu+hwIyAhmIwh0LIgmM6MWOEHIiJnD4Limql
kbQhkD7sUSYv6KEe1hqDXvp1PTDzwk2wpq5GOFs5yPhVSo/gVFQxqujtM7dt0k+K
Ak1Un0CU1la712HjIgT7zOrhOHi41iPc9adVS4ckaRerjKfvz44wlgywf6yOiWNh
jgnwxQKBgEb9jYDvOwz1xPZSBaHm5frgWMgqRVO97zpPM0orSXmogVMunnTHbpIl
hNM2mqo99UrvM1v8Yowq9jq2ADUoUGO9z3+xvWoVk21CPbvVozubTdlq0ck6PfiJ
CKBXk2AapAiTdeHqqwuhIPp+JvlKDsTF8jPqMXLq7WXZHWrLuubX
-----END RSA PRIVATE KEY-----"#;

        // Test that the private key can be parsed successfully
        let result = RsaPrivateKey::from_pkcs1_pem(private_key_pem);
        assert!(result.is_ok(), "Failed to parse RSA private key: {:?}", result.err());

        let private_key = result.unwrap();

        // Verify the key size (this key should be 2048 bits)
        assert_eq!(private_key.size(), 256, "Expected 2048-bit RSA key (256 bytes)");

        // If we got here, the key was parsed successfully and can be used
        // for cryptographic operations (the actual signing is tested in integration tests)
    }
}
