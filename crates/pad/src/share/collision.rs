pub fn would_collide(remote_initial_text: &str) -> bool {
    let trimmed = remote_initial_text.trim();
    !trimmed.is_empty()
}
