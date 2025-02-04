// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use serde_json::Value;
use reqwest::header::{AUTHORIZATION, HeaderValue};
// use tauri::command;
use tokio::runtime::Runtime;

// Learn more about Tauri commands at https://v1.tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_os_type() -> String {
    std::env::consts::OS.to_string() // Returns "windows", "macos", or "linux"
}

#[tauri::command]
fn console_log(msg: String) {
    println!("{}", msg);
}

async fn get_user_config(token: String) -> Result<Value, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://marte.izeeshan.dev/get-config")
        .header(AUTHORIZATION, HeaderValue::from_str(&token).map_err(|e| e.to_string())?)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    let json_response = response
        .json::<Value>()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(json_response)
}

async fn get_function_call_scripts(token: String) -> Result<Value, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://marte.izeeshan.dev/get-function-call-scripts")
        .header(AUTHORIZATION, HeaderValue::from_str(&token).map_err(|e| e.to_string())?)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    let json_response = response
        .json::<Value>()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(json_response)
}

async fn get_env_scripts(platform: String) -> Result<Value, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("https://marte.izeeshan.dev/env_script/{}", platform))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json_response = response
        .json::<Value>()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(json_response)
}

#[tauri::command]
fn execute_powershell_script(psscript: String, token: String) -> Result<Value, String> {
    let platform = get_os_type();
    println!("Platform: {}", platform);

    let rt = Runtime::new().map_err(|e| e.to_string())?;

    if platform.to_lowercase() == "windows" {
        let marte_dir = r#"C:\Program Files (x86)\Marte"#;
        let script_path = format!(r#"{}\setup_env.ps1"#, marte_dir);
        
        // Create directory if it doesn't exist
        if !std::path::Path::new(&marte_dir).exists() {
            std::fs::create_dir_all(&marte_dir).map_err(|e| e.to_string())?;
        }

        if !std::path::Path::new(&script_path).exists() { // if the script doesn't exist
            let script: Value = rt.block_on(get_env_scripts(platform))?;
            
            // Save the script to a file
            std::fs::write(&script_path, script["content"].as_str().unwrap()).map_err(|e| e.to_string())?;

            // call powershell script with elevated access
            let output = Command::new("powershell")
                .arg("-ExecutionPolicy")
                .arg("ByPass")
                .arg("-Command")
                .arg(format!("Start-Process powershell -ArgumentList '-ExecutionPolicy ByPass -File {}' -Verb RunAs", script_path))
                .output()
                .expect("Failed to execute PowerShell command with elevated access");
                
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                println!("Output: {}", stdout);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                println!("Error: {}", stderr);
            }
        }
        else {
            println!("Script already exists at {}", script_path);
        }
    }
    else if platform.to_lowercase() == "macos" {
        let marte_dir = format!("{}/.marte", std::env::var("HOME").unwrap());
        let script_path = format!("{}/setup_env.sh", marte_dir);

        if !std::path::Path::new(&script_path).exists() { // if the script doesn't exist
            let script: Value = rt.block_on(get_env_scripts(platform))?;
            
            // Save the script to a file
            std::fs::write(&script_path, script["content"].as_str().unwrap()).map_err(|e| e.to_string())?;

            // call powershell script
            let output = Command::new("sh")
                .arg("-c")
                .arg(&script_path)
                .output()
                .expect("Failed to execute PowerShell command");
                
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                println!("Output: {}", stdout);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                println!("Error: {}", stderr);
            }
        }
        else {
            println!("Script already exists at {}", script_path);
        }
    }

    let python_path = "C:\\Program Files (x86)\\Marte\\venv\\Scripts\\python.exe";
    let temp_path = std::env::temp_dir().join("operate_scripts");
    let script_path = temp_path.join("script.py");
    let config_path = temp_path.join("config.json");
    let function_call_scripts_path = temp_path.join("function_call_scripts.py");

    println!("Python path: {}", python_path);
    println!("Temp path: {}", temp_path.display());
    println!("Script path: {}", script_path.display());
    println!("Config path: {}", config_path.display());

    // Use a runtime to run the async function
    
    let config: Value = rt.block_on(get_user_config(token.clone()))?;
    let config_str = serde_json::to_string(&config).map_err(|e| e.to_string())?;

    let function_call_scripts: Value = rt.block_on(get_function_call_scripts(token))?;
    let function_call_scripts_content = function_call_scripts["content"].as_str().unwrap();

    std::fs::create_dir_all(&temp_path).map_err(|e| e.to_string())?;
    std::fs::write(&script_path, psscript).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, config_str).map_err(|e| e.to_string())?;
    std::fs::write(&function_call_scripts_path, function_call_scripts_content).map_err(|e| e.to_string())?;

    // Execute the Python script
    let output = Command::new(python_path)
        .arg(&script_path)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        println!("Output: {}", stdout);
        Ok(serde_json::Value::String(stdout))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(stderr)
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, console_log, execute_powershell_script])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}