/// Seconds since the last user input (keyboard/mouse), across the whole
/// session. Returns 0 if the platform query fails, which simply means we
/// never treat the user as idle.
pub fn idle_seconds() -> u64 {
    user_idle::UserIdle::get_time()
        .map(|t| t.as_seconds())
        .unwrap_or(0)
}
