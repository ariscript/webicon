#![allow(special_module_name)] // I tried...

mod lib;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("0.0.0.0:8787")
        .await
        .expect("tcp listener");
    axum::serve(listener, lib::router()).await.unwrap();
}
