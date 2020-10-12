use crate::db::DbPool;
use crate::models::GhUserRecord;
use crate::template_context::{ Breadcrumbs, BreadcrumbsContext };
use reqwest::Client as ReqwestClient;
use rocket::{ get, http::Cookie, http::CookieJar, response::Responder, State };
use rocket_contrib::templates::Template;
use serde::{ Deserialize, Serialize };
use thiserror::Error;


/// Describes the two bits of information needed from Github itself to
/// successfully complete an OAuth workflow with them. These need to be loaded
/// when the program starts and "wired" in.
pub struct GhCredentials {
    /// The github client id. This one gets exposed publicly.
    pub client_id: String,

    /// The secret key that is known only to us on the server and to Github.
    /// Keep this one private!
    pub client_secret: String,
}

/// Configures a Reqwest client that is compatible with what Github requires of
/// HTTP clients interacting with it. In this case, it means having a User-Agent
/// string in the header.
pub fn gh_client() -> ReqwestClient {
    reqwest::ClientBuilder::new()
        // github requires that a user agent be set to use its api
        .user_agent("Rust/reqwest/uDevGames.com")
        .build()
        .unwrap()
}

/// Presents the login page. This is a simple page with a link to Github.com
/// which is where users start the authorization process. Other OAuth providers
/// may be supported in the future... but don't count on it.
#[get("/login")]
pub fn login_with_github(gh_credentials: State<GhCredentials>) -> Template {
    #[derive(Serialize)]
    struct Context {
        oauth_url: String,
        breadcrumbs: BreadcrumbsContext,
        suppress_auth_controls: bool,
    };

    let context = Context {
        oauth_url: format!(
            "http://github.com/login/oauth/authorize?client_id={}",
            gh_credentials.client_id
        ),
        breadcrumbs: Breadcrumbs::from_crumbs(vec![]).to_context(),
        suppress_auth_controls: true,
    };

    Template::render("login", &context)
}

/// Responder which communicates the failure state of `gh_callback` to the
/// 500 handler.
#[derive(Responder)]
pub enum GithubCallbackError {
    /// An error authorizing the user. While is this destructured to a String
    /// here, I'm not totally sure that's the right way in the long term.
    #[response(status = 500)]
    AuthError(String)
}

/// Github will redirect users to this URL on successful authentication with a
/// code. This is exchanged with our secret key for an authorization, which we
/// can use to query the Github API as that user. Since we don't request any
/// scopes the only thing we can do is query our current identity, which is all
/// we wanted to do, anyway.
#[get("/gh_callback?<code>")]
pub async fn gh_callback(
    gh_credentials: State<'_, GhCredentials>,
    gh_client: State<'_, ReqwestClient>,
    db_pool: State<'_, DbPool>,
    cookies: &CookieJar<'_>,
    code: String
) -> Result<Template, GithubCallbackError> {
    let auth_result = auth_with_github(
        &gh_client, &db_pool, &gh_credentials, &code
    ).await;
    let user_record = match auth_result {
        Ok(user_record) => user_record,
        Err(e) => return Err(
            GithubCallbackError::AuthError(format!("{:?}", e))
        )
    };
    let cookie = Cookie::new(
        "gh_user_id", user_record.id.to_string()
    );

    cookies.add_private(cookie);

    #[derive(Serialize)]
    struct Context {
        suppress_auth_controls: bool,
        breadcrumbs: BreadcrumbsContext,
    };

    let context = Context {
        suppress_auth_controls: true,
        breadcrumbs: Breadcrumbs::from_crumbs(vec![]).to_context(),
    };

    Ok(Template::render("gh_callback", &context))
}

/// The response we get back from Github with our access token, which allows us
/// to make requests to the Github API as the user. Aside from `access_token` we
/// ignore the other fields as they are not relevant to us.
#[derive(Deserialize)]
struct AuthorizationResponse {
    access_token: String, token_type: String, scope: String,
}

impl std::fmt::Debug for AuthorizationResponse {
    /// This custom debug printer omits the access token, which prevents it
    /// from being logged. Insecurely storing the access token would allow any
    /// attacker to make requests to the Github API as our customer, which would
    /// be bad. As the program works, as soon as the authentication workflow is
    /// complete we throw away the access token, so as long as we don't log it,
    /// we have successfully protected our users.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f, "AuthorizationResponse {{ access_token: REDACTED, token_type: \
                {}, scope: {} }}", self.token_type, self.scope
        )
    }
}

