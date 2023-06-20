#![allow(non_snake_case)]
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, BufRead};
use termion::color;
#[derive(Deserialize)]
struct ReqLog {
    req: Req,
}

#[derive(Deserialize)]
struct ResLog {
    res: Res,
    responseTime: Option<f64>,
    err: Option<Error>,
}

#[derive(Deserialize)]
struct Error {
    message: String,
    stack: String,
}

#[derive(Deserialize)]
struct Req {
    method: String,
    url: String,
}

#[derive(Deserialize)]
struct Res {
    statusCode: u64,
}

fn colorize_status_code(status_code: u64) -> String {
    let color: Box<dyn color::Color> = match status_code {
        200 => Box::new(color::Green),
        300 => Box::new(color::Blue),
        400 => Box::new(color::Yellow),
        500 => Box::new(color::Red),
        _ => Box::new(color::White),
    };

    format!(
        "{}{}{}",
        color::Fg(&*color),
        status_code,
        color::Fg(color::Reset)
    )
}

fn main() {
    let stdin = io::stdin();
    let mut req_logs: HashMap<String, ReqLog> = HashMap::new();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let value: Result<Value, _> = serde_json::from_str(&line);

        if let Ok(value) = value {
            handle_json_line(value, &mut req_logs, line);
        } else {
            println!("{}", line);
        }
    }
}

fn handle_json_line(value: Value, req_logs: &mut HashMap<String, ReqLog>, raw_line: String) {
    if let Some(req_id_value) = value.get("reqId") {
        let req_id = req_id_value.as_str().unwrap().to_string();

        if value.get("req").is_some() {
            let log: ReqLog = serde_json::from_value(value).unwrap();
            req_logs.insert(req_id.to_string(), log);
        } else if value.get("res").is_some() {
            req_logs.get(&req_id);
            if let Some(req_log) = req_logs.get(&req_id) {
                handle_res_log(req_log, value);
            }
            req_logs.remove(&req_id);
        }
    } else {
        let message_to_log = match value.get("msg") {
            Some(msg) => msg.as_str().unwrap().to_string(),
            None => raw_line,
        };

        println!("{}", message_to_log);
    }
}

fn handle_res_log(req_log: &ReqLog, value: Value) {
    let log: ResLog = serde_json::from_value(value).unwrap();
    let response_time = match log.responseTime {
        Some(time) => format!("{:.3}ms", time),
        None => "N/A".to_string(),
    };
    let status_code = log.res.statusCode;

    if let Some(err) = &log.err {
        println!(
            "{}\n{}",
            format!(
                "{}{}{}",
                color::Fg(color::Red),
                err.message,
                color::Fg(color::Reset)
            ),
            err.stack
        );
    }

    println!(
        "{} {} {} {}",
        req_log.req.url,
        req_log.req.method,
        colorize_status_code(status_code),
        response_time
    );
}
