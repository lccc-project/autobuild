#[allow(unused_parens)] // Removing the parenthesis makes the macro fail to parse
pub fn is_executable(m: &std::fs::Permissions) -> bool {
    cfg_match::cfg_match! {
        unix => ({
            use std::os::unix::fs::PermissionsExt;

            (m.mode() & 0o111) != 0
        }),
        target_os = "lilium" => ({
            use std::os::lilium::fs::PermissionsExt;

            m.test_permission("Executive").unwrap_or(false)
        })
        _ => true
    }
}
