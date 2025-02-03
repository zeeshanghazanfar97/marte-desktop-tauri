#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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

#[tauri::command]
fn execute_powershell_script(psscript: String, token: String) -> Result<Value, String> {
    println!("PS Script: {}", psscript);
    println!("Token: {}", token);
    let script = r#"
    if (Get-Command "uv" -ErrorAction SilentlyContinue) {
        Write-Host "uv installed already."
    } else {
        powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"
    }
    $uvPath = "$HOME/.local/bin/uv.exe"
    if (Test-Path $uvPath) {
        & $uvPath python install 3.11
        Write-Host "Python 3.11 installation initiated using uv."
        $programFiles = "C:\Program Files (x86)"
        $path_venv = Join-Path -Path $programFiles -ChildPath "Marte/venv"
        if (-Not (Test-Path $path_venv)) {
            & $uvPath venv $path_venv --python 3.11
            Write-Host "Virtual environment created at $path_venv using Python 3.11."
        } else {
            Write-Host "Virtual environment already exists at $path_venv."
        }
        Set-Location $path_venv
        & $uvPath pip install fastapi uvicorn requests beautifulsoup4 pyttsx3 cryptography pyperclip googlesearch-python
        Write-Host "Pip libraries installed with uv."
        exit 0
    } else {
        Write-Host "uv executable not found at $uvPath."
        exit 1
    }
    "#;

    let output = Command::new("powershell")
        .args(&["-Command", script])
        .output()
        .expect("Failed to execute PowerShell script");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Output: {}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error: {}", stderr);
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
    let rt = Runtime::new().map_err(|e| e.to_string())?;
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