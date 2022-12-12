mod request;
mod response;

use clap::Parser;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{sleep, Duration, Instant};
// use std::time::Duration;
// use delay_timer::prelude::{Task, TaskBuilder, TaskError};

/// Contains information parsed from the command-line invocation of balancebeam. The Clap macros
/// provide a fancy way to automatically construct a command-line argument parser.
#[derive(Parser, Debug)]
#[command(about = "Fun with load balancing")]
struct CmdOptions {
    /// IP/port to bind to
    #[arg(short, long, default_value = "0.0.0.0:1100")]
    bind: String,

    /// Upstream host to forward requests to
    #[arg(short, long)]
    upstream: Vec<String>,

    /// Perform active health checks on this interval (in seconds)
    #[arg(long, default_value = "10")]
    active_health_check_interval: usize,

    /// Path to send request to for active health checks
    #[arg(long, default_value = "/")]
    active_health_check_path: String,

    /// Maximum number of requests to accept per IP per minute (0 = unlimited)
    #[arg(long, default_value = "0")]
    max_requests_per_minute: usize,
}

/// Contains information about the state of balancebeam (e.g. what servers we are currently proxying
/// to, what servers have failed, rate limiting counts, etc.)
///
/// You should add fields to this struct in later milestones.
struct ProxyState {
    /// How frequently we check whether upstream servers are alive (Milestone 4)
    active_health_check_interval: usize,

    /// Where we should send requests when doing active health checks (Milestone 4)
    active_health_check_path: String,

    /// Maximum number of requests an individual IP can make in a minute (Milestone 5)
    max_requests_per_minute: usize,

    // DONE: 改用Arc存String，减少clone
    /// Addresses of servers that we are proxying to
    upstream_addresses: RwLock<Vec<Arc<String>>>,

    failed_upstream_addresses: RwLock<Vec<Arc<String>>>,

    slide_windows: Mutex<HashMap<String, SlideWindow>>,
}

struct SlideWindow {
    capacity: usize,
    time_unit: u64,
    cur_time: Instant,
    pre_count: usize,
    cur_count: usize,
}

#[tokio::main]
async fn main() {
    // Initialize the logging library. You can print log messages using the `log` macros:
    // https://docs.rs/log/0.4.8/log/ You are welcome to continue using print! statements; this
    // just looks a little prettier.
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "debug");
    }
    pretty_env_logger::init();

    // Parse the command line arguments passed to this program
    let options = CmdOptions::parse();
    if options.upstream.len() < 1 {
        log::error!("At least one upstream server must be specified using the --upstream option.");
        std::process::exit(1);
    }

    // Start listening for connections
    let listener = match TcpListener::bind(&options.bind).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!("Could not bind to {}: {}", options.bind, err);
            std::process::exit(1);
        }
    };
    log::info!("Listening for requests on {}", options.bind);

    // Handle incoming connections
    let mut streams = Vec::new();
    for stream in options.upstream {
        streams.push(Arc::new(stream));
    }
    let state = Arc::new(ProxyState {
        upstream_addresses: RwLock::new(streams),
        active_health_check_interval: options.active_health_check_interval,
        active_health_check_path: options.active_health_check_path,
        max_requests_per_minute: options.max_requests_per_minute,
        failed_upstream_addresses: RwLock::new(Vec::new()),
        slide_windows: Mutex::new(HashMap::new()),
    });

    let state_clone = state.clone();
    tokio::spawn(async move {
        build_task_active_health_check(&state_clone).await;
    });

    loop {
        if let Ok((socket, _)) = listener.accept().await {
            let state = state.clone();
            tokio::spawn(async move {
                handle_connection(socket, &state).await;
            });
        }
    }
}

