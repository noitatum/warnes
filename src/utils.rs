pub fn print_mem(output: &mut String, mem: &[u8]) {
    for i in 0..mem.len() {
        output.push_str(&format!("{:02x} ", mem[i]));
        if i & 0xF == 0xF {
            output.push_str("\n");
        }
    }
}
