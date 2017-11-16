

#[allow(unused_imports)]
use std::collections::HashMap;
use std::convert::From;
use std::error::Error;
use std::fmt;

use serde_json;

#[cfg(feature = "reqwest")]
use reqwest::unstable::async as reqwest;
#[cfg(feature = "reqwest")]
use futures::Future;

use requests::SlackWebRequestSender;

/// Lists conversations that the calling user has access to.
///
/// Wraps https://api.slack.com/methods/conversations.list

pub fn list<R>(
    client: &R,
    token: &str,
    request: &ListRequest,
) -> Result<ListResponse, ListError<R::Error>>
where
    R: SlackWebRequestSender,
{
    let limit = request.limit.map(|limit| limit.to_string());
    let params = vec![
        Some(("token", token)),
        request.exclude_archived.map(|exclude_archived| {
            ("exclude_archived", if exclude_archived { "1" } else { "0" })
        }),
        request.cursor.map(|cursor| ("cursor", cursor)),
        limit.as_ref().map(|limit| ("limit", &limit[..])),
        request.types.map(|types| ("types", types)),
    ];
    let params = params.into_iter().filter_map(|x| x).collect::<Vec<_>>();
    let url = ::get_slack_url_for_method("conversations.list");
    client
        .send(&url, &params[..])
        .map_err(ListError::Client)
        .and_then(|result| {
            serde_json::from_str::<ListResponse>(&result).map_err(ListError::MalformedResponse)
        })
        .and_then(|o| o.into())
}

#[cfg(feature = "reqwest")]
/// Lists conversations that the calling user has access to.
///
/// Wraps https://api.slack.com/methods/conversations.list

pub fn list_async(
    client: &reqwest::Client,
    token: &str,
    request: &ListRequest,
) -> impl Future<Item = ListResponse, Error = ListError<::reqwest::Error>> {
    let limit = request.limit.map(|limit| limit.to_string());
    let params = vec![
        Some(("token", token)),
        request.exclude_archived.map(|exclude_archived| {
            ("exclude_archived", if exclude_archived { "1" } else { "0" })
        }),
        request.cursor.map(|cursor| ("cursor", cursor)),
        limit.as_ref().map(|limit| ("limit", &limit[..])),
        request.types.map(|types| ("types", types)),
    ];
    let params = params.into_iter().filter_map(|x| x).collect::<Vec<_>>();
    let url = ::get_slack_url_for_method("conversations.list");
    let mut url = ::reqwest::Url::parse(&url).expect("Unable to parse url");
    url.query_pairs_mut().extend_pairs(params);
    client.get(url).send().map_err(ListError::Client).and_then(
        |mut result: reqwest::Response| result.json().map_err(ListError::Client),
    )
}

#[derive(Clone, Default, Debug)]
pub struct ListRequest<'a> {
    /// Don't return archived private channels.
    pub exclude_archived: Option<bool>,

    pub cursor: Option<&'a str>,

    pub limit: Option<u32>,

    pub types: Option<&'a str>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ListResponse {
    pub channels: Option<Vec<::Conversation>>,
    error: Option<String>,
    #[serde(default)]
    ok: bool,
}


