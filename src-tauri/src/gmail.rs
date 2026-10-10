//! The Gmail REST API (https://developers.google.com/gmail/api/reference/rest), and MIME in and out.

use base64::alphabet::URL_SAFE;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig, STANDARD, URL_SAFE_NO_PAD};
use base64::engine::DecodePaddingMode;
use base64::Engine;
use futures::stream::{self, StreamExt};
use mail_parser::{Address, MessageParser, MimeHeaders, PartType};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::Result;

const API: &str = "https://gmail.googleapis.com/gmail/v1/users/me";
/// Gmail's base64url sometimes carries padding and sometimes not.
const RAW: GeneralPurpose = GeneralPurpose::new(
    &URL_SAFE,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);
/// Parallel requests per list; Gmail's per-user quota tolerates this comfortably.
const PARALLEL: usize = 10;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Addr {
    pub name: Option<String>,
    pub email: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadSummary {
    pub account: String,
    pub id: String,
    pub from: Addr,
    pub subject: String,
    pub snippet: String,
    pub date: i64,
    pub count: usize,
    pub unread: bool,
    pub starred: bool,
    pub label_ids: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadPage {
    pub threads: Vec<ThreadSummary>,
    pub next_page_token: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub from: Addr,
    pub reply_to: Vec<Addr>,
    pub to: Vec<Addr>,
    pub cc: Vec<Addr>,
    pub date: i64,
    pub subject: String,
    pub html: Option<String>,
    pub text: Option<String>,
    pub message_id: Option<String>,
    pub references: Vec<String>,
    pub attachments: Vec<String>,
    pub label_ids: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub name: String,
    pub system: bool,
    pub unread: u32,
    pub total: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub to: String,
    pub cc: String,
    pub subject: String,
    pub body: String,
    pub thread_id: Option<String>,
    pub in_reply_to: Option<String>,
    #[serde(default)]
    pub references: Vec<String>,
}

pub struct Gmail<'a> {
    pub http: &'a reqwest::Client,
    pub token: String,
}

impl Gmail<'_> {
    async fn send<T: DeserializeOwned>(&self, request: reqwest::RequestBuilder) -> Result<T> {
        let response = request
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            let status = response.status();
            let body: Value = response.json().await.unwrap_or_default();
            let message = body["error"]["message"].as_str().unwrap_or("request failed");
            return Err(format!("Gmail {status}: {message}"));
        }
        response.json().await.map_err(|e| e.to_string())
    }

    async fn get<T: DeserializeOwned>(&self, path: impl std::fmt::Display, query: &[(&str, &str)]) -> Result<T> {
        self.send(self.http.get(format!("{API}{path}")).query(query)).await
    }

    async fn post<T: DeserializeOwned>(&self, path: &str, body: Value) -> Result<T> {
        self.send(self.http.post(format!("{API}{path}")).json(&body)).await
    }

    pub async fn user_info(&self) -> Result<Addr> {
        let info: Value = self
            .send(self.http.get("https://openidconnect.googleapis.com/v1/userinfo"))
            .await?;
        Ok(Addr {
            name: info["name"].as_str().map(String::from),
            email: info["email"]
                .as_str()
                .ok_or("Google did not return an email address")?
                .to_string(),
        })
    }

    pub async fn labels(&self) -> Result<Vec<Label>> {
        let list: Value = self.get("/labels", &[]).await?;
        let ids: Vec<String> = list["labels"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|l| l["type"] == "user" || SYSTEM_LABELS.contains(&l["id"].as_str().unwrap_or("")))
            .filter_map(|l| l["id"].as_str().map(String::from))
            .collect();
        // The list omits counts; each label's own resource carries them.
        let details: Vec<Result<Value>> = stream::iter(
            ids.iter()
                .map(|id| self.get::<Value>(format!("/labels/{id}"), &[]))
                .collect::<Vec<_>>(),
        )
        .buffered(PARALLEL)
        .collect()
        .await;
        let mut labels: Vec<Label> = details
            .into_iter()
            .filter_map(|l| l.ok())
            .map(|l| Label {
                id: l["id"].as_str().unwrap_or_default().to_string(),
                name: l["name"].as_str().unwrap_or_default().to_string(),
                system: l["type"] == "system",
                unread: l["threadsUnread"].as_u64().unwrap_or(0) as u32,
                total: l["threadsTotal"].as_u64().unwrap_or(0) as u32,
            })
            .collect();
        labels.sort_by_key(|l| {
            (
                !l.system,
                SYSTEM_LABELS.iter().position(|s| *s == l.id),
                l.name.to_lowercase(),
            )
        });
        Ok(labels)
    }

    pub async fn threads(
        &self,
        account: &str,
        label: Option<&str>,
        q: &str,
        page_token: Option<&str>,
    ) -> Result<ThreadPage> {
        let mut query = vec![("maxResults", "30"), ("q", q)];
        if let Some(label) = label {
            query.push(("labelIds", label));
        }
        if let Some(token) = page_token {
            query.push(("pageToken", token));
        }
        let list: Value = self.get("/threads", &query).await?;
        let ids: Vec<String> = list["threads"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|t| t["id"].as_str().map(String::from))
            .collect();
        let threads: Vec<Result<Value>> = stream::iter(
            ids.iter()
                .map(|id| {
                    self.get::<Value>(
                        format!("/threads/{id}"),
                        &[
                            ("format", "metadata"),
                            ("metadataHeaders", "From"),
                            ("metadataHeaders", "Subject"),
                        ],
                    )
                })
                .collect::<Vec<_>>(),
        )
        .buffered(PARALLEL)
        .collect()
        .await;
        Ok(ThreadPage {
            threads: threads
                .into_iter()
                .filter_map(|t| t.ok())
                .filter_map(|t| summarize(account, &t))
                .collect(),
            next_page_token: list["nextPageToken"].as_str().map(String::from),
        })
    }

    pub async fn thread(&self, id: &str) -> Result<Vec<Message>> {
        let thread: Value = self.get(format!("/threads/{id}"), &[("format", "minimal")]).await?;
        let ids: Vec<String> = thread["messages"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|m| !labels_of(m).iter().any(|l| l == "TRASH" || l == "DRAFT"))
            .filter_map(|m| m["id"].as_str().map(String::from))
            .collect();
        let raw: Vec<Result<Value>> = stream::iter(
            ids.iter()
                .map(|id| self.get::<Value>(format!("/messages/{id}"), &[("format", "raw")]))
                .collect::<Vec<_>>(),
        )
        .buffered(PARALLEL)
        .collect()
        .await;
        raw.into_iter()
            .map(|m| {
                let m = m?;
                let bytes = RAW
                    .decode(m["raw"].as_str().unwrap_or_default())
                    .map_err(|e| e.to_string())?;
                parse(
                    m["id"].as_str().unwrap_or_default(),
                    &bytes,
                    internal_date(&m),
                    labels_of(&m),
                )
            })
            .collect()
    }

    pub async fn modify_thread(&self, id: &str, add: &[String], remove: &[String]) -> Result<()> {
        let _: Value = self
            .post(
                &format!("/threads/{id}/modify"),
                json!({ "addLabelIds": add, "removeLabelIds": remove }),
            )
            .await?;
        Ok(())
    }

    pub async fn modify_message(&self, id: &str, add: &[String], remove: &[String]) -> Result<()> {
        let _: Value = self
            .post(
                &format!("/messages/{id}/modify"),
                json!({ "addLabelIds": add, "removeLabelIds": remove }),
            )
            .await?;
        Ok(())
    }

    pub async fn trash(&self, id: &str) -> Result<()> {
        let _: Value = self.post(&format!("/threads/{id}/trash"), json!({})).await?;
        Ok(())
    }

    pub async fn send_message(&self, from: &Addr, draft: &Draft) -> Result<()> {
        let raw = URL_SAFE_NO_PAD.encode(build(from, draft));
        let _: Value = self
            .post("/messages/send", json!({ "raw": raw, "threadId": draft.thread_id }))
            .await?;
        Ok(())
    }
}

