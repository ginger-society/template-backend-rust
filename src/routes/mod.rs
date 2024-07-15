use crate::middlewares::jwt::Claims;
use crate::models::response::MessageResponse;
use diesel::r2d2;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use okapi::openapi3::{Parameter, ParameterValue};
use r2d2_redis::RedisConnectionManager;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use rocket::serde::json::Json;
use rocket::{tokio, State};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::openapi;
use rocket_okapi::request::OpenApiFromRequest;
use rocket_okapi::request::RequestHeaderInput;
use IAMService::apis::configuration::{ApiKey, Configuration};
use IAMService::apis::default_api::{routes_index, routes_protected_route};
use IAMService::get_configuration;
use IAMService::models::MessageResponse as OtherMessageResponse;
/// This is a description. <br />You can do simple html <br /> like <b>this<b/>
#[openapi()]
#[get("/")]
pub fn index(
    rdb: &State<r2d2::Pool<ConnectionManager<PgConnection>>>,
    cache: &State<Pool<RedisConnectionManager>>,
) -> Json<MessageResponse> {
    Json(MessageResponse {
        message: "Ok".to_string(),
    })
}

async fn get_healthcheck(
    openapi_configuration: &Configuration,
) -> Result<OtherMessageResponse, Box<dyn std::error::Error>> {
    match routes_index(openapi_configuration).await {
        Ok(response) => Ok(response),
        Err(e) => Err(Box::new(e)),
    }
}

pub struct AuthHeader(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthHeader {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            Some(header) => Outcome::Success(AuthHeader(header.to_string())),
            None => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}
use rocket_okapi::Result as OpenApiResult;

impl<'a> OpenApiFromRequest<'a> for AuthHeader {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        name: String,
        required: bool,
    ) -> OpenApiResult<RequestHeaderInput> {
        let parameter = Parameter {
            name,
            location: "header".to_string(),
            description: Some("Authorization header".to_string()),
            required,
            deprecated: false,
            allow_empty_value: false, // Set to `false` if the header must not be empty
            extensions: Default::default(),
            value: ParameterValue::Schema {
                schema: Default::default(), // You would define the schema here if needed
                style: None,
                explode: None,
                allow_reserved: false,
                example: None,
                examples: None,
            },
        };

        let header_input = RequestHeaderInput::Parameter(parameter);
        Ok(header_input)
    }
}

#[openapi()]
#[get("/route2")]
pub async fn route2(
    rdb: &State<r2d2::Pool<ConnectionManager<PgConnection>>>,
    cache: &State<Pool<RedisConnectionManager>>,
    claims: Claims,
    auth_header: AuthHeader,
) -> Json<MessageResponse> {
    let mut api_config = get_configuration();

    api_config.api_key = Some(ApiKey {
        key: auth_header.0.clone(),
        prefix: None, // You can set this to `Some("Bearer")` if needed
    });

    match routes_protected_route(&api_config).await {
        Ok(status) => println!("{:?}", status),
        Err(e) => {
            // Handle the error appropriately
            println!("{:?}", e);
            return Json(MessageResponse {
                message: "Health check failed".to_string(),
            });
        }
    }

    Json(MessageResponse {
        message: format!("Hello, {}! This is a protected route.", claims.user_id),
    })
}
