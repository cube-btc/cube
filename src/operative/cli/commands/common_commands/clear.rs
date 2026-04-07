use std::io::Write;

// clear
pub fn clear_command() {
    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout().flush().unwrap();
}
