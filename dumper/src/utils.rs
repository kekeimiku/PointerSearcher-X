use std::{
    borrow::Cow,
    fmt::Display,
    io::{stdout, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

pub struct Spinner {
    thread_handle: Option<thread::JoinHandle<()>>,
    still_spinning: Arc<AtomicBool>,
}

impl Spinner {
    pub fn start(msg: impl Into<Cow<'static, str>>) -> Self {
        let still_spinning = Arc::new(AtomicBool::new(true));
        let spinner_chars = ['-', '\\', '|', '/'];
        let msg = msg.into();
        let mut stdout = stdout();
        let time = Instant::now();
        let ssp = still_spinning.clone();
        let handle = thread::spawn(move || {
            spinner_chars
                .iter()
                .cycle()
                .take_while(|_| ssp.load(Ordering::Relaxed))
                .for_each(|c| {
                    write!(stdout, "\r\x1B[34m[{c}]\x1B[0m {msg}  Time: {:.1}s", time.elapsed().as_secs_f32())
                        .expect("error: failed to write to stdout");
                    stdout.flush().expect("error: failed to flush stdout");
                    thread::sleep(Duration::from_millis(100));
                })
        });

        Self { thread_handle: Some(handle), still_spinning }
    }

    pub fn stop(&mut self, msg: impl Display) {
        self.still_spinning.store(false, Ordering::Relaxed);
        self.thread_handle.take().unwrap().join().unwrap();
        println!("\x1B[34m[*]\x1B[0m {msg}")
    }
}
