use crate::error::{MilieuError, Result};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub user_id: String,
    pub warning: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UmkResponse {
    pub encrypted_umk: String,
    pub kdf_params: serde_json::Value,
    pub version: u32,
}

pub type UmkRequest = UmkResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectRequest {
    pub path: String,
    pub nonce: String,
    pub ciphertext: String,
    pub aad: String,
    pub ciphertext_hash: String,
    pub created_at: String,
    pub schema_version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectResponse {
    pub path: String,
    pub nonce: String,
    pub ciphertext: String,
    pub aad: String,
    pub ciphertext_hash: Option<String>,
    pub version: Option<u32>,
    pub created_at: String,
    pub schema_version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub version: u32,
    pub created_at: String,
    pub ciphertext_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoInfo {
    pub repo_id: String,
    pub name: String,
    pub last_seen: String,
    pub owner_email: Option<String>,
    pub access: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoListResponse {
    pub repos: Vec<RepoInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub host: String,
    pub created_at: String,
    pub expires_at: String,
    pub token_suffix: String,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoAccessEntry {
    pub email: String,
    pub role: String,
    pub status: String,
    pub invited_by: Option<String>,
    pub created_at: String,
    pub public_key: Option<String>,
    pub key_algorithm: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteInfo {
    pub id: String,
    pub repo_id: String,
    pub repo_name: String,
    pub role: String,
    pub invited_by: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserKeyResponse {
    pub public_key: String,
    pub algorithm: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoKeyResponse {
    pub wrapped_key: String,
    pub algorithm: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoCreateRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoResponse {
    pub repo_id: String,
    pub name: String,
}


#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    token: Option<String>,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(base_url: &str, token: Option<String>) -> Result<Self> {
        let client = reqwest::Client::builder().build()?;
        Ok(Self {
            base_url: base_url.to_string(),
            token,
            client,
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn auth_header(&self) -> Result<String> {
        match &self.token {
            Some(token) => Ok(format!("Bearer {}", token)),
            None => Err(MilieuError::AuthMissing),
        }
    }

    pub async fn login(&self, request: &LoginRequest) -> Result<LoginResponse> {
        let url = self.endpoint("/v1/auth/login");
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "login failed: {}",
                response.status()
            )));
        }
        Ok(response.json().await?)
    }

    pub async fn register(&self, request: &RegisterRequest) -> Result<RegisterResponse> {
        let url = self.endpoint("/v1/auth/register");
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "register failed: {}",
                response.status()
            )));
        }
        Ok(response.json().await?)
    }

    pub async fn get_umk(&self) -> Result<Option<UmkResponse>> {
        let url = self.endpoint("/v1/users/me/umk");
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "get umk failed: {}",
                response.status()
            )));
        }
        Ok(Some(response.json().await?))
    }

    pub async fn put_umk(&self, request: &UmkRequest) -> Result<()> {
        let url = self.endpoint("/v1/users/me/umk");
        let response = self
            .client
            .put(url)
            .header("Authorization", self.auth_header()?)
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "put umk failed: {}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn post_object(
        &self,
        repo_id: &str,
        branch: &str,
        request: &ObjectRequest,
    ) -> Result<ObjectResponse> {
        let path = format!(
            "/v1/repos/{}/branches/{}/objects",
            repo_id, branch
        );
        let url = self.endpoint(&path);
        let response = self
            .client
            .post(url)
            .header("Authorization", self.auth_header()?)
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            let message = match response.status() {
                StatusCode::NOT_FOUND => {
                    "post object failed: repo not found or no write access (read-only)".to_string()
                }
                StatusCode::UNAUTHORIZED => "post object failed: unauthorized".to_string(),
                StatusCode::FORBIDDEN => "post object failed: forbidden".to_string(),
                _ => format!("post object failed: {}", response.status()),
            };
            return Err(MilieuError::CommandFailed(message));
        }
        Ok(response.json().await?)
    }

    pub async fn get_latest(
        &self,
        repo_id: &str,
        branch: &str,
        path: &str,
    ) -> Result<Option<ObjectResponse>> {
        let endpoint = format!(
            "/v1/repos/{}/branches/{}/objects/latest?path={}",
            repo_id,
            branch,
            urlencoding::encode(path)
        );
        let url = self.endpoint(&endpoint);
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "get latest failed: {}",
                response.status()
            )));
        }
        Ok(Some(response.json().await?))
    }

    pub async fn get_version(
        &self,
        repo_id: &str,
        branch: &str,
        path: &str,
        version: u32,
    ) -> Result<crate::api::ObjectResponse> {
        let endpoint = format!(
            "/v1/repos/{}/branches/{}/objects/version?path={}&version={}",
            repo_id,
            branch,
            urlencoding::encode(path),
            version
        );
        let url = self.endpoint(&endpoint);
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "get version failed: {}",
                response.status()
            )));
        }
        Ok(response.json().await?)
    }

    pub async fn get_history(
        &self,
        repo_id: &str,
        branch: &str,
        path: &str,
    ) -> Result<Vec<HistoryEntry>> {
        let endpoint = format!(
            "/v1/repos/{}/branches/{}/objects/history?path={}",
            repo_id,
            branch,
            urlencoding::encode(path)
        );
        let url = self.endpoint(&endpoint);
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "get history failed: {}",
                response.status()
            )));
        }
        let body: Vec<HistoryEntry> = response.json().await?;
        Ok(body)
    }

    pub async fn get_repos(&self) -> Result<Vec<RepoInfo>> {
        let url = self.endpoint("/v1/users/me/repos");
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "get repos failed: {}",
                response.status()
            )));
        }
        let body: RepoListResponse = response.json().await?;
        Ok(body.repos)
    }

    pub async fn get_repo_access(&self, repo_id: &str) -> Result<Vec<RepoAccessEntry>> {
        let url = self.endpoint(&format!("/v1/repos/{}/access", repo_id));
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "get access failed: {}",
                response.status()
            )));
        }
        #[derive(Deserialize)]
        struct AccessResponse {
            entries: Vec<RepoAccessEntry>,
        }
        let body: AccessResponse = response.json().await?;
        Ok(body.entries)
    }

    pub async fn invite_repo_access(
        &self,
        repo_id: &str,
        email: &str,
        role: &str,
    ) -> Result<()> {
        let url = self.endpoint(&format!("/v1/repos/{}/access", repo_id));
        let response = self
            .client
            .post(url)
            .header("Authorization", self.auth_header()?)
            .json(&serde_json::json!({ "email": email, "role": role }))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "invite failed: {}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn update_repo_access(
        &self,
        repo_id: &str,
        email: &str,
        role: &str,
    ) -> Result<()> {
        let url = self.endpoint(&format!("/v1/repos/{}/access", repo_id));
        let response = self
            .client
            .patch(url)
            .header("Authorization", self.auth_header()?)
            .json(&serde_json::json!({ "email": email, "role": role }))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "update access failed: {}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn revoke_repo_access(&self, repo_id: &str, email: &str) -> Result<()> {
        let url = self.endpoint(&format!(
            "/v1/repos/{}/access?email={}",
            repo_id,
            urlencoding::encode(email)
        ));
        let response = self
            .client
            .delete(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "revoke access failed: {}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn delete_repo(&self, repo_id: &str) -> Result<()> {
        let url = self.endpoint(&format!("/v1/repos/{}", repo_id));
        let response = self
            .client
            .delete(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "delete repo failed: {}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn get_invites(&self) -> Result<Vec<InviteInfo>> {
        let url = self.endpoint("/v1/users/me/invites");
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "get invites failed: {}",
                response.status()
            )));
        }
        Ok(response.json().await?)
    }

    pub async fn get_user_key(&self) -> Result<Option<UserKeyResponse>> {
        let url = self.endpoint("/v1/users/me/key");
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "get user key failed: {}",
                response.status()
            )));
        }
        Ok(Some(response.json().await?))
    }

    pub async fn put_user_key(&self, public_key: &str, algorithm: &str) -> Result<()> {
        let url = self.endpoint("/v1/users/me/key");
        let response = self
            .client
            .put(url)
            .header("Authorization", self.auth_header()?)
            .json(&serde_json::json!({ "public_key": public_key, "algorithm": algorithm }))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "put user key failed: {}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn get_repo_key(&self, repo_id: &str) -> Result<Option<RepoKeyResponse>> {
        let url = self.endpoint(&format!("/v1/repos/{}/key", repo_id));
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "get repo key failed: {}",
                response.status()
            )));
        }
        Ok(Some(response.json().await?))
    }

    pub async fn put_repo_key(
        &self,
        repo_id: &str,
        wrapped_key: &str,
        algorithm: &str,
        email: Option<&str>,
    ) -> Result<()> {
        let url = self.endpoint(&format!("/v1/repos/{}/key", repo_id));
        let mut body = serde_json::json!({ "wrapped_key": wrapped_key, "algorithm": algorithm });
        if let Some(value) = email {
            body["email"] = serde_json::Value::String(value.to_string());
        }
        let response = self
            .client
            .put(url)
            .header("Authorization", self.auth_header()?)
            .json(&body)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "put repo key failed: {}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn accept_invite(&self, invite_id: &str) -> Result<()> {
        let url = self.endpoint(&format!(
            "/v1/users/me/invites/{}/accept",
            invite_id
        ));
        let response = self
            .client
            .post(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "accept invite failed: {}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn reject_invite(&self, invite_id: &str) -> Result<()> {
        let url = self.endpoint(&format!(
            "/v1/users/me/invites/{}/reject",
            invite_id
        ));
        let response = self
            .client
            .post(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "reject invite failed: {}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn get_sessions(&self) -> Result<Vec<SessionInfo>> {
        let url = self.endpoint("/v1/users/me/sessions");
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "get sessions failed: {}",
                response.status()
            )));
        }
        Ok(response.json().await?)
    }

    pub async fn logout(&self) -> Result<()> {
        let url = self.endpoint("/v1/auth/logout");
        let response = self
            .client
            .post(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "logout failed: {}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn create_repo(&self, name: &str) -> Result<RepoResponse> {
        let url = self.endpoint("/v1/repos");
        let response = self
            .client
            .post(url)
            .header("Authorization", self.auth_header()?)
            .json(&RepoCreateRequest {
                name: name.to_string(),
            })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "create repo failed: {}",
                response.status()
            )));
        }
        Ok(response.json().await?)
    }

    pub async fn get_repo_by_name(&self, name: &str) -> Result<RepoResponse> {
        let endpoint = format!("/v1/repos?name={}", urlencoding::encode(name));
        let url = self.endpoint(&endpoint);
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(MilieuError::CommandFailed("repo not found".to_string()));
        }
        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "get repo failed: {}",
                response.status()
            )));
        }
        Ok(response.json().await?)
    }

    pub async fn get_manifest(&self, repo_id: &str) -> Result<crate::manifest::Manifest> {
        let endpoint = format!("/v1/repos/{}/manifest", repo_id);
        let url = self.endpoint(&endpoint);
        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MilieuError::CommandFailed(format!(
                "get manifest failed: {}",
                response.status()
            )));
        }
        Ok(response.json().await?)
    }

    pub async fn put_manifest(&self, manifest: &crate::manifest::Manifest) -> Result<()> {
        let endpoint = format!("/v1/repos/{}/manifest", manifest.repo_id);
        let url = self.endpoint(&endpoint);
        let sanitized = manifest.without_state();
        let response = self
            .client
            .put(url)
            .header("Authorization", self.auth_header()?)
            .json(&sanitized)
            .send()
            .await?;

        if !response.status().is_success() {
            let message = match response.status() {
                StatusCode::NOT_FOUND => {
                    "put manifest failed: repo not found or no write access (read-only)".to_string()
                }
                StatusCode::UNAUTHORIZED => "put manifest failed: unauthorized".to_string(),
                StatusCode::FORBIDDEN => "put manifest failed: forbidden".to_string(),
                _ => format!("put manifest failed: {}", response.status()),
            };
            return Err(MilieuError::CommandFailed(message));
        }
        Ok(())
    }
}