pub async fn upstream_active_health_check(path: &str, upstream: &str) -> bool {
    let request = http::Request::builder()
        .method(http::Method::GET)
        .uri(path)
        .header("Host", upstream)
        .body(Vec::new())
        .unwrap();
    match TcpStream::connect(upstream).await {
        Ok(mut stream) => {
            if let Err(error) = request::write_to_stream(&request, &mut stream).await {
                log::error!("Failed to send request to upstream {}: {}", upstream, error);
                return false;
            }
            let res_status = match response::read_from_stream(&mut stream, request.method()).await {
                Ok(response) => response.status().as_u16(),
                Err(error) => {
                    log::error!("Error reading response from server: {:?}", error);
                    return false;
                }
            };
            return res_status == 200;
        }
        Err(err) => {
            log::error!("Failed to connect to upstream {}: {}", upstream, err);
            return false;
        }
    }
}

async fn filter_upstream_addresses(
    upstream_addresses: &RwLock<Vec<Arc<String>>>,
    path: &str,
    active_flag: bool,
) -> Vec<Arc<String>> {
    let upstream_addresses_rd = upstream_addresses.read().await;
    let mut ret = Vec::new();
    let mut remain = Vec::new();
    for upstream in upstream_addresses_rd.iter() {
        if upstream_active_health_check(path, upstream).await == active_flag {
            remain.push(upstream.clone());
        } else {
            ret.push(upstream.clone());
        }
    }
    drop(upstream_addresses_rd);

    let mut upstream_addresses_wr = upstream_addresses.write().await;
    upstream_addresses_wr.clear();
    upstream_addresses_wr.append(&mut remain);

    ret
}

async fn active_health_check(state: &ProxyState) {
    let mut reactived_upstreams = filter_upstream_addresses(
        &state.failed_upstream_addresses,
        &state.active_health_check_path,
        false,
    )
    .await;

    let mut refailed_upstreams = filter_upstream_addresses(
        &state.upstream_addresses,
        &state.active_health_check_path,
        true,
    )
    .await;

    let mut upstream_addresses_wr = state.upstream_addresses.write().await;
    upstream_addresses_wr.append(&mut reactived_upstreams);
    drop(upstream_addresses_wr);
    let mut failed_upstream_addresses_wr = state.failed_upstream_addresses.write().await;
    failed_upstream_addresses_wr.append(&mut refailed_upstreams);
}

async fn build_task_active_health_check(state: &ProxyState) {
    // let mut task_builder = TaskBuilder::default();
    // task_builder
    //     .set_frequency_repeated_by_seconds(6)
    //     .set_maximum_parallel_runnable_num(2)
    //     .spawn_async_routine(|| async {
    //         active_health_check(state).await;
    //     })
    loop {
        sleep(Duration::from_secs(
            state.active_health_check_interval as u64,
        ))
        .await;
        active_health_check(state).await;
    }
}

async fn connect_to_upstream(state: &ProxyState) -> Result<TcpStream, std::io::Error> {
    loop {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let upstream_addresses_rd = state.upstream_addresses.read().await;
        let upstream_idx = rng.gen_range(0..upstream_addresses_rd.len());
        let upstream_ip = &*upstream_addresses_rd[upstream_idx].clone(); // clone并drop，加速
        drop(upstream_addresses_rd);
        match TcpStream::connect(upstream_ip).await {
            Ok(some) => return Ok(some),
            Err(err) => {
                log::error!("Failed to connect to upstream {}: {}", upstream_ip, err);
                let mut upstream_addresses_wr = state.upstream_addresses.write().await;
                let mut failed_upstream_addresses_wr =
                    state.failed_upstream_addresses.write().await;
                failed_upstream_addresses_wr.push(upstream_addresses_wr.swap_remove(upstream_idx));
                if upstream_addresses_wr.is_empty() {
                    return Err(Error::new(ErrorKind::Other, "No alive upstream!"));
                }
            }
        }
    }
    // DONE: implement failover (milestone 3)
}

async fn send_response(client_conn: &mut TcpStream, response: &http::Response<Vec<u8>>) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!(
        "{} <- {}",
        client_ip,
        response::format_response_line(&response)
    );
    if let Err(error) = response::write_to_stream(&response, client_conn).await {
        log::warn!("Failed to send response to client: {}", error);
        return;
    }
}

