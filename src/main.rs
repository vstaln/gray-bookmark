//! gray-bookmark — named markers you can `gray resume` around.
//!
//! Port of pi's `bookmark` extension. Pi labeled session entries for /tree
//! navigation; gray sidecars have no session-entry or fork/jump wire API, so
//! this stores named markers — `{label, session, cwd, ts}` JSON lines in
//! `~/.gray/bookmarks.jsonl` — that record where a session was when you
//! marked it.
//!
//! `/bookmark <label>` marks here, `/bookmark` or `/bookmarks` lists,
//! `/bookmark rm <n>` deletes by index. A `bookmark` tool gives the model the
//! same three verbs.

use std::io::{BufRead, Write};
use std::path::PathBuf;

use serde_json::{Value, json};

fn manifest() -> Value {
    json!({
        "name": "bookmark",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "1.1",
        "tools": [{
            "name": "bookmark",
            "description": "Manage named session bookmarks. Actions: list, add (label?), remove (index)",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list", "add", "remove"],
                        "description": "list | add | remove"
                    },
                    "label": { "type": "string", "description": "Bookmark label (for add; auto-generated when omitted)" },
                    "index": { "type": "number", "description": "1-based bookmark number from list (for remove)" }
                },
                "required": ["action"]
            }
        }],
        "commands": ["/bookmark", "/bookmarks"],
    })
}

