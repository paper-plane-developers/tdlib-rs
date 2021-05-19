use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tdgrand::{
    self,
    client,
    enums::{AuthorizationState, Update, User},
    types::TdlibParameters,
};
use tokio::sync::mpsc::{self, Receiver, Sender};

fn ask_user(string: &str) -> String {
    println!("{}", string);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

async fn handle_update(update: Update, auth_tx: &Sender<AuthorizationState>) {
    match update {
        Update::AuthorizationState(update) => {
            auth_tx.send(update.authorization_state).await.unwrap();
        }
        _ => ()
    }
}

async fn handle_authorization_state(client_id: i32, mut auth_rx: Receiver<AuthorizationState>, run_flag: Arc<AtomicBool>) -> Receiver<AuthorizationState> {
    while let Some(state) = auth_rx.recv().await {
        match state {
            AuthorizationState::WaitTdlibParameters => {
                let parameters = TdlibParameters {
                    database_directory: "get_me_db".to_string(),
                    api_id: env!("API_ID").parse::<i32>().unwrap(),
                    api_hash: env!("API_HASH").to_string(),
                    system_language_code: "en".to_string(),
                    device_model: "Desktop".to_string(),
                    application_version: env!("CARGO_PKG_VERSION").to_string(),
                    ..Default::default()
                };

                let response = client::set_tdlib_parameters(client_id, parameters).await;
                if let Err(error) = response {
                    println!("{}", error.message);
                }
            }
            AuthorizationState::WaitEncryptionKey(_) => {
                loop {
                    let input = ask_user("Enter your encryption key (you can leave this empty if you want):");
                    match client::check_database_encryption_key(client_id, input).await {
                        Ok(_) => break,
                        Err(e) => println!("{}", e.message),
                    }
                }
            }
            AuthorizationState::WaitPhoneNumber => {
                loop {
                    let input = ask_user("Enter your phone number (include the country calling code):");
                    match client::set_authentication_phone_number(client_id, input, Default::default()).await {
                        Ok(_) => break,
                        Err(e) => println!("{}", e.message),
                    }
                }
            }
            AuthorizationState::WaitCode(_) => {
                loop {
                    let input = ask_user("Enter the verification code:");
                    match client::check_authentication_code(client_id, input).await {
                        Ok(_) => break,
                        Err(e) => println!("{}", e.message),
                    }
                }
            }
            AuthorizationState::Ready => {
                break;
            }
            AuthorizationState::Closed => {
                // Set the flag to false to stop receiving updates from the
                // spawned task
                run_flag.store(false, Ordering::Release);
                break;
            }
            _ => ()
        }
    }

    auth_rx
}

#[tokio::main]
async fn main() {
    // Create the client object
    let client_id = tdgrand::crate_client();

    // Create a mpsc channel for handling AuthorizationState updates separately
    // from the task
    let (auth_tx, auth_rx) = mpsc::channel(5);

    // Create a flag to make it possible to stop receiving updates
    let run_flag = Arc::new(AtomicBool::new(true));
    let run_flag_clone = run_flag.clone();

    // Spawn a task to receive updates/responses
    let handle = tokio::spawn(async move {
        while run_flag_clone.load(Ordering::Acquire) {
            if let Some((update, _client_id)) = tdgrand::step() {
                handle_update(update, &auth_tx).await;
            }
        }
    });

    // Set a fairly low verbosity level. We mainly do this because tdlib
    // requires to perform a random request with the client to start receiving
    // updates for it.
    client::set_log_verbosity_level(client_id, 1).await.unwrap();

    // Handle the authorization state to authenticate the client
    let auth_rx = handle_authorization_state(client_id, auth_rx, run_flag.clone()).await;

    // Run the get_me() method to get user informations
    let User::User(me) = client::get_me(client_id).await.unwrap();
    println!("Hi, I'm {}", me.username);

    // Tell the client to close
    client::close(client_id).await.unwrap();

    // Handle the authorization state to wait for the "Closed" state
    handle_authorization_state(client_id, auth_rx, run_flag.clone()).await;

    // Wait for the previously spawned task to end the execution
    handle.await.unwrap();
}
