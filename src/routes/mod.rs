use ginger_shared_rs::rocket_models::MessageResponse;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

/// This is a description. <br />You can do simple html <br /> like <b>this<b/>
#[openapi()]
#[get("/")]
pub fn index() -> Json<MessageResponse> {
    Json(MessageResponse {
        message: "Ok".to_string(),
    })
}
