use aws_sdk_sts::operation::assume_role::AssumeRoleOutput;
use aws_smithy_types_convert::date_time::DateTimeExt;
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, HeaderName, HeaderValue},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use hyper::StatusCode;
use serde::Serialize;

use crate::app::AppState;

pub(crate) struct AuthorizationHeader;

#[async_trait]
impl FromRequestParts<AppState> for AuthorizationHeader {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match parts
            .headers
            .get(HeaderName::from_static("authorization"))
            .map(HeaderValue::to_str)
        {
            Some(Ok(token)) if token == state.auth_token => Ok(Self),
            Some(Ok(_)) => Err((StatusCode::FORBIDDEN, "Invalid authorization token")),
            Some(Err(_)) => Err((StatusCode::BAD_REQUEST, "Invalid authorization header")),
            None => Err((StatusCode::BAD_REQUEST, "Missing authorization header")),
        }
    }
}

/// Get credentials handler
/// This handler will return the credentials of the role arn by assuming the role
pub(crate) async fn get_credentials_handler(
    State(AppState {
        sts_client,
        role_arn,
        role_session_name,
        ..
    }): State<AppState>,
    _: AuthorizationHeader,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sts_client
        .assume_role()
        .role_arn(&role_arn)
        .role_session_name(&role_session_name)
        .send()
        .await;

    match result {
        Ok(AssumeRoleOutput {
            credentials: Some(credentials),
            ..
        }) => Ok(Json(transform_credential(credentials, role_arn.clone()))),
        _ => Err(StatusCode::BAD_GATEWAY),
    }
}

#[derive(Serialize, Debug)]
struct ContainerCredentialsProviderResponse {
    #[serde(rename = "AccessKeyId")]
    access_key_id: String,

    #[serde(rename = "SecretAccessKey")]
    secret_access_key: String,

    #[serde(rename = "Token")]
    token: String,

    #[serde(rename = "Expiration")]
    expiration: DateTime<Utc>,

    #[serde(rename = "RoleArn")]
    role_arn: String,
}

fn transform_credential(
    credentials: aws_sdk_sts::types::Credentials,
    role_arn: String,
) -> ContainerCredentialsProviderResponse {
    ContainerCredentialsProviderResponse {
        access_key_id: credentials.access_key_id,
        secret_access_key: credentials.secret_access_key,
        token: credentials.session_token,
        expiration: credentials
            .expiration
            .to_chrono_utc()
            .expect("Unexpected time received"),
        role_arn,
    }
}

#[cfg(test)]
mod test {
    // test date time serialize 2016-02-25T06:03:31Z
    #[test]
    fn test_serialize_chrono_datetime() {
        let dt = chrono::DateTime::parse_from_rfc3339("2016-02-25T06:03:31Z")
            .unwrap()
            .to_utc();
        let serialized = serde_json::to_string(&dt).unwrap();
        assert_eq!(serialized, "\"2016-02-25T06:03:31Z\"");
    }
}