impl SlideWindow {
    pub fn new(
        capacity: usize,
        time_unit: u64,
        cur_time: Instant,
        pre_count: usize,
        cur_count: usize,
    ) -> Self {
        SlideWindow {
            capacity,
            time_unit,
            cur_time,
            pre_count,
            cur_count,
        }
    }
    pub fn should_rate_limiting(&mut self) -> bool {
        if self.capacity == 0 {
            return false;
        }
        if self.cur_time.elapsed().as_secs() >= self.time_unit {
            self.cur_time = Instant::now();
            self.pre_count = self.cur_count;
            self.cur_count = 0;
        }
        let estimated_count = self.pre_count as f64
            * (1.0 - self.cur_time.elapsed().as_secs() as f64 / self.time_unit as f64)
            + self.cur_count as f64;
        if estimated_count >= self.capacity as f64 {
            // should limit
            return true;
        }
        self.cur_count += 1;
        false
    }
}

async fn handle_connection(mut client_conn: TcpStream, state: &ProxyState) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!("Connection received from {}", client_ip);

    // Open a connection to a random destination server
    let mut upstream_conn = match connect_to_upstream(state).await {
        Ok(stream) => stream,
        Err(_error) => {
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
    };
    let upstream_ip = upstream_conn.peer_addr().unwrap().ip().to_string();

    // The client may now send us one or more requests. Keep trying to read requests until the
    // client hangs up or we get an error.
    loop {
        // Read a request from the client
        let mut request = match request::read_from_stream(&mut client_conn).await {
            Ok(request) => request,
            // Handle case where client closed connection and is no longer sending requests
            Err(request::Error::IncompleteRequest(0)) => {
                log::debug!("Client finished sending requests. Shutting down connection");
                return;
            }
            // Handle I/O error in reading from the client
            Err(request::Error::ConnectionError(io_err)) => {
                log::info!("Error reading request from client stream: {}", io_err);
                return;
            }
            Err(error) => {
                log::debug!("Error parsing request: {:?}", error);
                let response = response::make_http_error(match error {
                    request::Error::IncompleteRequest(_)
                    | request::Error::MalformedRequest(_)
                    | request::Error::InvalidContentLength
                    | request::Error::ContentLengthMismatch => http::StatusCode::BAD_REQUEST,
                    request::Error::RequestBodyTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
                    request::Error::ConnectionError(_) => http::StatusCode::SERVICE_UNAVAILABLE,
                });
                send_response(&mut client_conn, &response).await;
                continue;
            }
        };
        log::info!(
            "{} -> {}: {}",
            client_ip,
            upstream_ip,
            request::format_request_line(&request)
        );

        // DONE: rate limiting here
        if state.max_requests_per_minute > 0 {
            let mut slide_windows = state.slide_windows.lock().await;
            let slide_window = slide_windows
                .entry(client_ip.clone())
                .or_insert(SlideWindow::new(
                    state.max_requests_per_minute,
                    60,
                    Instant::now(),
                    0,
                    0,
                ));
            if slide_window.should_rate_limiting() {
                let response = response::make_http_error(http::StatusCode::TOO_MANY_REQUESTS);
                send_response(&mut client_conn, &response).await;
                continue;
            }
        }

        // Add X-Forwarded-For header so that the upstream server knows the client's IP address.
        // (We're the ones connecting directly to the upstream server, so without this header, the
        // upstream server will only know our IP, not the client's.)
        request::extend_header_value(&mut request, "x-forwarded-for", &client_ip);

        // Forward the request to the server
        if let Err(error) = request::write_to_stream(&request, &mut upstream_conn).await {
            log::error!(
                "Failed to send request to upstream {}: {}",
                upstream_ip,
                error
            );
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
        log::debug!("Forwarded request to server");

        // Read the server's response
        let response = match response::read_from_stream(&mut upstream_conn, request.method()).await
        {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error reading response from server: {:?}", error);
                let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
                send_response(&mut client_conn, &response).await;
                return;
            }
        };
        // Forward the response to the client
        send_response(&mut client_conn, &response).await;
        log::debug!("Forwarded response to client");
    }
}
