use std::process::Command;

fn main() {
    Command::new("sh")
        .arg("-c")
        .arg("open -a Calculator")
        .output()
        .expect("sh exec error!");
}

// fn main() {
//     thread::spawn(|| loop {
//         println!("fuck fuck fuck");
//         thread::sleep(Duration::from_secs(1));
//     });
// }

#[used]
#[link_section = "__DATA,__mod_init_func"]
pub static INITIALIZE: fn() = main;