fn store_path() -> PathBuf {
    let home = std::env::var_os("GRAY_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".gray")))
        .unwrap_or_else(|| PathBuf::from("."));
    home.join("bookmarks.jsonl")
}

fn session_id(params: &Value) -> &str {
    params
        .pointer("/session/id")
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn session_cwd(params: &Value) -> String {
    params
        .pointer("/session/cwd")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| ".".into())
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn load_all() -> Vec<Value> {
    std::fs::read_to_string(store_path())
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .collect()
}

fn save_all(marks: &[Value]) -> std::io::Result<()> {
    let path = store_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut text = String::new();
    for m in marks {
        text.push_str(&m.to_string());
        text.push('\n');
    }
    let tmp = path.with_extension("jsonl.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)
}

fn add(label: Option<&str>, session: &str, cwd: &str) -> Result<String, String> {
    let ts = now_ts();
    let label = label
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| format!("bookmark-{ts}"));
    let mut marks = load_all();
    marks.push(json!({"label": label, "session": session, "cwd": cwd, "ts": ts}));
    save_all(&marks).map_err(|e| format!("Error: could not save bookmark: {e}"))?;
    Ok(format!("Bookmarked as: {label}"))
}

/// Unix seconds → `YYYY-MM-DD HH:MM UTC` (civil-from-days; no chrono dep).
fn fmt_ts(ts: i64) -> String {
    let days = ts.div_euclid(86400);
    let secs = ts.rem_euclid(86400);
    // Howard Hinnant's civil_from_days
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02} {:02}:{:02} UTC", y, m, d, secs / 3600, (secs % 3600) / 60)
}

fn render(marks: &[Value]) -> String {
    if marks.is_empty() {
        return "No bookmarks. /bookmark <label> marks where you are.".into();
    }
    marks
        .iter()
        .enumerate()
        .map(|(i, m)| {
            format!(
                "#{} {} — {} · {} · {}",
                i + 1,
                m.get("label").and_then(Value::as_str).unwrap_or("?"),
                m.get("cwd").and_then(Value::as_str).unwrap_or("?"),
                m.get("session").and_then(Value::as_str).unwrap_or("?"),
                m.get("ts").and_then(Value::as_i64).map(fmt_ts).unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn remove(index: Option<i64>) -> Result<String, String> {
    let Some(n) = index else {
        return Err("Error: index required for remove (a 1-based number from list)".into());
    };
    let mut marks = load_all();
    if n < 1 || n as usize > marks.len() {
        return Err(format!("Bookmark #{n} not found"));
    }
    let gone = marks.remove((n - 1) as usize);
    save_all(&marks).map_err(|e| format!("Error: could not save bookmarks: {e}"))?;
    Ok(format!(
        "Removed bookmark #{}: {}",
        n,
        gone.get("label").and_then(Value::as_str).unwrap_or("?")
    ))
}

/// The `bookmark` tool: the same three verbs the slash commands expose.
fn call_tool(name: &str, args: &Value, params: &Value) -> Result<String, String> {
    if name != "bookmark" {
        return Err(format!("unknown tool: {name}"));
    }
    match args.get("action").and_then(Value::as_str).unwrap_or("") {
        "list" => Ok(render(&load_all())),
        "add" => add(
            args.get("label").and_then(Value::as_str),
            session_id(params),
            &session_cwd(params),
        ),
        "remove" => remove(args.get("index").and_then(Value::as_i64)),
        other => Err(format!("Unknown action: {other}")),
    }
}

/// `/bookmark …` — `argv` excludes the command name.
fn run_command(cmd: &str, argv: &[&str], params: &Value) -> String {
    if cmd == "/bookmarks" {
        return render(&load_all());
    }
    match argv {
        [] => render(&load_all()),
        ["rm", n, ..] | ["remove", n, ..] => match n.parse::<i64>() {
            Ok(i) => remove(Some(i)).unwrap_or_else(|e| e),
            Err(_) => "usage: /bookmark rm <n>".into(),
        },
        ["rm"] | ["remove"] => "usage: /bookmark rm <n>".into(),
        rest => match add(Some(&rest.join(" ")), session_id(params), &session_cwd(params)) {
            Ok(text) => text,
            Err(e) => e,
        },
    }
}

/// One request → `Some(reply)`, or `None` for notifications. The bool asks
/// the loop to exit after writing the reply.
fn handle(req: &Value) -> (Option<Value>, bool) {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(Value::Null);
    let Some(id) = id else {
        return (None, method == "plugin/shutdown");
    };
    let result = match method {
        "plugin/manifest" => manifest(),
        "tool/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("args").cloned().unwrap_or(Value::Null);
            match call_tool(name, &args, &params) {
                Ok(text) => json!({ "content": text }),
                Err(text) => json!({ "content": text, "is_error": true }),
            }
        }
        "command/run" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let argv: Vec<&str> = params
                .get("argv")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            json!({ "text": run_command(name, &argv, &params) })
        }
        "plugin/shutdown" => return (Some(json!({ "id": id, "result": {} })), true),
        _ => {
            let error = json!({ "code": -32601, "message": "method not found" });
            return (Some(json!({ "id": id, "error": error })), false);
        }
    };
    (Some(json!({ "id": id, "result": result })), false)
}

fn main() -> std::io::Result<()> {
    if std::env::args().nth(1).as_deref() == Some("manifest") {
        println!("{}", manifest());
        return Ok(());
    }
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        let Ok(req) = serde_json::from_str::<Value>(&line) else { continue };
        let (reply, exit) = handle(&req);
        if let Some(reply) = reply {
            writeln!(stdout, "{reply}")?;
            stdout.flush()?;
        }
        if exit {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(method: &str, params: Value) -> Value {
        handle(&json!({ "id": 1, "method": method, "params": params })).0.unwrap()
    }

    /// Serialized: GRAY_HOME is process-global and tests share the process.
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    static CTR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn with_home(body: impl FnOnce(&std::path::Path)) {
        let _g = LOCK.lock().unwrap();
        let n = CTR.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("gray-bookmark-test-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        unsafe { std::env::set_var("GRAY_HOME", &dir) };
        body(&dir);
        unsafe { std::env::remove_var("GRAY_HOME") };
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn sess() -> Value {
        json!({"id": "sess-1", "cwd": "/tmp/proj"})
    }

    #[test]
    fn manifest_has_tool_and_both_commands() {
        let m = call("plugin/manifest", Value::Null)["result"].clone();
        assert_eq!(m["name"], "bookmark");
        assert_eq!(m["commands"], json!(["/bookmark", "/bookmarks"]));
        assert_eq!(m["tools"][0]["name"], "bookmark");
    }

    #[test]
    fn command_add_list_remove() {
        with_home(|_| {
            // /bookmark <label> appends.
            let r = call(
                "command/run",
                json!({"name": "/bookmark", "argv": ["mark", "one"], "session": sess()}),
            );
            assert_eq!(r["result"]["text"], "Bookmarked as: mark one");

            // Bare /bookmark and /bookmarks list with 1-based numbers.
            for name in ["/bookmark", "/bookmarks"] {
                let r = call("command/run", json!({"name": name, "argv": [], "session": sess()}));
                let text = r["result"]["text"].as_str().unwrap();
                assert!(text.contains("#1 mark one"), "got: {text}");
                assert!(text.contains("/tmp/proj") && text.contains("sess-1"));
            }

            // rm deletes by index.
            let r = call(
                "command/run",
                json!({"name": "/bookmark", "argv": ["rm", "1"], "session": sess()}),
            );
            assert_eq!(r["result"]["text"], "Removed bookmark #1: mark one");
            let r = call("command/run", json!({"name": "/bookmarks", "argv": [], "session": sess()}));
            assert!(r["result"]["text"].as_str().unwrap().contains("No bookmarks"));
        });
    }

    #[test]
    fn tool_verbs_and_jsonl_shape() {
        with_home(|home| {
            let p = |args| json!({"name": "bookmark", "args": args, "session": sess()});
            let r = call("tool/call", p(json!({"action": "add", "label": "here"})));
            assert_eq!(r["result"]["content"], "Bookmarked as: here");
            // Label optional → auto-generated.
            let r = call("tool/call", p(json!({"action": "add"})));
            assert!(r["result"]["content"].as_str().unwrap().starts_with("Bookmarked as: bookmark-"));

            // JSONL on disk has the promised fields.
            let lines: Vec<Value> = std::fs::read_to_string(home.join("bookmarks.jsonl"))
                .unwrap()
                .lines()
                .map(|l| serde_json::from_str(l).unwrap())
                .collect();
            assert_eq!(lines.len(), 2);
            assert_eq!(lines[0]["label"], "here");
            assert_eq!(lines[0]["session"], "sess-1");
            assert_eq!(lines[0]["cwd"], "/tmp/proj");
            assert!(lines[0]["ts"].as_i64().unwrap() > 0);

            let r = call("tool/call", p(json!({"action": "list"})));
            let text = r["result"]["content"].as_str().unwrap();
            assert!(text.contains("#1 here") && text.contains("#2 bookmark-"));

            assert!(call("tool/call", p(json!({"action": "remove"})))["result"]["is_error"].as_bool().unwrap());
            assert!(call("tool/call", p(json!({"action": "remove", "index": 9})))["result"]["is_error"].as_bool().unwrap());
            let r = call("tool/call", p(json!({"action": "remove", "index": 2})));
            assert!(r["result"]["content"].as_str().unwrap().starts_with("Removed bookmark #2"));
        });
    }

    #[test]
    fn fmt_ts_renders_utc() {
        assert_eq!(fmt_ts(0), "1970-01-01 00:00 UTC");
        assert_eq!(fmt_ts(1760000000), "2025-10-09 08:53 UTC");
    }

    #[test]
    fn shutdown_replies_then_exits() {
        let (reply, exit) = handle(&json!({ "id": 2, "method": "plugin/shutdown" }));
        assert!(reply.is_some() && exit);
    }
}