impl<E: Error> Into<Result<ListResponse, ListError<E>>> for ListResponse {
    fn into(self) -> Result<ListResponse, ListError<E>> {
        if self.ok {
            Ok(self)
        } else {
            Err(self.error.as_ref().map(String::as_ref).unwrap_or("").into())
        }
    }
}
#[derive(Debug)]
pub enum ListError<E: Error> {
    /// No authentication token provided.
    NotAuthed,
    /// Invalid authentication token.
    InvalidAuth,
    /// Authentication token is for a deleted user or team.
    AccountInactive,
    /// The method was passed an argument whose name falls outside the bounds of common decency. This includes very long names and names with non-alphanumeric characters other than _. If you get this error, it is typically an indication that you have made a very malformed API call.
    InvalidArgName,
    /// The method was passed a PHP-style array argument (e.g. with a name like foo[7]). These are never valid with the Slack API.
    InvalidArrayArg,
    /// The method was called via a POST request, but the charset specified in the Content-Type header was invalid. Valid charset names are: utf-8 iso-8859-1.
    InvalidCharset,
    /// The method was called via a POST request with Content-Type application/x-www-form-urlencoded or multipart/form-data, but the form data was either missing or syntactically invalid.
    InvalidFormData,
    /// The method was called via a POST request, but the specified Content-Type was invalid. Valid types are: application/x-www-form-urlencoded multipart/form-data text/plain.
    InvalidPostType,
    /// The method was called via a POST request and included a data payload, but the request did not include a Content-Type header.
    MissingPostType,
    /// The team associated with your request is currently undergoing migration to an Enterprise Organization. Web API and other platform operations will be intermittently unavailable until the transition is complete.
    TeamAddedToOrg,
    /// The method was called via a POST request, but the POST data was either missing or truncated.
    RequestTimeout,
    /// The response was not parseable as the expected object
    MalformedResponse(serde_json::error::Error),
    /// The response returned an error that was unknown to the library
    Unknown(String),
    /// The client had an error sending the request to Slack
    Client(E),
}

impl<'a, E: Error> From<&'a str> for ListError<E> {
    fn from(s: &'a str) -> Self {
        match s {
            "not_authed" => ListError::NotAuthed,
            "invalid_auth" => ListError::InvalidAuth,
            "account_inactive" => ListError::AccountInactive,
            "invalid_arg_name" => ListError::InvalidArgName,
            "invalid_array_arg" => ListError::InvalidArrayArg,
            "invalid_charset" => ListError::InvalidCharset,
            "invalid_form_data" => ListError::InvalidFormData,
            "invalid_post_type" => ListError::InvalidPostType,
            "missing_post_type" => ListError::MissingPostType,
            "team_added_to_org" => ListError::TeamAddedToOrg,
            "request_timeout" => ListError::RequestTimeout,
            _ => ListError::Unknown(s.to_owned()),
        }
    }
}

impl<E: Error> fmt::Display for ListError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl<E: Error> Error for ListError<E> {
    fn description(&self) -> &str {
        match *self {
            ListError::NotAuthed => "not_authed: No authentication token provided.",
            ListError::InvalidAuth => "invalid_auth: Invalid authentication token.",
            ListError::AccountInactive => {
                "account_inactive: Authentication token is for a deleted user or team."
            }
            ListError::InvalidArgName => {
                "invalid_arg_name: The method was passed an argument whose name falls outside the bounds of common decency. This includes very long names and names with non-alphanumeric characters other than _. If you get this error, it is typically an indication that you have made a very malformed API call."
            }
            ListError::InvalidArrayArg => {
                "invalid_array_arg: The method was passed a PHP-style array argument (e.g. with a name like foo[7]). These are never valid with the Slack API."
            }
            ListError::InvalidCharset => {
                "invalid_charset: The method was called via a POST request, but the charset specified in the Content-Type header was invalid. Valid charset names are: utf-8 iso-8859-1."
            }
            ListError::InvalidFormData => {
                "invalid_form_data: The method was called via a POST request with Content-Type application/x-www-form-urlencoded or multipart/form-data, but the form data was either missing or syntactically invalid."
            }
            ListError::InvalidPostType => {
                "invalid_post_type: The method was called via a POST request, but the specified Content-Type was invalid. Valid types are: application/x-www-form-urlencoded multipart/form-data text/plain."
            }
            ListError::MissingPostType => {
                "missing_post_type: The method was called via a POST request and included a data payload, but the request did not include a Content-Type header."
            }
            ListError::TeamAddedToOrg => {
                "team_added_to_org: The team associated with your request is currently undergoing migration to an Enterprise Organization. Web API and other platform operations will be intermittently unavailable until the transition is complete."
            }
            ListError::RequestTimeout => {
                "request_timeout: The method was called via a POST request, but the POST data was either missing or truncated."
            }
            ListError::MalformedResponse(ref e) => e.description(),
            ListError::Unknown(ref s) => s,
            ListError::Client(ref inner) => inner.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ListError::MalformedResponse(ref e) => Some(e),
            ListError::Client(ref inner) => Some(inner),
            _ => None,
        }
    }
}