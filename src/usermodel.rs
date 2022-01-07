///
/// Simple trait definition for any concrete user model
///
pub trait UserModel {
    fn new() -> Self;
    /// Sample the next message timing for this
    /// user model
    fn get_current_time(&self) -> u64;
    fn set_limit(&mut self, limit: u64);
    fn get_next_message_timing(&mut self) -> u64;
}

