use futures::{stream, StreamExt};
use hyper::client::HttpConnector;
use hyper::Client;
use hyper::{Body, Method, Request, Response};
use serde_json::json;
use std::ops::Range;
use std::time::{Duration, Instant};

mod opts;

use crate::opts::Opts;
use clap::Parser;

async fn seed() {
    let query = json!({
        "variables": {},
        "query": r#"mutation {
            upsertOneAccount(where: {email: "test@test.com"}, create: {email: "test@test.com", balance: 100}, update: {balance: 100}) {
                id
                email
                balance
            }
        }"#
    });

    let body = serde_json::to_vec(&query).unwrap();

    let req = Request::builder()
        .method(Method::POST)
        .uri("http://127.0.0.1:4466")
        .body(Body::from(body))
        .unwrap();

    let client = Client::new();
    client.request(req).await.unwrap();
}

async fn start_tx(client: &Client<HttpConnector>, timeout: u32, max_wait: u32) -> Option<String> {
    let body = serde_json::json!({
        "max_wait": max_wait,
        "timeout": timeout
    });

    let body_bytes = serde_json::to_vec(&body).unwrap();

    let req = Request::builder()
        .uri("http://127.0.0.1:4466/transaction/start")
        .method(Method::POST)
        .body(Body::from(body_bytes))
        .unwrap();

    // let client = Client::new();
    println!("GETTING NEW TX");
    let resp = client.request(req).await.unwrap();

    let json_resp = response_to_json(resp).await;
    println!("TT {json_resp}");
    let obj = json_resp.as_object().unwrap();

    if !obj.contains_key("id") {
        println!("FAILED TO GET TX");
        return None;
    }
    let tx_id = obj.get("id").unwrap().as_str().unwrap();

    Some(tx_id.into())
}

