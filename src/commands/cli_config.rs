use crate::{config::Config, utils::output};

pub fn show(config: &Config, json: bool) {
    let path = Config::path().map_or_else(|| "(unknown)".to_string(), |p| p.display().to_string());
    let token = config.resolve_token();

    if json {
        let token = token.map(|_| "<redacted>".to_string());
        println!(
            "{}",
            serde_json::json!({
                "config_path": path,
                "api_base_url": config.api_base_url,
                "token": token,
            })
        );
    } else {
        println!(
            "{}",
            output::kv_table_cells(vec![
                ("Config path", output::cell(path)),
                (
                    "API base URL",
                    config
                        .api_base_url
                        .as_deref()
                        .map_or_else(|| output::cell("(not set)"), output::url_cell)
                ),
                (
                    "Token",
                    output::cell(
                        token.map_or_else(|| "(not set)".to_string(), |_| "<redacted>".to_string())
                    ),
                ),
            ])
        );
    }
}