/// Describes the error states coming from the `auth_with_github` function.
#[derive(Debug, Error)]
enum AuthWithGithubError {
    /// Errored exchanging the access code for an access token.
    #[error("Could not retrieve access token with error {0}")]
    AccessTokenRetrievalError(#[from] GetAccessTokenError),

    /// Errored calling `get_or_update_user_detail` due to a database or
    /// network issues.
    #[error("Could not query or update the database with error {0}")]
    DatabaseError(#[from] GetOrUpdateUserDetailError)
}

/// Authenticates with Github by exchanging the access code the user gave us for
/// an access token that Github issues us. Fetches the user's details from
/// Github and persists them to the database.
async fn auth_with_github(
    gh_client: &ReqwestClient, db_pool: &DbPool,
    gh_credentials: &GhCredentials, code: &String
) -> Result<GhUserRecord, AuthWithGithubError> {
    let authorization = get_access_token(
        &gh_client, &gh_credentials, &code
    ).await?;
    let user = get_or_update_user_detail(
        &gh_client, &db_pool, &authorization
    ).await?;

    Ok(user)
}

/// Error states for `get_access_token`.
#[derive(Debug, Error)]
enum GetAccessTokenError {
    /// Failed to query Github with some flavor of HTTP error.
    #[error("Could not add header with error {0}")]
    HttpError(#[from] reqwest::Error)
}

/// Exchange our access code for an access token.
async fn get_access_token(
    gh_client: &ReqwestClient, gh_credentials: &GhCredentials, code: &String
) -> Result<AuthorizationResponse, GetAccessTokenError> {
    let params = [
        ("client_id", gh_credentials.client_id.as_str()),
        ("client_secret", gh_credentials.client_secret.as_str()),
        ("code", code.as_str())
    ];

    let r = gh_client.post(
            "https://github.com/login/oauth/access_token"
        )
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await?
        .json::<AuthorizationResponse>()
        .await?;

    Ok(r)
}

/// The structure we map the user details from Github onto for internal user.
///
/// Broadly speaking, these are the only fields we're truly interested in from
/// Github. The id is the most important, for it is how we can durably refer to
/// a user even if they change their alias on Github. The login pre-populates
/// a user's identity on uDevGames, and the avatar and link to their github
/// might become useful in the future, though it's not a sure thing.
#[derive(Deserialize, Debug)]
struct UserResponse {
    id: i64, login: String, avatar_url: String, html_url: String
}

/// Error states for `get_or_update_user_detail`.
#[derive(Error, Debug)]
enum GetOrUpdateUserDetailError {
    /// We couldn't get or save the user to the database.
    #[error("Couldn't load data with error {0}. Call a DBA.")]
    DbLoadError(#[from] crate::models::ModelError),

    /// We couldn't get the user details from Github.
    #[error("Could not retrieve user details from Github with error {0}")]
    GetUserDetailError(#[from] GetUserDetailError)
}

/// Gets the user's details from Github (user id and login, most importantly),
/// then saves/updates those details in our database as a local cache of that
/// information.
async fn get_or_update_user_detail(
    gh_client: &ReqwestClient,
    db_pool: &DbPool,
    authorization: &AuthorizationResponse
) -> Result<GhUserRecord, GetOrUpdateUserDetailError> {
    let user = get_user_detail(
        &gh_client, &authorization.access_token
    ).await?;
    let gh_user_record = GhUserRecord::find_and_update_p(
        &db_pool, user.id, &user.login, &user.avatar_url, &user.html_url
    )?;

    Ok(gh_user_record)
}

/// Error states getting the user's details from Github.
#[derive(Debug, Error)]
enum GetUserDetailError {
    /// Error getting the user's details from Github.
    #[error("Could not request user detail from Github with error {0}")]
    HttpError(#[from] reqwest::Error)
}

/// Gets the user's details from Github.
async fn get_user_detail(
    gh_client: &ReqwestClient, access_token: &String
) -> Result<UserResponse, GetUserDetailError> {
   let r = gh_client.get("https://api.github.com/user")
        .header("Authorization", format!("token {}", access_token))
        .header("Accept", "application/json")
        .send()
        .await?
        .json()
        .await?;

    Ok(r)
}

/// Logs the user out. Pitches all the cookies we set.
#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Template {
    cookies.remove_private(Cookie::named("gh_user_id"));

    #[derive(Debug, Serialize)]
    struct Context {
        breadcrumbs: BreadcrumbsContext,
    }

    let context = Context {
        breadcrumbs: Breadcrumbs::from_crumbs(vec![]).to_context()
    };

    Template::render("logout", &context)
}

