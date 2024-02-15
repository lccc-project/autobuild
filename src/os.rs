#[allow(unused_parens)] // Removing the parenthesis makes the macro fail to parse
pub fn is_executable(m: &std::fs::Permissions) -> bool {
    cfg_match::cfg_match! {
        unix => ({
            use std::os::unix::fs::PermissionsExt;

            (m.mode() & 0o111) != 0
        }),
        _ => true
    }
}
