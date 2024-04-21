use clap::Parser;
use serde::Deserialize;
use serde_json::from_str;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::process::exit;
use termion::color;
#[derive(Deserialize)]
struct BaseLog {
    msg: String,
    // level: u32,
    // time: u64,
    name: Option<String>,
}

// define request log that extends log
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WithRequestInfo {
    req_id: String,
    req: Request,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WithResponseInfo {
    req_id: String,
    res: Res,
    response_time: f64,
}

#[derive(Deserialize)]
struct WithError {
    err: Error,
}

#[derive(Deserialize)]
struct Error {
    message: String,
    stack: String,
}

#[derive(Deserialize, Clone)]
struct Request {
    method: String,
    url: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Res {
    status_code: u16,
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
    let mut req_logs: HashMap<String, WithRequestInfo> = HashMap::new();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let value: Result<BaseLog, _> = from_str(&line);

        if let Ok(log) = value {
            handle_json_log(&args, log, &mut req_logs, line);
        } else {
            println!("{}", line);
        }
    }
}

fn handle_json_log(
    args: &Args,
    log: BaseLog,
    req_logs: &mut HashMap<String, WithRequestInfo>,
    raw_line: String,
) {
    let message: &str = &log.msg;
    match message {
        "incoming request" => {
            let request_log: WithRequestInfo =
                from_str(&raw_line).expect("Error parsing request log");
            req_logs.insert(request_log.req_id.clone(), request_log);
        }
        "request completed" => {
            let response_log: WithResponseInfo =
                from_str(&raw_line).expect("Error parsing response log");
            let maybe_request_log = req_logs.remove(&response_log.req_id);
            match maybe_request_log {
                Some(request_log) => {
                    handle_res_log(&response_log, &request_log, &args.filter);
                }
                None => println!("{}", raw_line),
            }
        }
        _ => {
            // check if raw_line
            let maybe_with_err: Result<WithError, _> = from_str(&raw_line);
            match maybe_with_err {
                Ok(with_err) => {
                    handle_error_log(log, &with_err);
                }
                Err(_) => {
                    println!("[{}] {}", log.name.unwrap_or('-'.to_string()), message);
                }
            }
        }
    }
}

fn handle_res_log(
    response_log: &WithResponseInfo,
    request_log: &WithRequestInfo,
    filter: &Option<String>,
) {
    let time = format!("{:.3}ms", response_log.response_time);
    let status_code = response_log.res.status_code;

    if filter.is_some() {
        if !filter_status_code(status_code, filter.as_ref().unwrap().as_str()) {
            return;
        }
    }

    println!(
        "{} {} {} {}",
        colorize_status_code(status_code),
        request_log.req.method,
        request_log.req.url,
        time
    );
}

fn handle_error_log(log: BaseLog, error_detail: &WithError) {
    println!(
        "{}\n{}",
        match log.name {
            Some(name) => format!(
                "{}[{}] {}{}",
                color::Fg(color::Red),
                name,
                error_detail.err.message,
                color::Fg(color::Reset)
            ),
            None => format!(
                "{}{}{}",
                color::Fg(color::Red),
                error_detail.err.message,
                color::Fg(color::Reset)
            ),
        },
        error_detail.err.stack
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
