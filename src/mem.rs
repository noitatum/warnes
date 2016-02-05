pub struct Memory {
    ram : [u8; 2048],
}

impl Default for Memory {
    fn default () -> Memory {
        Memory {
            ram  : [0;  2048],
        }
    }
}
