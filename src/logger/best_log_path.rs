use std::path::PathBuf;

pub fn get_best_log_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        let exe_path = std::env::current_exe().unwrap();
        let exe_dir = exe_path.parent().unwrap();
        return exe_dir.join("logs");
    }

    if cfg!(target_os = "windows") {
        // On Windows, try LOCALAPPDATA first
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            PathBuf::from(local_app_data)
                .join("emmylua_dap")
                .join("logs")
        } else {
            // Fall back to the directory next to the executable
            let exe_path = std::env::current_exe().unwrap();
            let exe_dir = exe_path.parent().unwrap();
            exe_dir.join("logs")
        }
    } else {
        // On non-Windows platforms, try XDG_DATA_HOME first
        if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg_data_home)
                .join("emmylua_dap")
                .join("logs")
        } else {
            // If XDG_DATA_HOME is not set, use default XDG path
            if let Ok(home) = std::env::var("HOME") {
                PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("emmylua_dap")
                    .join("logs")
            } else {
                // Fall back to the directory next to the executable
                let exe_path = std::env::current_exe().unwrap();
                let exe_dir = exe_path.parent().unwrap();
                exe_dir.join("logs")
            }
        }
    }
}
