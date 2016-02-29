use std::num::Wrapping as W;

const PAGE_MASK         : W<u16> = W(0xFF00 as u16);

pub trait LoadStore {
    fn load(&mut self, address: W<u16>) -> W<u8>;
    fn store(&mut self, address: W<u16>, value: W<u8>);

    fn load_word(&mut self, address: W<u16>) -> W<u16> {
        let low = W16!(self.load(address));
        (W16!(self.load(address + W(1))) << 8) | low
    }

    fn store_word(&mut self, address: W<u16>, word: W<u16>) {
        self.store(address, W8!(word >> 8));
        self.store(address + W(1), W8!(word));
    }

    fn load_word_page_wrap(&mut self, address: W<u16>) -> W<u16> {
        let low = self.load(address);
        let high = self.load((address & PAGE_MASK) | W16!(W8!(address) + W(1)));
        (W16!(high) << 8) | W16!(low)
    }
}
