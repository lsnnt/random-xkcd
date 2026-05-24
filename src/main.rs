use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use reqwest::Client;
use scraper::{Html, Selector};
// 
// Things i Learned
// ? can use only when Result is there otherwise we have to use ok_or then ?
// for future things use await
// 

async fn getimageurl() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let body = client
        .get("https://c.xkcd.com/random/comic/")
        .send()
        .await?
        .text()
        .await?;

    let document = Html::parse_document(&body);
    
    // If we dont use .map_err then We cannot convert the error type to the std::error::Error+Send+Sync
    
    let selector = Selector::parse(r#"meta[property="og:image"]"#)
    .map_err(|e| format!("Error: {e}"))?;

    let element = document
        .select(&selector)
        .next()
        .ok_or("meta tag not found")?;

    let content = element
        .value()
        .attr("content")
        .ok_or("missing content attribute")?;

    Ok(content.to_string())
}
// Cannot use Box<dyn std::error::Error+Send+Sync> here because the IntoResponse implementation requires StatusCode for 
async fn handler() -> Result<impl IntoResponse,StatusCode> {
    let url = getimageurl().await
    .map_err(|_|  StatusCode::INTERNAL_SERVER_ERROR)?;
    let client = Client::new();
    
    let resp = client.get(url)
    .send()
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let contenttype = resp
    .headers()
    .get(header::CONTENT_TYPE)
    .ok_or(StatusCode::BAD_GATEWAY)?.clone();
    
    let bytesforimg = resp.bytes()
    .await
    .map_err(|_| StatusCode::BAD_GATEWAY)
    ?;
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE,contenttype)],
        bytesforimg
    ))

}
#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(handler));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}