async fn commit_tx(client: &Client<HttpConnector>, tx_id: &str) {
    let uri = format!("http://127.0.0.1:4466/transaction/{}/commit", tx_id);

    let req = Request::builder()
        .uri(uri.as_str())
        .method(Method::POST)
        .body(Body::from(r#"{}"#))
        .unwrap();

    let req_fut = client.request(req);

    match tokio::time::timeout(Duration::from_millis(5000), req_fut).await {
        Ok(result) => match result {
            Ok(resp) => {
                let result = response_to_json(resp).await;
                println!("commit {:?}: {:?}", tx_id, result.to_string());
            }
            Err(e) => println!("Network error: {:?}", e),
        },
        Err(_) => println!("Commit Timeout: no response for {:?}", tx_id),
    };
}

async fn rollback_tx(client: &Client<HttpConnector>, tx_id: &str) {
    let uri = format!("http://127.0.0.1:4466/transaction/{}/rollback", tx_id);

    let req = Request::builder()
        .uri(uri.as_str())
        .method(Method::POST)
        .body(Body::from(r#"{}"#))
        .unwrap();

    let req_fut = client.request(req);

    match tokio::time::timeout(Duration::from_millis(5000), req_fut).await {
        Ok(result) => match result {
            Ok(resp) => {
                let result = response_to_json(resp).await;
                println!("commit {:?}: {:?}", tx_id, result.to_string());
            }
            Err(e) => println!("Network error: {:?}", e),
        },
        Err(_) => println!("Commit Timeout: no response for {:?}", tx_id),
    };
}

async fn batch_update_balance(client: &Client<HttpConnector>, tx_id: &str) {
    let query1 = json!({
        "variables":{},
        "query": r#"mutation {
            updateOneAccount(
                data: {
                    balance: { decrement: 100 }
                }
                where: { 
                    email: "test@test.com"
                }) { 
                    id,
                    email,
                    balance 
                }
            }"#
    });

    let query2 = json!({
        "variables":{},
        "query": r#"mutation {
            updateOneAccount(
                data: {
                    balance: { decrement: 200 }
                }
                where: { 
                    email: "test@test.com"
                }) { 
                    id,
                    email,
                    balance 
                }
            }"#
    });

    let batch = json!({
        "batch": [query1, query2],
        "transaction": true,
    });

    let body = serde_json::to_vec(&batch).unwrap();
    let req = Request::builder()
        .method(Method::POST)
        .uri("http://127.0.0.1:4466")
        .header("X-transaction-id", tx_id)
        .body(Body::from(body))
        .unwrap();

    // let resp = client.request(req).await.unwrap();
    let req_fut = client.request(req);

    match tokio::time::timeout(Duration::from_millis(5000), req_fut).await {
        Ok(result) => match result {
            Ok(resp) => {
                let result = response_to_json(resp).await;
                println!("RESP {:?}: {:?}", tx_id, result.to_string());
            }
            Err(e) => println!("Network error: {:?}", e),
        },
        Err(_) => println!("Update Timeout: no response for {:?}", tx_id),
    };
}

async fn update_balance(client: &Client<HttpConnector>, tx_id: &str) {
    let query = json!({
        "variables":{},
        "query": r#"mutation {
            updateOneAccount(
                data: {
                    balance: { decrement: 100 }
                }
                where: { 
                    email: "test@test.com"
                }) { 
                    id,
                    email,
                    balance 
                }
            }"#
    });

    let body = serde_json::to_vec(&query).unwrap();
    let req = Request::builder()
        .method(Method::POST)
        .uri("http://127.0.0.1:4466")
        .header("X-transaction-id", tx_id)
        .body(Body::from(body))
        .unwrap();

    // let resp = client.request(req).await.unwrap();
    let req_fut = client.request(req);

    match tokio::time::timeout(Duration::from_millis(5000), req_fut).await {
        Ok(result) => match result {
            Ok(resp) => {
                let result = response_to_json(resp).await;
                println!("RESP {:?}: {:?}", tx_id, result.to_string());
            }
            Err(e) => println!("Network error: {:?}", e),
        },
        Err(_) => println!("Update Timeout: no response for {:?}", tx_id),
    };
}

async fn simple(attempts: u32, concurrency: u32, timeout: u32, max_wait: u32) {
    let attempts = Range {
        start: 0,
        end: attempts,
    };

    for i in attempts {
        println!("ATTEMPT NUMBER {i}");
        let concurrency = Range {
            start: 0,
            end: concurrency,
        };

        let fetches = stream::iter(concurrency.map(|_| async {
            let updates = 0..2;
            let client = Client::new();

            let tx_id = start_tx(&client, timeout, max_wait).await;
            if tx_id.is_none() {
                return;
            }
            let tx_id = tx_id.unwrap();
            println!("starting tx: {:?}", tx_id);
            let update_fut = stream::iter(updates.map(|_| async {
                update_balance(&client, &tx_id).await;
                batch_update_balance(&client, &tx_id).await;
            }))
            .buffer_unordered(1)
            .collect::<Vec<()>>();
            update_fut.await;
            commit_tx(&client, &tx_id).await;
            println!("commit tx: {:?}", tx_id);
        }))
        .buffer_unordered(60)
        .collect::<Vec<()>>();
        println!("Running...");
        fetches.await;
        println!("ATTEMPT DONE {i}");
    }
}

// Mixture of commits, rollbacks and just stops
async fn mixed(attempts: u32, concurrency: u32, timeout: u32, max_wait: u32) {
    let attempts = Range {
        start: 0,
        end: attempts,
    };

    for i in attempts {
        println!("ATTEMPT NUMBER {i}");
        let concurrency = Range {
            start: 0,
            end: concurrency,
        };

        let fetches = stream::iter(concurrency.map(|i| async move {
            let updates = 0..2;
            let client = Client::new();

            let tx_id = start_tx(&client, timeout, max_wait).await;
            if tx_id.is_none() {
                return;
            }
            let tx_id = tx_id.unwrap();
            println!("starting tx: {:?}", tx_id);
            let update_fut = stream::iter(updates.map(|_| async {
                update_balance(&client, &tx_id).await;
                batch_update_balance(&client, &tx_id).await;
            }))
            .buffer_unordered(1)
            .collect::<Vec<()>>();
            update_fut.await;
            if i % 2 == 0 {
                println!("Rolling back: {}", tx_id);
                return rollback_tx(&client, &tx_id).await;
            }

            if i % 3 == 0 {
                println!("stopping before commit {}", tx_id);
                return;
            }

            commit_tx(&client, &tx_id).await;
            println!("commit tx: {:?}", tx_id);
        }))
        .buffer_unordered(60)
        .collect::<Vec<()>>();
        println!("Running...");
        fetches.await;
        println!("ATTEMPT DONE {i}");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Opts::parse();

    let start = Instant::now();
    println!("Seeding");
    seed().await;

    println!(
        "Running {} attempts: {} concurrency: {}",
        args.name, args.iterations, args.concurrency
    );

    if args.name == "mixed".to_string() {
        mixed(args.iterations, args.concurrency, args.timeout, args.wait).await;
    } else {
        simple(args.iterations, args.concurrency, args.timeout, args.wait).await;
    }

    println!("requests took {:?}", start.elapsed());
    Ok(())
}

async fn response_to_json(resp: Response<Body>) -> serde_json::Value {
    let body_start = resp.into_body();
    let full_body = hyper::body::to_bytes(body_start).await.unwrap();

    serde_json::from_slice(full_body.as_ref()).unwrap()
}
