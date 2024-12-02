//! Webicon: Fetch facvicons "fast".
//!
//! Copyright (C) 2024 Ari Prakash
//!
//! This program is free software:
//! you can redistribute it and/or modify it under the terms of the GNU Affero
//! General Public License as published by the Free Software Foundation, either
//! version 3 of the License, or (at your option) any later version.
//!
//! This program is
//! distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
//! even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
//! See the GNU Affero General Public License for more details.
//!
//! You should have
//! received a copy of the GNU Affero General Public License along with this program.
//! If not, see <https://www.gnu.org/licenses/>.

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
