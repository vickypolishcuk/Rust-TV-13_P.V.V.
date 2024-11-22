use warp::Filter;
use warp::ws::{Message, WebSocket};
use futures::{FutureExt, StreamExt};
use std::sync::{Arc, Mutex};

// Список користувачів, що зараз підключені. Кожен користувач має канал для отримання повідомлень.
type Users = Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;
// Arc - Забезпечує багатопотоковий доступ. Mutex - Гарантує синхронний доступ до списку користувачів.
// Історія повідомлень чату
type History = Arc<tokio::sync::Mutex<Vec<String>>>;

#[tokio::main]
async fn main() {
    // Створюємо список користувачів (users) та історію повідомлень 
    let users = Users::default();
    let history = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let chat_route = warp::path("chat")
        .and(warp::ws())
        .and(with_users(users.clone()))
        .and(with_history(history.clone()))
        .map(|ws: warp::ws::Ws, users, history| {
            ws.on_upgrade(move |socket| handle_connection(socket, users, history))
        });

    println!("Запуск сервера на http://localhost:3030");
    warp::serve(chat_route).run(([127, 0, 0, 1], 3030)).await;
}

// Забезпечує доступ до глобального списку користувачів.
fn with_users(
    users: Users,
) -> impl Filter<Extract = (Users,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || users.clone())
}

// Забезпечує доступ до глобальної історії повідомлень.
fn with_history(
    history: History,
) -> impl Filter<Extract = (History,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || history.clone())
}

// Функція для підключення користувачів до сервера
async fn handle_connection(ws: WebSocket, users: Users, history: History) {
    // user_ws_tx - Для надсилання повідомлень. user_ws_rx - Для отримання повідомлень.
    let (user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let rx = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("Помилка надсилання повідомлення: {}", e);
        }
    }));

    // Send message history to the new user
    {
        let history_guard = history.lock().await;
        for message in history_guard.iter() {
            if let Err(e) = tx.send(Ok(Message::text(message.clone()))) {
                eprintln!("Помилка надсилання історії: {}", e);
            }
        }
    }

    users.lock().unwrap().push(tx);

    println!("Користувач підключився!");

    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    broadcast_message(msg, &users, &history).await;
                } else {
                    // Handle binary (image, file) messages
                    broadcast_binary_message(msg, &users).await;
                }
            }
            Err(e) => {
                eprintln!("Помилка отримання повідомлення: {}", e);
                break;
            }
        }
    }

    println!("Користувач відключився!");
}

// Функція для трансляції файлових повідомлень (фото, відео, аудіо, документ)
async fn broadcast_binary_message(msg: Message, users: &Users) {
    let binary_data = msg.into_bytes();  // Отримуємо бінарні дані (base64-кодоване зображення)

    let mut disconnected_users = vec![];
    let mut users_guard = users.lock().unwrap();

    for (index, user) in users_guard.iter().enumerate() {
        if let Err(_) = user.send(Ok(Message::binary(binary_data.clone()))) {
            disconnected_users.push(index);
        }
    }

    for &index in disconnected_users.iter().rev() {
        users_guard.remove(index);
    }
}

// Функція для трансляції текстових повідомлень
async fn broadcast_message(msg: Message, users: &Users, history: &History) {
    if let Ok(text) = msg.to_str() {
        {
            let mut history_guard = history.lock().await;
            history_guard.push(text.to_string());
        }
        let mut disconnected_users = vec![];
        let mut users_guard = users.lock().unwrap();

        for (index, user) in users_guard.iter().enumerate() {
            if let Err(_) = user.send(Ok(Message::text(text))) {
                disconnected_users.push(index);
            }
        }

        for &index in disconnected_users.iter().rev() {
            users_guard.remove(index);
        }
    }
}