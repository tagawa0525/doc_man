use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = doc_man::app();
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind");

    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.expect("server error");
}
