pub mod logger;
pub mod levels;

#[cfg(any(feature = "logerror", feature = "logwarn", feature = "loginfo", feature = "logdebug"))]
pub extern crate chrono;

pub use crate::levels::Log;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
