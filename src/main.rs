#![allow(non_snake_case)]
use clap::Parser;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::process::exit;
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
    statusCode: u16,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    /// Filter the status code
    filter: Option<String>,
}

fn colorize_status_code(status_code: u16) -> String {
    let status_code_class = status_code / 100;

    let color: Box<dyn color::Color> = match status_code_class {
        2 => Box::new(color::Green),
        3 => Box::new(color::Blue),
        4 => Box::new(color::Yellow),
        5 => Box::new(color::Red),
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
    let args = Args::parse();

    let maybe_filter = args.filter.clone();
    // validate filter arg if provided
    if maybe_filter.is_some() {
        let filter = maybe_filter.unwrap();
        // filter should have length of 3. otherwise show an error in stderr and exit
        if filter.len() != 3 {
            eprintln!("Filter should have length of 3");
            exit(1);
        }
    }

    let stdin = io::stdin();
    let mut req_logs: HashMap<String, ReqLog> = HashMap::new();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let value: Result<Value, _> = serde_json::from_str(&line);

        if let Ok(value) = value {
            handle_json_line(&args, value, &mut req_logs, line);
        } else {
            println!("{}", line);
        }
    }
}

fn handle_json_line(
    args: &Args,
    value: Value,
    req_logs: &mut HashMap<String, ReqLog>,
    raw_line: String,
) {
    if let Some(req_id_value) = value.get("reqId") {
        let req_id = req_id_value.as_str().unwrap().to_string();

        if value.get("req").is_some() {
            let log: ReqLog = serde_json::from_value(value).unwrap();
            req_logs.insert(req_id.to_string(), log);
        } else if value.get("res").is_some() {
            req_logs.get(&req_id);
            if let Some(req_log) = req_logs.get(&req_id) {
                handle_res_log(req_log, value, &args.filter);
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

fn handle_res_log(req_log: &ReqLog, value: Value, filter: &Option<String>) {
    let log: ResLog = serde_json::from_value(value).unwrap();
    let response_time = match log.responseTime {
        Some(time) => format!("{:.3}ms", time),
        None => "N/A".to_string(),
    };
    let status_code = log.res.statusCode;

    if filter.is_some() {
        if !filter_status_code(status_code, filter.as_ref().unwrap().as_str()) {
            return;
        }
    }

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
        colorize_status_code(status_code),
        req_log.req.url,
        req_log.req.method,
        response_time
    );
}

fn filter_status_code(code: u16, filter: &str) -> bool {
    match filter {
        "xxx" => true, // Match any status code
        _ => {
            let filter_chars: Vec<char> = filter.chars().collect();
            let code_chars: Vec<char> = code.to_string().chars().collect();

            for i in 0..3 {
                if filter_chars[i] != 'x' && filter_chars[i] != code_chars[i] {
                    return false;
                }
            }
            true
        }
    }
}
