//! Authentication application logic.

use reqwest::RequestBuilder;

use crate::core::models::AuthStrategy;

/// Applies auth strategy onto request builder.
pub fn apply_auth(builder: RequestBuilder, auth: &AuthStrategy) -> RequestBuilder {
    match auth {
        AuthStrategy::None => builder,
        AuthStrategy::Basic { username, password } => {
            builder.basic_auth(username.to_owned(), Some(password.to_owned()))
        }
        AuthStrategy::Bearer { token } => builder.bearer_auth(token.to_owned()),
        AuthStrategy::ApiKey {
            key,
            value,
            in_header,
        } => {
            if *in_header {
                builder.header(key, value)
            } else {
                builder.query(&[(key, value)])
            }
        }
    }
}
