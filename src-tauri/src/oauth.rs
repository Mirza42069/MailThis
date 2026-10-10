//! Google sign-in for an installed app: PKCE with a loopback redirect.
//! https://developers.google.com/identity/protocols/oauth2/native-app

use std::time::Duration;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Url;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::Result;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
/// Gmail read, label, trash and send; `openid email profile` give the address and display name.
const SCOPES: &str = "https://www.googleapis.com/auth/gmail.modify openid email profile";

#[derive(Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
}

fn random() -> Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| e.to_string())?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

/// Opens Google's consent page in the system browser and waits for it to redirect back.
pub async fn sign_in(
    http: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    open: impl FnOnce(&str) -> Result<()>,
) -> Result<Tokens> {
    let listener = TcpListener::bind("127.0.0.1:0").await.map_err(|e| e.to_string())?;
    let redirect = format!(
        "http://127.0.0.1:{}",
        listener.local_addr().map_err(|e| e.to_string())?.port()
    );
    let verifier = random()?;
    let state = random()?;
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));

    let url = Url::parse_with_params(
        AUTH_URL,
        &[
            ("client_id", client_id),
            ("redirect_uri", &redirect),
            ("response_type", "code"),
            ("scope", SCOPES),
            ("code_challenge", &challenge),
            ("code_challenge_method", "S256"),
            ("access_type", "offline"),
            // Always ask, so Google returns a refresh token even for an account signed in before.
            ("prompt", "consent select_account"),
            ("state", &state),
        ],
    )
    .map_err(|e| e.to_string())?;
    open(url.as_str())?;

    let code = tokio::time::timeout(Duration::from_secs(300), accept(&listener, &state))
        .await
        .map_err(|_| "Sign-in timed out. Try Add Account again.".to_string())??;

    exchange(
        http,
        &[
            ("code", code.as_str()),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", &redirect),
            ("grant_type", "authorization_code"),
            ("code_verifier", &verifier),
        ],
    )
    .await
}

/// Reads the one redirect request; anything else (a favicon fetch) is answered and skipped.
async fn accept(listener: &TcpListener, state: &str) -> Result<String> {
    loop {
        let (mut socket, _) = listener.accept().await.map_err(|e| e.to_string())?;
        let mut buf = vec![0u8; 8192];
        let n = socket.read(&mut buf).await.map_err(|e| e.to_string())?;
        let request = String::from_utf8_lossy(&buf[..n]);
        let path = request.split_whitespace().nth(1).unwrap_or("/");
        let url = Url::parse(&format!("http://127.0.0.1{path}")).map_err(|e| e.to_string())?;
        let param = |key: &str| url.query_pairs().find(|(k, _)| k == key).map(|(_, v)| v.into_owned());

        let (code, error) = (param("code"), param("error"));
        if code.is_none() && error.is_none() {
            let _ = socket
                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                .await;
            continue;
        }
        let ok = code.is_some() && param("state").as_deref() == Some(state);
        let body = format!(
            "<!doctype html><meta charset=utf-8><title>MailThis</title>\
             <body style=\"margin:0;height:100vh;display:grid;place-items:center;background:#0a0a0a;color:#e3e3e3;font:15px system-ui\">\
             <p>{}</p>",
            if ok { "Signed in. You can close this tab and return to MailThis." } else { "Sign-in was cancelled." }
        );
        let _ = socket
            .write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .as_bytes(),
            )
            .await;
        return match (ok, code, error) {
            (true, Some(code), _) => Ok(code),
            (_, _, Some(error)) => Err(format!("Google sign-in failed: {error}")),
            _ => Err("Google sign-in returned an unexpected response.".into()),
        };
    }
}

pub async fn refresh(
    http: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<Tokens> {
    exchange(
        http,
        &[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ],
    )
    .await
}

async fn exchange(http: &reqwest::Client, form: &[(&str, &str)]) -> Result<Tokens> {
    let response = http
        .post(TOKEN_URL)
        .form(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Google token request failed: {text}"));
    }
    response.json().await.map_err(|e| e.to_string())
}
