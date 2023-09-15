use std::net::SocketAddr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Form, Router,
};
//use rpcdemo::DEFAULT_ADDR;
use serde::Deserialize;
use volo_gen::myredis::{RedisServeClient,RedisServeClientBuilder};
use volo_gen::myredis::{Kv,Varible};
use pilota::FastStr;
static DEFAULT_ADDR:&str = "127.0.0.1:8080";
use mini_redis::FilterLayer;
#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let addr: SocketAddr = DEFAULT_ADDR.parse().unwrap();
    let rpc_cli = RedisServeClientBuilder::new("mini-redis")
    .layer_outer(FilterLayer)
    .address(addr).build();

    // build the application with router
    let app = Router::new()
        .route("/ping", get(ping))
        .route("/get/:keys", get(get_key).with_state(rpc_cli.clone()))
        .route(
            "/set",
            get(show_set_form).post(set_key).with_state(rpc_cli.clone()),
        )
        .route("/del", get(show_del_form).post(del_key).with_state(rpc_cli));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ping() -> (StatusCode, &'static str) {
    (StatusCode::OK, "pong")
}

/// Get a key
async fn get_key(Path(key): Path<String>, State(rpc_cli): State<RedisServeClient>) -> Response {
    let req = Varible{key:FastStr::from(key)}; //clone once; key is just a key 
    match rpc_cli.get_var(req).await{
        Ok(resp) =>{
            if resp.val == FastStr::from("Not existed"){
                (StatusCode::OK, "Not Found").into_response()
            }else{
                resp.val.to_string().into_response()
            }
        }
        Err(_resp) =>{
            (StatusCode::BAD_REQUEST, "Network error").into_response()
        }
        
    }
}

#[derive(Deserialize, Debug)]
struct FormKey {
    key: String,
    value: String,
}

/// Show the form for set a key
async fn show_set_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/set" method="post">
                    <label for="key">
                        Enter key:
                        <input type="text" name="key">
                        Enter value:
                        <input type="text" name="value">
                    </label>
                    <input type="submit" value="Subscribe!">
                </form>
            </body>
        </html>
        "#,
    )
}

/// Set a key
async fn set_key(State(rpc_cli): State<RedisServeClient>, Form(setkey): Form<FormKey>) -> Response {
    println!("{:?}",setkey.key);

    let req = Kv{key:FastStr::from(setkey.key.to_string()),val:FastStr::from(setkey.value.to_string())}; //cloned
        
    match rpc_cli.set_var(req).await{
        Ok(_resp) => {
            (StatusCode::OK, "set ok").into_response()
        }
        Err(_resp)=>{
            return (StatusCode::CONFLICT, "forbidden").into_response()
        }
    }

    
}

async fn show_del_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/del" method="post">
                    <label for="key">
                        Enter key:
                        <input type="text" name="key">
                    </label>
                    <input type="submit" value="Subscribe!">
                </form>
            </body>
        </html>
        "#,
    )
}

async fn del_key(
    State(rpc_cli): State<RedisServeClient>,
    delkey: String,
) -> (StatusCode, &'static str) {
    //rpc_cli.del(delkey.key.into()).await.unwrap();
    let req = Varible{key:FastStr::from(delkey)}; //clone once; key is just a key 
    match rpc_cli.del_var(req).await{
        Ok(resp) =>{
            if resp.content == FastStr::from("Not existed"){
                (StatusCode::OK, "Not Found")
            }else{
                (StatusCode::OK, "del ok")
            }
        }
        Err(_resp) =>{
            (StatusCode::BAD_REQUEST, "Network error")
        }
        
    }
    
}