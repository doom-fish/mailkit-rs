use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::MailKitError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationMethodResult {
    pub method: String,
    pub version: Option<String>,
    pub result: String,
    pub reason: Option<String>,
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationResult {
    pub authserv_id: String,
    pub methods: Vec<AuthenticationMethodResult>,
}

pub type MEAuthenticationResult = AuthenticationResult;

impl AuthenticationResult {
    pub fn parse(header_value: &str) -> Result<Self, MailKitError> {
        let mut segments = header_value
            .split(';')
            .map(str::trim)
            .filter(|segment| !segment.is_empty());

        let authserv_id = segments
            .next()
            .ok_or_else(|| {
                MailKitError::InvalidArgument(
                    "Authentication-Results header did not contain an authserv-id".into(),
                )
            })?
            .to_string();

        let methods = segments
            .map(parse_method_segment)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            authserv_id,
            methods,
        })
    }

    pub fn parse_headers(
        headers: Option<&BTreeMap<String, Vec<String>>>,
    ) -> Result<Vec<Self>, MailKitError> {
        let Some(headers) = headers else {
            return Ok(Vec::new());
        };

        headers
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case("authentication-results"))
            .flat_map(|(_, values)| values.iter())
            .map(|value| Self::parse(value))
            .collect()
    }
}

fn parse_method_segment(segment: &str) -> Result<AuthenticationMethodResult, MailKitError> {
    let tokens = tokenize_auth_segment(segment);
    let Some(first) = tokens.first() else {
        return Err(MailKitError::InvalidArgument(
            "Authentication-Results method segment was empty".into(),
        ));
    };

    let (method_spec, result) = first.split_once('=').ok_or_else(|| {
        MailKitError::InvalidArgument(format!(
            "Authentication-Results method segment `{segment}` was missing `=`"
        ))
    })?;
    let (method, version) = method_spec
        .split_once('/')
        .map_or((method_spec, None), |(method, version)| {
            (method, Some(version.to_string()))
        });

    let mut reason = None;
    let mut properties = BTreeMap::new();
    for token in &tokens[1..] {
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };
        let value = trim_wrapping_quotes(value);
        if key.eq_ignore_ascii_case("reason") {
            reason = Some(value.to_string());
        } else {
            properties.insert(key.to_string(), value.to_string());
        }
    }

    Ok(AuthenticationMethodResult {
        method: method.to_string(),
        version,
        result: trim_wrapping_quotes(result).to_string(),
        reason,
        properties,
    })
}

fn trim_wrapping_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

fn tokenize_auth_segment(segment: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut comment_depth = 0_usize;

    for ch in segment.chars() {
        match ch {
            '"' if comment_depth == 0 => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            '(' if !in_quotes => {
                comment_depth += 1;
            }
            ')' if !in_quotes && comment_depth > 0 => {
                comment_depth -= 1;
            }
            ch if ch.is_whitespace() && !in_quotes && comment_depth == 0 => {
                if !current.is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ if comment_depth > 0 => {}
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current.trim().to_string());
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::{AuthenticationMethodResult, AuthenticationResult};

    #[test]
    fn parses_authentication_results_header() {
        let parsed = AuthenticationResult::parse(
            "mx.example.com; dkim=pass header.i=@example.com header.s=selector1; spf=pass smtp.mailfrom=example.com",
        )
        .expect("parse auth results");

        assert_eq!(parsed.authserv_id, "mx.example.com");
        assert_eq!(parsed.methods.len(), 2);
        assert_eq!(parsed.methods[0].method, "dkim");
        assert_eq!(parsed.methods[0].result, "pass");
        assert_eq!(
            parsed.methods[0],
            AuthenticationMethodResult {
                method: "dkim".into(),
                version: None,
                result: "pass".into(),
                reason: None,
                properties: [
                    ("header.i".into(), "@example.com".into()),
                    ("header.s".into(), "selector1".into()),
                ]
                .into_iter()
                .collect(),
            }
        );
    }

    #[test]
    fn preserves_reason_token_with_spaces() {
        let parsed = AuthenticationResult::parse(
            "mx.example.com; dmarc=fail reason=\"policy mismatch\" header.from=example.com",
        )
        .expect("parse auth results");

        assert_eq!(parsed.methods[0].reason.as_deref(), Some("policy mismatch"));
    }
}
