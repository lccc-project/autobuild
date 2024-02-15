use lccc_siphash::siphash::sys::SipHashState;

#[derive(Clone, Debug)]
pub struct Rand(SipHashState);

impl Rand {
    pub fn init() -> Self {
        let mut keys;

        #[cfg(unix)]
        unsafe {
            keys = [0, 0];
            libc::getrandom(keys.as_mut_ptr().cast(), 16, libc::GRND_RANDOM);
        }

        let [k0, k1] = keys;

        Self(SipHashState::from_keys(k0, k1))
    }

    pub fn gen(&mut self) -> u64 {
        self.0.update_before_rounds(0x123456789ABCDEF);
        self.0.round();
        self.0.round();
        self.0.update_after_rounds(0x123456789ABCDEF);
        let mut state = self.0;
        state.update_before_final();
        state.finish()
    }
}
