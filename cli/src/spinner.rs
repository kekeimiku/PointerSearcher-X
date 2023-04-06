use std::{
    io,
    io::Write,
    ops::Add,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread, time,
};

pub struct Spinner {
    thread_handle: Option<thread::JoinHandle<()>>,
    still_spinning: Arc<AtomicBool>,
}

impl Spinner {
    pub fn start(msg: &'static str) -> Self {
        let ins = time::Instant::now();
        let still_spinning = Arc::new(AtomicBool::new(true));
        let mut stdout = io::stdout();
        let spinner_chars = ['-', '\\', '|', '/'];

        let ssp = still_spinning.clone();
        let handle = thread::spawn(move || {
            while ssp.load(Ordering::Relaxed) {
                spinner_chars.iter().for_each(|char| {
                    write!(stdout, "\r[{}] {msg}  Time: {}s", char, ins.elapsed().as_secs().add(1)).unwrap();
                    stdout.flush().unwrap();
                    thread::sleep(time::Duration::from_millis(100));
                })
            }
        });

        Self { thread_handle: Some(handle), still_spinning }
    }

    pub fn stop(&mut self, msg: &'static str) {
        if let Some(handle) = self.thread_handle.take() {
            self.still_spinning.store(false, Ordering::Relaxed);
            handle.join().unwrap();
        }
        println!("\n[*] {msg}")
    }
}
