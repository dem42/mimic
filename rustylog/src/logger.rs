///////////////////////////
// NOTE!
// Macros get inlined so you have to put the conditional compilation outside of the
// macro body
//////////////////////////

#[cfg(all(debug_assertions, feature = "logdebug"))]
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        let line = line!();
        let file = file!();
        match $level {
            Log::Debug => {
                let date = rustylog::chrono::Local::now();
                print!("DEBUG [{}] {}:{}: ", date.format("%Y-%m-%d %H:%M:%S"), file, line); println!($($arg)*);
            },
            Log::Info => {
                let date = rustylog::chrono::Local::now();
                print!("INFO [{}]: {}:{}: ", date.format("%Y-%m-%d %H:%M:%S"), file, line); println!($($arg)*);
            },
            Log::Warn => {
                let date = rustylog::chrono::Local::now();
                print!("WARN [{}]: {}:{}: ", date.format("%Y-%m-%d %H:%M:%S"), file, line); println!($($arg)*);
            },
            Log::Error => {
                let date = rustylog::chrono::Local::now();
                print!("ERROR [{}]: {}:{}: ", date.format("%Y-%m-%d %H:%M:%S"), file, line); println!($($arg)*);
            },
        }
    }
}

#[cfg(all(debug_assertions, feature = "loginfo"))]
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        let line = line!();
        let file = file!();
        match $level {
            Log::Info => {
                let date = rustylog::chrono::Local::now();
                print!("INFO [{}]: {}:{}: ", date.format("%Y-%m-%d %H:%M:%S"), file, line); println!($($arg)*);
            },
            Log::Warn => {
                let date = rustylog::chrono::Local::now();
                print!("WARN [{}]: {}:{}: ", date.format("%Y-%m-%d %H:%M:%S"), file, line); println!($($arg)*);
            },
            Log::Error => {
                let date = rustylog::chrono::Local::now();
                print!("ERROR [{}]: {}:{}: ", date.format("%Y-%m-%d %H:%M:%S"), file, line); println!($($arg)*);
            },
            _ => {},
        }
    }
}

#[cfg(all(debug_assertions, feature = "logwarn"))]
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        let line = line!();
        let file = file!();
        match $level {
            Log::Warn => {
                let date = rustylog::chrono::Local::now();
                print!("WARN [{}]: {}:{}: ", date.format("%Y-%m-%d %H:%M:%S"), file, line); println!($($arg)*);
            },
            Log::Error => {
                let date = rustylog::chrono::Local::now();
                print!("ERROR [{}]: {}:{}: ", date.format("%Y-%m-%d %H:%M:%S"), file, line); println!($($arg)*);
            },
            _ => {},
        }
    }
}

#[cfg(all(debug_assertions, feature = "logerror"))]
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        let line = line!();
        let file = file!();
        match $level {
            Log::Error => {
                let date = rustylog::chrono::Local::now();
                print!("ERROR [{}]: {}:{}: ", date.format("%Y-%m-%d %H:%M:%S"), file, line); println!($($arg)*);
            },
            _ => {},
        }
    }
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {};
}
