mod gmail;
mod oauth;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tauri_plugin_opener::OpenerExt;

use gmail::{Addr, Draft, Gmail, Label, Message, ThreadPage};

pub type Result<T> = std::result::Result<T, String>;

/// Refresh tokens live in Windows Credential Manager under this service name.
const KEYRING: &str = "MailThis";

/// The Google OAuth client compiled in from `.env` (build.rs), so people can just
/// sign in. A client saved in Settings takes precedence.
const BUILT_IN: Option<(&str, &str)> = match (
    option_env!("MAILTHIS_GOOGLE_CLIENT_ID"),
    option_env!("MAILTHIS_GOOGLE_CLIENT_SECRET"),
) {
    (Some(id), Some(secret)) if !id.is_empty() => Some((id, secret)),
    _ => None,
};

/// `config.json` in the app's config directory. Holds nothing secret beyond the
/// OAuth client, which Google itself treats as non-confidential for desktop apps.
#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Config {
    client_id: String,
    client_secret: String,
    accounts: Vec<Addr>,
}

struct App {
    http: reqwest::Client,
    path: PathBuf,
    config: Mutex<Config>,
    /// Access tokens per account, with when they expire.
    tokens: Mutex<HashMap<String, (String, Instant)>>,
}

impl App {
    fn config(&self) -> Config {
        self.config.lock().unwrap().clone()
    }

    fn save(&self, config: Config) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(&self.path, serde_json::to_vec_pretty(&config).unwrap()).map_err(|e| e.to_string())?;
        *self.config.lock().unwrap() = config;
        Ok(())
    }

    fn client(&self) -> Result<(String, String)> {
        let config = self.config();
        if !config.client_id.is_empty() {
            return Ok((config.client_id, config.client_secret));
        }
        BUILT_IN
            .map(|(id, secret)| (id.to_string(), secret.to_string()))
            .ok_or_else(|| "Add your Google OAuth client in Settings first.".into())
    }

    fn remember(&self, account: &str, token: &oauth::Tokens) {
        let expires = Instant::now() + Duration::from_secs(token.expires_in);
        self.tokens
            .lock()
            .unwrap()
            .insert(account.to_string(), (token.access_token.clone(), expires));
    }

    async fn gmail(&self, account: &str) -> Result<Gmail<'_>> {
        let cached = self.tokens.lock().unwrap().get(account).cloned();
        let token = match cached {
            Some((token, expires)) if expires > Instant::now() + Duration::from_secs(60) => token,
            _ => {
                let refresh = keyring::Entry::new(KEYRING, account)
                    .and_then(|e| e.get_password())
                    .map_err(|_| format!("{account} needs to be signed in again (Settings)."))?;
                let (id, secret) = self.client()?;
                let fresh = oauth::refresh(&self.http, &id, &secret, &refresh).await?;
                self.remember(account, &fresh);
                fresh.access_token
            }
        };
        Ok(Gmail {
            http: &self.http,
            token,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    client_id: String,
    has_secret: bool,
    built_in: bool,
    accounts: Vec<Addr>,
}

#[tauri::command]
fn settings(app: State<'_, App>) -> Settings {
    let config = app.config();
    Settings {
        client_id: config.client_id,
        has_secret: !config.client_secret.is_empty(),
        built_in: BUILT_IN.is_some(),
        accounts: config.accounts,
    }
}

#[tauri::command]
fn set_client(app: State<'_, App>, client_id: String, client_secret: String) -> Result<()> {
    let mut config = app.config();
    config.client_id = client_id.trim().to_string();
    // Left blank in the form means "keep the saved one".
    if !client_secret.trim().is_empty() {
        config.client_secret = client_secret.trim().to_string();
    }
    app.save(config)
}

#[tauri::command]
async fn add_account(handle: tauri::AppHandle, app: State<'_, App>) -> Result<Addr> {
    let (id, secret) = app.client()?;
    let tokens = oauth::sign_in(&app.http, &id, &secret, |url| {
        handle.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
    })
    .await?;
    let user = Gmail {
        http: &app.http,
        token: tokens.access_token.clone(),
    }
    .user_info()
    .await?;
    let refresh = tokens
        .refresh_token
        .as_deref()
        .ok_or("Google did not grant offline access; try again.")?;
    keyring::Entry::new(KEYRING, &user.email)
        .and_then(|e| e.set_password(refresh))
        .map_err(|e| e.to_string())?;
    app.remember(&user.email, &tokens);

    let mut config = app.config();
    config.accounts.retain(|a| a.email != user.email);
    config.accounts.push(user.clone());
    app.save(config)?;
    Ok(user)
}

#[tauri::command]
fn remove_account(app: State<'_, App>, account: String) -> Result<()> {
    let _ = keyring::Entry::new(KEYRING, &account).and_then(|e| e.delete_credential());
    app.tokens.lock().unwrap().remove(&account);
    let mut config = app.config();
    config.accounts.retain(|a| a.email != account);
    app.save(config)
}

#[tauri::command]
async fn labels(app: State<'_, App>, account: String) -> Result<Vec<Label>> {
    app.gmail(&account).await?.labels().await
}

#[tauri::command]
async fn threads(
    app: State<'_, App>,
    account: String,
    label: Option<String>,
    query: String,
    page_token: Option<String>,
) -> Result<ThreadPage> {
    app.gmail(&account)
        .await?
        .threads(&account, label.as_deref(), &query, page_token.as_deref())
        .await
}

#[tauri::command]
async fn thread(app: State<'_, App>, account: String, id: String) -> Result<Vec<Message>> {
    app.gmail(&account).await?.thread(&id).await
}

#[tauri::command]
async fn modify_thread(
    app: State<'_, App>,
    account: String,
    id: String,
    add: Vec<String>,
    remove: Vec<String>,
) -> Result<()> {
    app.gmail(&account).await?.modify_thread(&id, &add, &remove).await
}

#[tauri::command]
async fn modify_message(
    app: State<'_, App>,
    account: String,
    id: String,
    add: Vec<String>,
    remove: Vec<String>,
) -> Result<()> {
    app.gmail(&account).await?.modify_message(&id, &add, &remove).await
}

#[tauri::command]
async fn trash(app: State<'_, App>, account: String, id: String) -> Result<()> {
    app.gmail(&account).await?.trash(&id).await
}

#[tauri::command]
async fn send(app: State<'_, App>, account: String, draft: Draft) -> Result<()> {
    let from = app
        .config()
        .accounts
        .into_iter()
        .find(|a| a.email == account)
        .ok_or("Unknown account")?;
    app.gmail(&account).await?.send_message(&from, &draft).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let path = app.path().app_config_dir()?.join("config.json");
            let config = std::fs::read(&path)
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or_default();
            app.manage(App {
                http: reqwest::Client::new(),
                path,
                config: Mutex::new(config),
                tokens: Mutex::default(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            settings,
            set_client,
            add_account,
            remove_account,
            labels,
            threads,
            thread,
            modify_thread,
            modify_message,
            trash,
            send
        ])
        .run(tauri::generate_context!())
        .expect("error while running MailThis");
}
