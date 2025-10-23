//! Authentication strategy implementations

use crate::auth::Authenticator;
use async_trait::async_trait;
use std::collections::HashMap;

/// API Key authentication strategy
pub struct ApiKeyAuth {
    key: String,
    location: ApiKeyLocation,
    name: String,
}

/// API key location options
#[derive(Debug, Clone)]
pub enum ApiKeyLocation {
    Header,
    Query,
    Cookie,
}

impl ApiKeyAuth {
    /// Create a new API key authenticator
    pub fn new<S: Into<String>>(key: S, location: ApiKeyLocation, name: S) -> Self {
        Self {
            key: key.into(),
            location,
            name: name.into(),
        }
    }

    /// Create API key authenticator for Authorization header
    pub fn authorization_header(key: &str) -> Self {
        Self::new(key, ApiKeyLocation::Header, "Authorization")
    }

    /// Create API key authenticator for X-API-Key header
    pub fn x_api_key_header(key: &str) -> Self {
        Self::new(key, ApiKeyLocation::Header, "X-API-Key")
    }
}

#[async_trait]
impl Authenticator for ApiKeyAuth {
    async fn authenticate(
        &self,
        headers: &mut HashMap<String, String>,
    ) -> Result<(), crate::A2aError> {
        if self.key.is_empty() {
            return Err(crate::A2aError::Authentication(
                "API key is empty".to_string(),
            ));
        }

        match self.location {
            ApiKeyLocation::Header => {
                headers.insert(self.name.clone(), self.key.clone());
            }
            ApiKeyLocation::Query => {
                // For query parameters, we'd need to modify the URL
                // This is handled at the transport level
                return Err(crate::A2aError::Authentication(
                    "Query parameter authentication not yet implemented".to_string(),
                ));
            }
            ApiKeyLocation::Cookie => {
                // For cookies, we'd need to set cookie headers
                return Err(crate::A2aError::Authentication(
                    "Cookie authentication not yet implemented".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn auth_type(&self) -> &'static str {
        "api_key"
    }

    fn is_configured(&self) -> bool {
        !self.key.is_empty()
    }
}

/// Bearer token authentication strategy
pub struct BearerAuth {
    token: String,
}

impl BearerAuth {
    /// Create a new bearer token authenticator
    pub fn new<S: Into<String>>(token: S) -> Self {
        Self {
            token: token.into(),
        }
    }
}

#[async_trait]
impl Authenticator for BearerAuth {
    async fn authenticate(
        &self,
        headers: &mut HashMap<String, String>,
    ) -> Result<(), crate::A2aError> {
        if self.token.is_empty() {
            return Err(crate::A2aError::Authentication(
                "Bearer token is empty".to_string(),
            ));
        }

        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.token),
        );
        Ok(())
    }

    fn auth_type(&self) -> &'static str {
        "bearer"
    }

    fn is_configured(&self) -> bool {
        !self.token.is_empty()
    }
}

/// OAuth2 client credentials authentication
pub struct OAuth2ClientCredentials {
    token_url: String,
    client_id: String,
    client_secret: String,
    scope: Option<String>,
    cached_token: Option<String>,
    token_expiry: Option<std::time::Instant>,
}

impl OAuth2ClientCredentials {
    /// Create a new OAuth2 client credentials authenticator
    pub fn new<S: Into<String>>(
        token_url: S,
        client_id: S,
        client_secret: S,
        scope: Option<S>,
    ) -> Self {
        Self {
            token_url: token_url.into(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            scope: scope.map(|s| s.into()),
            cached_token: None,
            token_expiry: None,
        }
    }

    /// Check if the cached token is still valid
    fn is_token_valid(&self) -> bool {
        if let Some(expiry) = self.token_expiry {
            std::time::Instant::now() < expiry
        } else {
            false
        }
    }

    /// Request a new access token
    async fn request_token(&self) -> Result<String, crate::A2aError> {
        use reqwest::Client;

        let client = Client::new();
        let mut form = std::collections::HashMap::new();
        form.insert("grant_type".to_string(), "client_credentials".to_string());
        form.insert("client_id".to_string(), self.client_id.clone());
        form.insert("client_secret".to_string(), self.client_secret.clone());

        if let Some(ref scope) = self.scope {
            form.insert("scope".to_string(), scope.clone());
        }

        let response = client
            .post(&self.token_url)
            .header("Accept", "application/json")
            .form(&form)
            .send()
            .await
            .map_err(|e| crate::A2aError::Network(e))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::A2aError::Authentication(format!(
                "OAuth2 token request failed: {}",
                error_text
            )));
        }

        let token_response: OAuth2TokenResponse = response
            .json()
            .await
            .map_err(|e| crate::A2aError::Network(e))?;

        Ok(token_response.access_token)
    }
}

#[async_trait]
impl Authenticator for OAuth2ClientCredentials {
    async fn authenticate(
        &self,
        headers: &mut HashMap<String, String>,
    ) -> Result<(), crate::A2aError> {
        let token = if self.is_token_valid() {
            self.cached_token.as_ref().unwrap()
        } else {
            return Err(crate::A2aError::Authentication(
                "OAuth2 token expired or not available. Token refreshing not yet implemented."
                    .to_string(),
            ));
        };

        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        Ok(())
    }

    fn auth_type(&self) -> &'static str {
        "oauth2_client_credentials"
    }

    fn is_configured(&self) -> bool {
        !self.client_id.is_empty() && !self.client_secret.is_empty()
    }
}

/// OAuth2 token response
#[derive(serde::Deserialize)]
struct OAuth2TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: Option<u64>,
    scope: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_key_auth() {
        let auth = ApiKeyAuth::authorization_header("test-key");
        let mut headers = HashMap::new();

        auth.authenticate(&mut headers).await.unwrap();

        assert_eq!(headers.get("Authorization"), Some(&"test-key".to_string()));
    }

    #[tokio::test]
    async fn test_bearer_auth() {
        let auth = BearerAuth::new("test-token");
        let mut headers = HashMap::new();

        auth.authenticate(&mut headers).await.unwrap();

        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer test-token".to_string())
        );
    }

    #[tokio::test]
    async fn test_empty_api_key() {
        let auth = ApiKeyAuth::authorization_header("");
        let mut headers = HashMap::new();

        let result = auth.authenticate(&mut headers).await;
        assert!(matches!(result, Err(crate::A2aError::Authentication(_))));
    }

    #[test]
    fn test_auth_types() {
        let api_key = ApiKeyAuth::authorization_header("key");
        assert_eq!(api_key.auth_type(), "api_key");

        let bearer = BearerAuth::new("token");
        assert_eq!(bearer.auth_type(), "bearer");
    }
}
