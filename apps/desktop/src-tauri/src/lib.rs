#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_returns_expected_message() {
        let result = greet("AirBridge");
        assert_eq!(result, "Hello, AirBridge! You've been greeted from Rust!");
    }

    #[test]
    fn greet_handles_empty_name() {
        let result = greet("");
        assert!(result.contains("Hello, !"));
    }

    #[test]
    fn greet_handles_unicode_name() {
        let result = greet("Ünïcödé");
        assert!(result.starts_with("Hello, Ünïcödé!"));
    }
}
