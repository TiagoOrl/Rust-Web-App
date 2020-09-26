use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::fs;


use handlebars::Handlebars;
use serde_json::json;
use rand::Rng;

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

pub struct HTTPHandler {
    get_home: String,
    get_ajax_msg: String,
    get_ajax_update: String,
    get_analytics: String,
    get_sleep: String,
    server_ok: String,
    server_404: String
}

pub struct AnalyticsManager {
    msg_count: u32,
    view_count: u32,
    share_count: u32,
    usr_count: u32
}


impl AnalyticsManager {

    pub fn new() -> AnalyticsManager {
        let mut rng = rand::thread_rng();
        AnalyticsManager {
        msg_count: rng.gen_range(0,100) as u32,
        view_count: rng.gen_range(0,100) as u32,
        share_count: rng.gen_range(0,100) as u32,
        usr_count: rng.gen_range(0,100) as u32,
        }
        
    }

    pub fn update_dashboard(&mut self) -> String {
        let mut rng = rand::thread_rng();

        self.msg_count += rng.gen_range(0,100) as u32;
        self.view_count += rng.gen_range(0,100) as u32;
        self.share_count += rng.gen_range(0,100) as u32;
        self.usr_count += rng.gen_range(0,100) as u32;

        json!({
            "msg_count" : self.msg_count,
            "view_count" : self.view_count,
            "share_count" : self.share_count,
            "usr_count" : self.usr_count,
        }).to_string()
    }
}

impl HTTPHandler {

    pub fn new() -> HTTPHandler {
        
        HTTPHandler {
            get_home: String::from("GET / HTTP/1.1\r\n"),
            get_ajax_msg: String::from("GET /ajax_get_msg HTTP/1.1\r\n"),
            get_ajax_update: String::from("GET /ajax_update HTTP/1.1\r\n"),
            get_analytics: String::from("GET /analytics HTTP/1.1\r\n"),
            get_sleep: String::from("GET /sleep HTTP/1.1\r\n"),
            server_ok: String::from("HTTP/1.1 200 OK\r\n\r\n"),
            server_404: String::from("HTTP/1.1 404 NOT FOUND\r\n\r\n")
        }
    }

    pub fn handle_page_request(&self, buffer: &[u8; 1024]) -> String {

        let mut filename = String::new();
        let mut status_line = String::new();
        let mut contents: String = String::from("");
        let handlebars = Handlebars::new();
        let mut is_ajax_request = false;
        let mut json_data = json!({});
        

        if buffer.starts_with(self.get_home.as_bytes())  {
            status_line = self.server_ok.clone();
            filename = "hello.html".to_string();
            json_data = json!({"name" : "User"});
        }

        else if buffer.starts_with(self.get_analytics.as_bytes()) {
            status_line = self.server_ok.clone();
            filename = "analytics.html".to_string();

            json_data = json!({
                "msg_count": "546",
                "view_count": "12874",
                "share_count": "647",
                "usr_count": "10938",
            });
        }

        else if buffer.starts_with(self.get_ajax_msg.as_bytes()) {
            status_line = self.server_ok.clone();
            is_ajax_request = true;
            contents = String::from("oifdsjifomds oifunsoi nfdoiu nofdsunfidnosfiundoiu fhfiasd \n fudsnfudsoiu");
        }

        else if buffer.starts_with(self.get_ajax_update.as_bytes()) {
            status_line = self.server_ok.clone();
            is_ajax_request = true;

            contents = AnalyticsManager::update_dashboard(&mut AnalyticsManager::new());
            
        }

        else {
            status_line = self.server_404.clone();
            filename = "404.html".to_string();
        }

        if !is_ajax_request{
            contents = handlebars.render_template(&fs::read_to_string(&filename).unwrap(), &json_data).unwrap();
        }
        
        let http_response = format!("{}{}", status_line, contents);
        
        http_response
    }
}

impl ThreadPool {
    pub fn new(n: usize) -> ThreadPool {
        assert!(n > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(n);

        for id in 0..n {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {workers, sender}
    }

    pub fn execute<J> (&self, f: J)
    where 
        J: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        
        let thread = thread::spawn( move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker got a job {}, executing.", id);
            job();
        });

        Worker {id, thread}
    }
}