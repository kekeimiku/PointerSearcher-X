use std::{
    io::{self, Write},
    ops::Add,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread, time,
};

use ptrsx::c64::Page;
use terminal_size::terminal_size;

pub fn select_base_module<'a>(items: &[Page<'a>]) -> Result<Vec<Page<'a>>, super::error::Error> {
    let words = items
        .iter()
        .filter_map(|m| Path::new(m.path).file_name())
        .enumerate()
        .map(|(k, v)| format!("[\x1B[32m{k}\x1B[0m: {}] ", v.to_string_lossy()));

    let (width, _) = terminal_size().ok_or("get terminal_size")?;
    let width = width.0 as usize;

    let mut s = String::with_capacity(0x2000);
    let mut current_line_len = 0;
    for word in words {
        let word_len = word.len() + 1;
        if current_line_len + word_len > width {
            s.push('\n');
            current_line_len = word_len;
        } else {
            s.push(' ');
            current_line_len += word_len;
        }
        s.push_str(&word);
    }

    println!("{s}\n\x1B[33mSelect base modules, multiple separated by spaces.\x1B[0m");

    let mut selected_items = vec![];
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input
        .split_whitespace()
        .map(|n| n.parse())
        .collect::<Result<Vec<usize>, _>>()?;

    for k in input {
        if k > items.len() {
            break;
        }
        selected_items.push(items[k].to_owned())
    }

    if selected_items.is_empty() {
        return Err("Select at least one.".into());
    }

    Ok(selected_items)
}

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
                    write!(stdout, "\r\x1B[34m[{}]\x1B[0m {msg}  Time: {}s", char, ins.elapsed().as_secs().add(1))
                        .unwrap();
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
        println!("\n\x1B[34m[*]\x1B[0m {msg}")
    }
}
