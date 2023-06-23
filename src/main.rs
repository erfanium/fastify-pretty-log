#![allow(non_snake_case)]
use clap::Parser;
use serde::Deserialize;
use serde_json::from_str;
use serde_json::from_value;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::process::exit;
use termion::color;
#[derive(Deserialize)]
struct Log {
    msg: String,
    reqId: Option<String>,
    req: Option<Req>,
    res: Option<Res>,
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

    #[arg(short, long)]
    /// no-error
    no_errors: bool,
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
    let mut req_logs: HashMap<String, Log> = HashMap::new();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let value: Result<Log, _> = from_str(&line);

        if let Ok(log) = value {
            handle_json_log(&args, log, &mut req_logs, line);
        } else {
            println!("{}", line);
        }
    }
}

fn handle_json_log(args: &Args, log: Log, req_logs: &mut HashMap<String, Log>, raw_line: String) {
    let msg: &str = log.msg.as_ref();

    match msg {
        "incoming request" => {
            let req_id = log.reqId.clone().unwrap();
            req_logs.insert(req_id, log);
        }
        "request completed" => {
            let req_id = log.reqId.as_ref().unwrap();
            let maybe_request_log = req_logs.remove(req_id);
            match maybe_request_log {
                Some(request_log) => {
                    handle_res_log(&log, &request_log, &args.filter);
                }
                None => println!("{}", raw_line),
            }
        }
        _ => match &log.err {
            Some(err) => {
                if args.no_errors {
                    return;
                }
                handle_error_log(&log);
            }
            None => println!("{}", raw_line),
        },
    }
}

fn handle_res_log(response_log: &Log, request_log: &Log, filter: &Option<String>) {
    let response_time = match response_log.responseTime {
        Some(time) => format!("{:.3}ms", time),
        None => "N/A".to_string(),
    };
    let status_code = response_log.res.as_ref().unwrap().statusCode;

    if filter.is_some() {
        if !filter_status_code(status_code, filter.as_ref().unwrap().as_str()) {
            return;
        }
    }

    println!(
        "{} {} {} {}",
        colorize_status_code(status_code),
        request_log.req.as_ref().unwrap().method,
        request_log.req.as_ref().unwrap().url,
        response_time
    );
}

fn handle_error_log(log: &Log) {
    println!(
        "{}\n{}",
        format!(
            "{}{}{}",
            color::Fg(color::Red),
            log.err.as_ref().unwrap().message,
            color::Fg(color::Reset)
        ),
        log.err.as_ref().unwrap().stack
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
