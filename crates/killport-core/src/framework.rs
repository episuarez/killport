//! Best-effort framework detection: command line first (authoritative), then a
//! port-default hint as fallback. Returns a human label, never used for kill logic.

pub fn detect(cmd: &[String], port: u16) -> Option<String> {
    let toks = cmd.join(" ").to_lowercase();
    let has = |n: &str| toks.contains(n);

    let label = if has("vite") {
        "Vite"
    } else if has("next") {
        "Next.js"
    } else if has("nuxt") {
        "Nuxt"
    } else if has("astro") {
        "Astro"
    } else if has("remix") {
        "Remix"
    } else if has("gatsby") {
        "Gatsby"
    } else if has("sveltekit") || has("svelte-kit") {
        "SvelteKit"
    } else if has("react-scripts") {
        "Create React App"
    } else if has("angular") || has("@angular") {
        "Angular"
    } else if has("webpack") {
        "Webpack"
    } else if has("vue-cli-service") {
        "Vue CLI"
    } else if has("uvicorn") || has("fastapi") {
        "FastAPI/Uvicorn"
    } else if has("flask") {
        "Flask"
    } else if has("django") || has("manage.py") {
        "Django"
    } else if has("gunicorn") {
        "Gunicorn"
    } else if has("artisan") {
        "Laravel"
    } else if has("rails") || has("puma") {
        "Rails"
    } else if has("express") {
        "Express"
    } else {
        return port_hint(port).map(str::to_string);
    };
    Some(label.to_string())
}

fn port_hint(port: u16) -> Option<&'static str> {
    match port {
        3000 => Some("Node / Next.js / CRA"),
        3001 => Some("Node (alt)"),
        4200 => Some("Angular"),
        5000 => Some("Flask / .NET"),
        5173 => Some("Vite"),
        5432 => Some("PostgreSQL"),
        6379 => Some("Redis"),
        8000 => Some("Django / Python"),
        8080 => Some("HTTP / Webpack"),
        8888 => Some("Jupyter"),
        9000 => Some("PHP / SonarQube"),
        27017 => Some("MongoDB"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::detect;

    fn v(p: &[&str]) -> Vec<String> {
        p.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn cmd_beats_port() {
        assert_eq!(
            detect(&v(&["node", ".bin/vite"]), 3000).as_deref(),
            Some("Vite")
        );
    }

    #[test]
    fn port_hint_fallback() {
        assert_eq!(detect(&v(&["node", "x.js"]), 5173).as_deref(), Some("Vite"));
    }

    #[test]
    fn none_when_unknown() {
        assert_eq!(detect(&v(&["foo"]), 12345), None);
    }
}
