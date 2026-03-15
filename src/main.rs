use axum::{http::StatusCode, routing::get, Json, Router};
use tower_http::trace::TraceLayer;
use chrono::{Duration, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct Task {
    text: String,
    done: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let todo_url  = std::env::var("TODO_URL").unwrap_or_else(|_| "http://localhost:8765".into());
    let txtme_url = std::env::var("TXTME_URL").unwrap_or_else(|_| "http://localhost:5543".into());
    let txtme_key = std::env::var("TXTME_API_KEY").unwrap_or_default();
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(5544);

    // PORT: BRIEF_HOUR — hour of day to send the brief (0–23, default: 7)
    let brief_hour: u32 = std::env::var("BRIEF_HOUR")
        .ok()
        .and_then(|h| h.parse().ok())
        .unwrap_or(7);

    // PORT: BRIEF_TZ — timezone for the brief schedule (default: America/New_York)
    let brief_tz: Tz = std::env::var("BRIEF_TZ")
        .ok()
        .and_then(|tz| tz.parse().ok())
        .unwrap_or(chrono_tz::America::New_York);

    tokio::spawn(scheduler_loop(todo_url, txtme_url, txtme_key, brief_hour, brief_tz));

    let app = Router::new()
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http());

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await.unwrap();
    println!("[daily-brief] listening on :{port}");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

async fn scheduler_loop(
    todo_url: String,
    txtme_url: String,
    txtme_key: String,
    brief_hour: u32,
    brief_tz: Tz,
) {
    loop {
        let wait = secs_until_brief(brief_hour, brief_tz);
        println!("[daily-brief] next brief in {}m", wait.as_secs() / 60);
        tokio::time::sleep(wait).await;

        match send_brief(&todo_url, &txtme_url, &txtme_key).await {
            Ok(())  => println!("[daily-brief] sent"),
            Err(e) => eprintln!("[daily-brief] error: {e}"),
        }

        // Guard against firing twice in the same minute
        tokio::time::sleep(std::time::Duration::from_secs(90)).await;
    }
}

fn secs_until_brief(hour: u32, tz: Tz) -> std::time::Duration {
    let now_local = Utc::now().with_timezone(&tz);
    let target_time = NaiveTime::from_hms_opt(hour, 0, 0).unwrap();

    let target_date = if now_local.time() < target_time {
        now_local.date_naive()
    } else {
        now_local.date_naive() + Duration::days(1)
    };

    let target = tz.from_local_datetime(&target_date.and_time(target_time)).unwrap();
    let secs   = (target.with_timezone(&Utc) - Utc::now()).num_seconds().max(0) as u64;
    std::time::Duration::from_secs(secs)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max).collect::<String>())
    }
}

// PORT: MESSAGE_FORMAT — customize the text message content
async fn send_brief(
    todo_url:  &str,
    txtme_url: &str,
    txtme_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let tasks: Vec<Task> = client
        .get(format!("{todo_url}/tasks"))
        .send()
        .await?
        .json()
        .await?;

    let top = tasks.iter().find(|t| !t.done);

    let message = match top {
        None    => "GM! All clear today.".to_string(),
        Some(t) => format!("GM!\nNext up: {}", truncate(&t.text, 60)),
    };

    client
        .post(format!("{txtme_url}/notify"))
        .header("X-Api-Key", txtme_key)
        .json(&json!({"message": message}))
        .send()
        .await?;

    Ok(())
}
