use tuono_lib::axum::response::{IntoResponse, Json};
use tuono_lib::axum::http::StatusCode;
use tuono_lib::Request;

use serde_json::json;

#[tuono_lib::api(POST)]
async fn post_test(_req: Request) -> impl IntoResponse{

    // Simulate some processing
    let response = json!({
        "message": "Post test successful",
        "status": "ok"
    });

    // Return a JSON response with status code 200 OK
    (StatusCode::OK)
}