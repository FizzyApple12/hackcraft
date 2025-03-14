use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use reqwest::StatusCode;
use tokio::{
    sync::watch::{self},
    task::JoinHandle,
};

use super::EndpointModule;

pub const UPDATES_BASE_ENDPOINT: &str = "/updates";

#[derive(Clone)]
struct UpdateState {
    update_sender: watch::Sender<String>,
}

#[derive(Clone)]
enum SocketSenderMessage {
    NewUpdate(String),
    PingPong,
}

pub struct UpdatesModule {}

impl EndpointModule for UpdatesModule {
    fn create_router() -> Router {
        let latest_update: String = get_latest_release();

        let (update_sender, _) = watch::channel::<String>(latest_update);

        let state = UpdateState { update_sender };

        Router::new()
            .route("/", get(live))
            .route("/webhook", post(github_webhook))
            .with_state(state)
    }
}

async fn live(State(state): State<UpdateState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(state, socket))
}

async fn handle_socket(state: UpdateState, mut socket: WebSocket) {
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_err() {
        return;
    }

    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if let Message::Close(_) = msg {
                return;
            }
        } else {
            return;
        }
    }

    let (mut sender, mut receiver) = socket.split();

    let (send_task_sender, mut send_task_receiver) =
        watch::channel::<SocketSenderMessage>(SocketSenderMessage::PingPong);

    let send_task_sender_2 = send_task_sender.clone();
    let send_task_sender_3 = send_task_sender.clone();

    let mut update_receiver = state.update_sender.subscribe();

    let mut update_receiver_2 = state.update_sender.subscribe();

    let _ = send_task_sender.send(SocketSenderMessage::NewUpdate(
        update_receiver.borrow_and_update().clone(),
    ));

    let mut send_task: JoinHandle<()> = tokio::spawn(async move {
        loop {
            if (send_task_receiver.changed().await).is_ok() {
                let next_message = send_task_receiver.borrow_and_update().clone();

                match next_message {
                    SocketSenderMessage::NewUpdate(url) => {
                        if sender.send(Message::Text(url)).await.is_err() {
                            return;
                        }
                    }
                    SocketSenderMessage::PingPong => {
                        if sender
                            .send(Message::Ping(vec![
                                103, 111, 111, 100, 32, 109, 111, 114, 110, 105, 110, 103, 33, 33,
                                33,
                            ]))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                }
            }
        }
    });

    let mut update_watcher_task: JoinHandle<()> = tokio::spawn(async move {
        loop {
            if (update_receiver.changed().await).is_ok() {
                let _ = send_task_sender.send(SocketSenderMessage::NewUpdate(
                    update_receiver.borrow_and_update().clone(),
                ));
            }
        }
    });

    let mut ping_pong_task: JoinHandle<()> = tokio::spawn(async move {
        loop {
            let _ = send_task_sender_2.send(SocketSenderMessage::PingPong);

            tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
        }
    });

    let mut recv_task: JoinHandle<()> = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                return;
            }

            if let Message::Text(_) = msg {
                let _ = send_task_sender_3.send(SocketSenderMessage::NewUpdate(
                    update_receiver_2.borrow_and_update().clone(),
                ));
            }
        }
    });

    tokio::select! {
        _rv_a = (&mut send_task) => {
            update_watcher_task.abort();
            ping_pong_task.abort();
            recv_task.abort();
        },
        _rv_b = (&mut recv_task) => {
            send_task.abort();
            update_watcher_task.abort();
            ping_pong_task.abort();
        }
        _rv_c = (&mut update_watcher_task) => {
            send_task.abort();
            ping_pong_task.abort();
            recv_task.abort();
        }
        _rv_d = (&mut ping_pong_task) => {
            send_task.abort();
            update_watcher_task.abort();
            recv_task.abort();
        }
    }
}

async fn github_webhook(State(state): State<UpdateState>) -> StatusCode {
    let _ = state.update_sender.send(get_latest_release());

    StatusCode::OK
}

fn get_latest_release() -> String {
    // TODO: dkjhgkjhgfdsajhfdsakj
    "".to_string()
}
