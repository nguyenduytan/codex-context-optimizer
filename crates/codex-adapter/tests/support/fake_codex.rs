use std::{
    io::{self, Read, Write},
    thread,
    time::Duration,
};
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--version"] {
        println!("codex-cli 0.153.4");
        return;
    }
    if args == ["exec", "--help"] {
        println!("--json --config --ephemeral --sandbox --model");
        return;
    }
    let mut prompt = String::new();
    io::stdin().read_to_string(&mut prompt).unwrap();
    let mode = std::env::var("CTXC_FAKE_SCENARIO").unwrap_or_default();
    if mode == "wait" || prompt == "FAKE_WAIT" {
        thread::sleep(Duration::from_secs(10));
    }
    if mode == "malformed" {
        println!("raw fallback evidence");
    }
    if mode == "quota" {
        println!(r#"{{"type":"turn.failed","error":{{"message":"quota exceeded"}}}}"#);
        std::process::exit(9);
    }
    if mode == "unsupported" {
        eprintln!("unsupported option");
        std::process::exit(2);
    }
    println!(
        "{}",
        serde_json::json!({"type":"item.completed","item":{"type":"agent_message","text":if mode == "echo" {prompt} else {"fake complete".into()}}})
    );
    if mode != "missing" {
        println!(
            r#"{{"type":"turn.completed","usage":{{"input_tokens":42,"cached_input_tokens":10,"output_tokens":7}}}}"#
        );
    }
    io::stdout().flush().unwrap();
}
