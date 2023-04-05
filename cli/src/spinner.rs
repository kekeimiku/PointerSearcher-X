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

impl Default for Spinner {
    fn default() -> Self {
        Self {
            thread_handle: None,
            still_spinning: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl Spinner {
    pub fn start(&mut self, msg: &'static str) {
        let ins = time::Instant::now();
        let still_spinning = self.still_spinning.clone();
        let mut stdout = io::stdout();
        let spinner_chars = ['-', '\\', '|', '/'];

        let handle = thread::spawn(move || {
            while still_spinning.load(Ordering::Relaxed) {
                spinner_chars.iter().for_each(|char| {
                    write!(stdout, "\r[{}] {msg}  Time: {}s", char, ins.elapsed().as_secs().add(1)).unwrap();
                    stdout.flush().unwrap();
                    thread::sleep(time::Duration::from_millis(100));
                })
            }
        });

        self.thread_handle = Some(handle);
    }

    pub fn stop(&mut self, msg: &'static str) {
        if let Some(handle) = self.thread_handle.take() {
            self.still_spinning.store(false, Ordering::Relaxed);
            handle.join().unwrap();
        }
        println!("\n[*] {msg}")
    }
}
