// main.rs
mod command_executor;
mod worker_communication;
use std::sync::mpsc;
use std::thread;

#[tokio::main]
async fn main() {
    let worker_url = "https://serverworker.adoba.workers.dev/";
    let glances_url = "https://glances-server.adoba.workers.dev/";

    // Create channels for each command
    let (bash_tx, bash_rx) = mpsc::channel();
    let (glances_tx, glances_rx) = mpsc::channel();

    // Spawn a separate thread to execute the glances command
    let glances_handle = thread::spawn(move || {
        if let Err(err) = command_executor::execute_glances_command(glances_tx) {
            eprintln!("Error executing glances command: {:?}", err);
        } else {
            eprintln!("good job");
        }
    });

    // Spawn another thread to execute the tritonserver command
    let bash_handle = thread::spawn(move || {
        if let Err(err) = command_executor::execute_bash_command(bash_tx) {
            eprintln!("Error executing bash command: {:?}", err);
        } else {
            eprintln!("bash job");
        }
    });

    // Handle output from both commands concurrently
    let glances_worker = handle_glances_output(glances_rx, glances_url);
    let bash_worker = handle_bash_output(bash_rx, worker_url);

    // Wait for both workers to complete
    tokio::join!(glances_worker, bash_worker);
}

async fn handle_glances_output(glances_rx: mpsc::Receiver<String>, glances_url: String) {
    for command_output in glances_rx {
        println!("Received output from glances command");
        if !command_output.trim().is_empty() {
            println!("Output from sudo glances command:");
            println!("{}", command_output);
            // Send data to the Glances Worker
            let response = match worker_communication::send_glances_data_request(&glances_url, &command_output).await {
                Ok(response) => response,
                Err(e) => {
                    println!("Error: {}", e);
                    continue;
                }
            };
            // Check the response from the Glances Worker
            if response == "execute_glances_command" {
                println!("Received execute_glances_command response from Glances Worker");
            } else {
                println!("Received unknown response from Glances Worker: {}", response);
            }
        }
    }
}

async fn handle_bash_output(bash_rx: mpsc::Receiver<String>, worker_url: String) {
    for command_output in bash_rx {
        println!("Request to Cloudflare Worker was successful. Printing something else.");
        println!("{}", command_output);
        let response = match worker_communication::send_data_request(&worker_url, &command_output).await {
            Ok(response) => response,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };
        // Check the response from the Worker
        if response == "execute_bash_command" {
            println!("Received execute_bash_command response from Worker");
        } else {
            println!("Received unknown response from Worker, issue is with bash: {}", response);
        }
    }
}