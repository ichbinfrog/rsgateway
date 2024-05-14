use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{spawn, JoinHandle},
};

use tokio::net::TcpListener;

use crate::{
    dns::resolver::Google,
    http::{client::Client, request::Request},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing.");

            job();
        });

        Worker { id, thread }
    }
}
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

pub struct Proxy {
    listener: TcpListener,
}

impl Proxy {
    pub async fn new(addr: &str) -> Self {
        Self {
            listener: TcpListener::bind(addr).await.unwrap(),
        }
    }

    pub async fn run(&self) {
        loop {
            let (mut stream, _) = self.listener.accept().await.unwrap();
            tokio::spawn(async move {
                let req = Request::parse(&mut stream).await.unwrap();

                let mut url = "http://httpbin.org".to_string();
                url.push_str(&String::try_from(req.parts.url.path).unwrap());
                let mut headers = req.parts.headers.clone();
                headers
                    .raw
                    .insert("host".to_string(), "httpbin.org".to_string());
                let resp = Client::get::<Google>(url, headers).await.unwrap();
                println!("{:?}", resp);
            });
        }
    }
}