/// System labels shown in the sidebar, in this order; the rest (CATEGORY_*, CHAT…) are hidden.
const SYSTEM_LABELS: [&str; 7] = ["INBOX", "STARRED", "IMPORTANT", "SENT", "DRAFT", "SPAM", "TRASH"];

fn labels_of(message: &Value) -> Vec<String> {
    message["labelIds"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|l| l.as_str().map(String::from))
        .collect()
}

fn internal_date(message: &Value) -> i64 {
    message["internalDate"]
        .as_str()
        .and_then(|d| d.parse().ok())
        .unwrap_or(0)
}

fn header(message: &Value, name: &str) -> String {
    message["payload"]["headers"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|h| h["name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case(name)))
        .and_then(|h| h["value"].as_str())
        .unwrap_or_default()
        .to_string()
}

/// One row of the list: Mail.app style, the newest message's sender and the thread's first subject.
fn summarize(account: &str, thread: &Value) -> Option<ThreadSummary> {
    let messages: Vec<&Value> = thread["messages"]
        .as_array()?
        .iter()
        .filter(|m| !labels_of(m).iter().any(|l| l == "DRAFT"))
        .collect();
    let (first, last) = (*messages.first()?, *messages.last()?);
    let mut label_ids: Vec<String> = messages.iter().flat_map(|m| labels_of(m)).collect();
    label_ids.sort();
    label_ids.dedup();
    let subject = header(first, "Subject");
    Some(ThreadSummary {
        account: account.to_string(),
        id: thread["id"].as_str()?.to_string(),
        from: parse_addr(&header(last, "From")),
        subject: if subject.is_empty() {
            "(no subject)".into()
        } else {
            subject
        },
        snippet: decode_entities(last["snippet"].as_str().unwrap_or_default()),
        date: internal_date(last),
        count: messages.len(),
        unread: label_ids.iter().any(|l| l == "UNREAD"),
        starred: label_ids.iter().any(|l| l == "STARRED"),
        label_ids,
    })
}

/// Snippets arrive HTML-escaped.
fn decode_entities(s: &str) -> String {
    s.replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// `Name <a@b.c>`, `"Name" <a@b.c>` or a bare `a@b.c`, as Gmail's metadata headers give them (already decoded).
fn parse_addr(value: &str) -> Addr {
    match (value.rfind('<'), value.rfind('>')) {
        (Some(open), Some(close)) if open < close => {
            let name = value[..open].trim().trim_matches('"').trim();
            Addr {
                name: (!name.is_empty()).then(|| name.to_string()),
                email: value[open + 1..close].trim().to_string(),
            }
        }
        _ => Addr {
            name: None,
            email: value.trim().to_string(),
        },
    }
}

fn addrs(address: Option<&Address>) -> Vec<Addr> {
    address
        .into_iter()
        .flat_map(|a| a.iter())
        .filter_map(|a| {
            Some(Addr {
                name: a.name().map(String::from).filter(|n| !n.is_empty()),
                email: a.address()?.to_string(),
            })
        })
        .collect()
}

pub fn parse(id: &str, bytes: &[u8], date: i64, label_ids: Vec<String>) -> Result<Message> {
    let message = MessageParser::default()
        .parse(bytes)
        .ok_or("Could not read this message")?;
    let ids = |v: &mail_parser::HeaderValue| -> Vec<String> {
        match v {
            mail_parser::HeaderValue::Text(t) => vec![t.to_string()],
            mail_parser::HeaderValue::TextList(l) => l.iter().map(|t| t.to_string()).collect(),
            _ => vec![],
        }
    };
    let from = addrs(message.from()).into_iter().next().unwrap_or(Addr {
        name: None,
        email: String::new(),
    });
    Ok(Message {
        id: id.to_string(),
        from,
        reply_to: addrs(message.reply_to()),
        to: addrs(message.to()),
        cc: addrs(message.cc()),
        date,
        subject: message.subject().unwrap_or("(no subject)").to_string(),
        // Only a real HTML part counts; mail-parser would otherwise wrap the text part in HTML.
        html: message
            .html_part(0)
            .filter(|part| matches!(part.body, PartType::Html(_)))
            .and_then(|_| message.body_html(0).map(|b| b.into_owned())),
        text: message.body_text(0).map(|b| b.into_owned()),
        message_id: message.message_id().map(|m| format!("<{m}>")),
        references: ids(message.references())
            .into_iter()
            .map(|r| format!("<{r}>"))
            .collect(),
        attachments: message
            .attachments()
            .filter_map(|a| a.attachment_name().map(String::from))
            .collect(),
        label_ids,
    })
}

/// RFC 2047 for a header that is not plain ASCII.
fn encode_word(s: &str) -> String {
    if s.is_ascii() {
        s.to_string()
    } else {
        format!("=?UTF-8?B?{}?=", STANDARD.encode(s))
    }
}

/// A plain-text RFC 5322 message. Gmail stamps Date and Message-ID itself.
pub fn build(from: &Addr, draft: &Draft) -> Vec<u8> {
    let from = match &from.name {
        Some(name) => format!(
            "{} <{}>",
            encode_word(&format!("\"{}\"", name.replace('"', ""))),
            from.email
        ),
        None => from.email.clone(),
    };
    let mut head = vec![format!("From: {from}"), format!("To: {}", draft.to.trim())];
    if !draft.cc.trim().is_empty() {
        head.push(format!("Cc: {}", draft.cc.trim()));
    }
    head.push(format!("Subject: {}", encode_word(&draft.subject)));
    if let Some(id) = &draft.in_reply_to {
        head.push(format!("In-Reply-To: {id}"));
        let mut references = draft.references.clone();
        if !references.contains(id) {
            references.push(id.clone());
        }
        head.push(format!("References: {}", references.join(" ")));
    }
    head.extend([
        "MIME-Version: 1.0".to_string(),
        "Content-Type: text/plain; charset=UTF-8".to_string(),
        "Content-Transfer-Encoding: base64".to_string(),
    ]);
    let body = STANDARD.encode(draft.body.replace("\r\n", "\n").replace('\n', "\r\n"));
    let lines: Vec<&str> = body
        .as_bytes()
        .chunks(76)
        .map(|c| std::str::from_utf8(c).unwrap_or_default())
        .collect();
    format!("{}\r\n\r\n{}\r\n", head.join("\r\n"), lines.join("\r\n")).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_header_addresses() {
        assert_eq!(
            parse_addr("\"Ian from Ramp\" <hi@ramp.com>"),
            Addr {
                name: Some("Ian from Ramp".into()),
                email: "hi@ramp.com".into()
            }
        );
        assert_eq!(
            parse_addr("a@b.co"),
            Addr {
                name: None,
                email: "a@b.co".into()
            }
        );
    }

    #[test]
    fn reply_threads_by_headers() {
        let draft = Draft {
            to: "hi@ramp.com".into(),
            cc: String::new(),
            subject: "Re: Héllo".into(),
            body: "Thanks!\nSee you".into(),
            thread_id: Some("t1".into()),
            in_reply_to: Some("<b@x>".into()),
            references: vec!["<a@x>".into()],
        };
        let raw = String::from_utf8(build(
            &Addr {
                name: Some("Me".into()),
                email: "me@gmail.com".into(),
            },
            &draft,
        ))
        .unwrap();
        assert!(raw.contains("In-Reply-To: <b@x>\r\n"));
        assert!(raw.contains("References: <a@x> <b@x>\r\n"));
        assert!(raw.contains("Subject: =?UTF-8?B?"));
        assert!(!raw.contains("Cc:"));

        let parsed = parse("m1", raw.as_bytes(), 0, vec![]).unwrap();
        assert_eq!(parsed.subject, "Re: Héllo");
        assert_eq!(
            parsed.text.as_deref().map(|t| t.replace("\r\n", "\n")),
            Some("Thanks!\nSee you".into())
        );
        assert_eq!(parsed.html, None);
        assert_eq!(parsed.from.email, "me@gmail.com");
    }

    #[test]
    fn parses_html_and_references() {
        let raw = b"From: Ann <ann@x.com>\r\nTo: me@gmail.com\r\nSubject: Hi\r\nMessage-ID: <c@x>\r\nReferences: <a@x> <b@x>\r\n\
MIME-Version: 1.0\r\nContent-Type: multipart/alternative; boundary=b\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nplain\r\n\
--b\r\nContent-Type: text/html\r\n\r\n<p>rich</p>\r\n--b--\r\n";
        let parsed = parse("m2", raw, 0, vec![]).unwrap();
        assert_eq!(parsed.message_id.as_deref(), Some("<c@x>"));
        assert_eq!(parsed.references, vec!["<a@x>", "<b@x>"]);
        assert!(parsed.html.unwrap().contains("rich"));
        assert_eq!(parsed.from.name.as_deref(), Some("Ann"));
    }
}
