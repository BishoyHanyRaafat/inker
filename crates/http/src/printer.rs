use axum::Router;
use regex::Regex;
use std::collections::HashMap;
use tracing::info;

pub fn print_routes(app: &Router) {
    let debug_str = format!("{:?}", app);

    let method_re =
        Regex::new(r#"RouteId\((\d+)\): MethodRouter\(MethodRouter \{ ([^}]+) \}\)"#).unwrap();

    let path_re = Regex::new(r#"RouteId\((\d+)\): "([^"]+)""#).unwrap();

    let mut route_methods: HashMap<String, Vec<String>> = HashMap::new();
    for cap in method_re.captures_iter(&debug_str) {
        let id = &cap[1];
        let methods_str = &cap[2];
        let methods = methods_str
            .split(',')
            .filter_map(|s| {
                let s = s.trim();
                if s.ends_with(": None") {
                    None
                } else if s.ends_with(": Route") || s.ends_with(": BoxedHandler") {
                    Some(s.split(':').next().unwrap().to_uppercase())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        route_methods.insert(id.to_string(), methods);
    }

    // Determine max path length for alignment
    let max_path_len = path_re
        .captures_iter(&debug_str)
        .map(|cap| cap[2].len())
        .max()
        .unwrap_or(0);

    // Print routes
    for cap in path_re.captures_iter(&debug_str) {
        let route_id = &cap[1];
        let path = &cap[2];

        if path.contains("/{*__private__axum_fallback}") {
            continue;
        }

        let padded_path = format!("{:<width$}", path, width = max_path_len);

        // Build the methods string
        let methods_display = if let Some(methods) = route_methods.get(route_id) {
            if methods.is_empty() {
                "FALLBACK".to_string()
            } else {
                methods.join(", ")
            }
        } else {
            "UNKNOWN".to_string()
        };

        info!(target: "http::Route", "{} [{}]", padded_path, methods_display);
    }
}
