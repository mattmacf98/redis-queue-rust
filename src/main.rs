use hyper::{Body, Request, Response, Server};
use hyper::body;
use hyper::http::StatusCode;
use hyper::service::{make_service_fn, service_fn};
use std::net::SocketAddr;
use std::env;
use std::{thread, time};
use serde::{Serialize, Deserialize};
use serde_json;
use bytes::{BufMut, BytesMut};

mod tasks;
use tasks::{
    add::AddTask,
    subtract::SubtractTask,
    multiply::MultiplyTask
};
use crate::tasks::{TaskMessage, TaskType};
use crate::tasks::TaskType::{ADD, MULTIPLY, SUBTRACT};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingBody {
    pub one: i32,
    pub two: i32
}

async fn handle(req: Request<Body>) -> Result<Response<Body>, &'static str> {
    let client =  redis::Client::open(env::var("REDIS_URL").unwrap()).unwrap();
    let task_type = req.uri().to_string().replace("/", "");

    let bytes = body::to_bytes(req.into_body()).await.unwrap();
    let response_body: IncomingBody = serde_json::from_slice(&bytes).unwrap();

    let message_type: TaskType;
    match task_type.as_str() {
        "add" => {
            let body = AddTask{ one: response_body.one, two: response_body.two };
            message_type = TaskType::ADD(body);
        },
        "multiply" => {
            let body = MultiplyTask{ one: response_body.one, two: response_body.two };
            message_type = TaskType::MULTIPLY(body);
        },
        "subtract" => {
            let body = SubtractTask{ one: response_body.one, two: response_body.two };
            message_type = TaskType::SUBTRACT(body);
        }
        _ => {
            let response = Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("task not found"));
            return Ok(response.unwrap());
        }
    };

    let message = TaskMessage{ task: message_type };
    let serialized_message = bincode::serialize(&message).unwrap();

    let mut con = client.get_connection().unwrap();
    let _: () = redis::cmd("LPUSH")
        .arg("some_queue")
        .arg(serialized_message.clone())
        .query(&mut con).unwrap();

    Ok(Response::new(Body::from("task sent!")))
}

#[tokio::main]
async fn main() {
    let app_type = env::var("APP_TYPE").unwrap();

    match app_type.as_str() {
        "server" => {
            let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
            let server = Server::bind(&addr).serve(make_service_fn(|_conn| {
                async {
                    Ok::<_, hyper::Error>(service_fn(move |req| {
                        async {
                            handle(req).await
                        }
                    }))
                }
            }));

            if let Err(e) = server.await {
                eprintln!("server error: {}", e);
            }
        },
        "worker" => {
            let client =  redis::Client::open(env::var("REDIS_URL").unwrap()).unwrap();

            loop {
                let outcome: Option<Vec<u8>> = {
                    let mut con = client.get_connection().unwrap();

                    redis::cmd("RPOP")
                        .arg("some_queue")
                        .query(&mut con)
                        .unwrap()
                };

                match outcome {
                    Some(data) => {
                        let deserialized_message: TaskMessage = bincode::deserialize(&data).unwrap();
                        match deserialized_message.task {
                            ADD(task) => {
                                println!("Add: {}", task.run());
                            },
                            MULTIPLY(task) => {
                                println!("Multiply: {}", task.run());
                            },
                            SUBTRACT(task) => {
                                println!("Subtract: {}", task.run());
                            }
                        }
                    }
                    None => {
                        println!("Task queue empty");
                        let five_seconds = time::Duration::from_secs(5);
                        tokio::time::sleep(five_seconds).await;
                    }
                }
                }
            },
            _ => panic!("{} app type is not supported", app_type)
    }
}
