use lccc_siphash::siphash::sys::SipHashState;

#[derive(Clone, Debug)]
pub struct Rand(SipHashState);

impl Rand {
    #[allow(unused_parens)] // Removing them changes how the macro is parsed
    pub fn init() -> Self {
        let mut keys;

        cfg_match::cfg_match! {
            unix => ({
                unsafe {
                    keys = [0, 0];
                    libc::getrandom(keys.as_mut_ptr().cast(), 16, libc::GRND_RANDOM);
                }
            }),
            windows => ({
                unsafe {
                    keys = [0, 0];

                    let mut handle = std::mem::zeroed();
                    let status = windows_sys::Win32::Security::Cryptography::BCryptOpenAlgorithmProvider(
                        &mut handle,
                        windows_sys::Win32::Security::Cryptography::BCRYPT_AES_ALGORITHM,
                        std::ptr::null(),
                        0,
                    );
                    assert_eq!(status, windows_sys::Win32::Foundation::STATUS_SUCCESS);

                    windows_sys::Win32::Security::Cryptography::BCryptGenRandom(
                        handle,
                        keys.as_mut_ptr().cast(),
                        16,
                        0
                    );
                    windows_sys::Win32::Security::Cryptography::BCryptCloseAlgorithmProvider(
                        handle,
                        0
                    );
                }
            }),
            target_os = "lilium" => ({
                use lilium_sys::sys::random::{GetRandomBytes, RANDOM_DEVICE};
                keys = [0, 0];

                let err = unsafe{GetRandomBytes(keys.as_mut_ptr().cast(), 16, RANDOM_DEVICE)};
                assert_eq!(err, 0);
            })
            _ => compile_error!("unsupported platform due to inability to generate random number")
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
