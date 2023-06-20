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
    if status_code >= 500 {
        format!(
            "{}{}{}",
            color::Fg(color::Red),
            status_code,
            color::Fg(color::Reset)
        )
    } else if status_code >= 400 {
        format!(
            "{}{}{}",
            color::Fg(color::Yellow),
            status_code,
            color::Fg(color::Reset)
        )
    } else if status_code >= 300 {
        format!(
            "{}{}{}",
            color::Fg(color::Blue),
            status_code,
            color::Fg(color::Reset)
        )
    } else if status_code >= 200 {
        format!(
            "{}{}{}",
            color::Fg(color::Green),
            status_code,
            color::Fg(color::Reset)
        )
    } else {
        status_code.to_string()
    }
}

fn main() {
    let stdin = io::stdin();
    let mut req_logs: HashMap<String, ReqLog> = HashMap::new();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let value: Result<Value, _> = serde_json::from_str(&line);

        if let Ok(value) = value {
            if let Some(req_id_value) = value.get("reqId") {
                let req_id = req_id_value.as_str().unwrap().to_string();

                if value.get("req").is_some() {
                    handle_req_log(value, &req_id, &mut req_logs);
                } else if value.get("res").is_some() {
                    handle_res_log(value, &req_id, &mut req_logs);
                }
            } else {
                let message_to_log = match value.get("msg") {
                    Some(msg) => msg.as_str().unwrap().to_string(),
                    None => line,
                };

                println!("{}", message_to_log);
            }
        } else {
            println!("{}", line);
        }
    }
}

fn handle_req_log(value: Value, req_id: &str, req_logs: &mut HashMap<String, ReqLog>) {
    let log: ReqLog = serde_json::from_value(value).unwrap();
    req_logs.insert(req_id.to_string(), log);
}

fn handle_res_log(value: Value, req_id: &str, req_logs: &mut HashMap<String, ReqLog>) {
    let log: ResLog = serde_json::from_value(value).unwrap();
    if let Some(req_log) = req_logs.get(req_id) {
        let response_time = match log.responseTime {
            Some(time) => format!("{:.3}ms", time),
            None => "N/A".to_string(),
        };
        let status_code = log.res.statusCode;

        println!(
            "{} {} {} {}",
            req_log.req.url,
            req_log.req.method,
            colorize_status_code(status_code),
            response_time
        );

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

        req_logs.remove(req_id);
    }
}
