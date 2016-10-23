pub fn print_mem(output: &mut String, mem: &[u8]) {
    output.push_str("|00|01|02|03|04|05|06|07|08|09|0A|0B|0C|0D|0E|0F|\n");
    for i in 0..mem.len() {
        output.push_str(&format!("|{:02x}", mem[i]));
        if i & 0xF == 0xF {
            output.push_str("|\n");
        }
    }
}
