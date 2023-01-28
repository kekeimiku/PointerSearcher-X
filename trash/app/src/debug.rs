use iced_x86::{
    Decoder, DecoderOptions, Formatter, GasFormatter, IntelFormatter, MasmFormatter, NasmFormatter,
};

pub enum AsmFormat {
    Nasm,
    Masm,
    Intel,
    Gas,
}

#[repr(u8)]
pub enum Size {
    Byte = 0b00,
    Word = 0b01,
    DoubleWord = 0b11,
    QuadWord = 0b10,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Regs {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rbp: u64,
    rsp: u64,
    rip: u64,
}

impl Regs {
    pub fn new(line: &str) -> Self {
        let r = line
            .split_whitespace()
            .map(|s| s.split(':').collect::<Vec<_>>())
            .collect::<Vec<_>>();
        Self {
            rax: r[0][1].parse().unwrap(),
            rbx: r[1][1].parse().unwrap(),
            rcx: r[2][1].parse().unwrap(),
            rdx: r[3][1].parse().unwrap(),
            rsi: r[4][1].parse().unwrap(),
            rdi: r[5][1].parse().unwrap(),
            r8: r[6][1].parse().unwrap(),
            r9: r[7][1].parse().unwrap(),
            r10: r[8][1].parse().unwrap(),
            r11: r[9][1].parse().unwrap(),
            r12: r[10][1].parse().unwrap(),
            r13: r[11][1].parse().unwrap(),
            r14: r[12][1].parse().unwrap(),
            r15: r[13][1].parse().unwrap(),
            rbp: r[14][1].parse().unwrap(),
            rsp: r[15][1].parse().unwrap(),
            rip: r[16][1].parse().unwrap(),
        }
    }
}

pub fn bpf_break_point(addr: usize, size: Size) {
    let size = match size {
        Size::Byte => 1,
        Size::Word => 2,
        Size::DoubleWord => 4,
        Size::QuadWord => 8,
    };
    let cmd = format!("watchpoint:0x{:x}:{}:w ", addr, size)
        + r#"{ printf("rax:%ld rbx:%ld rcx:%ld rdx:%ld rsi:%ld rdi:%ld r8:%ld r9:%ld r10:%ld r11:%ld r12:%ld r13:%ld r14:%ld r15:%ld rbp:%ld rsp:%ld rip:%ld \n", reg("ax"), reg("bx"), reg("cx"), reg("dx"), reg("si"), reg("di"), reg("r8"), reg("r9"), reg("r10"), reg("r11"), reg("r12"), reg("r13"), reg("r14"), reg("r15"), reg("bp"), reg("sp"), reg("ip")) }"#;

    let child = std::process::Command::new("./bpftrace")
        .arg("-q")
        .arg("-e")
        .arg(cmd)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let mut out = std::io::BufReader::new(child.stdout.unwrap());
    let mut line = String::new();
    while let Ok(_) = std::io::BufRead::read_line(&mut out, &mut line) {
        let r = Regs::new(line.trim());
        println!("{:?}", r);
    }
}

// 查找指针 todo 汇总寄存器+推荐值
pub fn find_pointer(bytes: &[u8], ip: usize, addr: usize, format: AsmFormat) -> String {
    let decoder = Decoder::with_ip(64, bytes, ip as _, DecoderOptions::NONE);

    let mut formatter = match format {
        AsmFormat::Nasm => Box::new(NasmFormatter::new()) as Box<dyn Formatter>,
        AsmFormat::Masm => Box::new(MasmFormatter::new()) as _,
        AsmFormat::Intel => Box::new(IntelFormatter::new()) as _,
        AsmFormat::Gas => Box::new(GasFormatter::new()) as _,
    };

    let mut output = String::new();
    let mut ret = String::new();
    let instructions = decoder.into_iter().collect::<Vec<_>>();

    for (i, ins) in instructions.iter().enumerate() {
        if ins.next_ip() as usize == addr {
            let low = i.saturating_sub(5);
            let high = (i + 5).min(instructions.len());
            for (j, ins) in instructions.iter().enumerate().take(high).skip(low) {
                ret.push_str(&format!("{} {:016X} ", if j == i { ">>>" } else { "   " }, ins.ip()));
                let k = (ins.ip() - ip as u64) as usize;
                let instr_bytes = &bytes[k..k + ins.len()];
                for b in instr_bytes.iter() {
                    ret.push_str(&format!("{:02X}", b));
                }
                if instr_bytes.len() < 10 {
                    for _ in 0..10usize.saturating_sub(instr_bytes.len()) {
                        ret.push_str("  ");
                    }
                }
                output.clear();
                formatter.format(ins, &mut output);
                ret.push_str(&format!(" {}\n", output));
            }
            break;
        }
    }
    ret
}