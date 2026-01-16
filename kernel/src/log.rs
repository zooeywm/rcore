#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        $crate::println!("\x1b[31m[ERROR][{}] {}\x1b[0m",
            env!("CARGO_PKG_NAME"),
            format_args!($($arg)+)
        );
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        $crate::println!("\x1b[93m[WARN][{}] {}\x1b[0m",
            env!("CARGO_PKG_NAME"),
            format_args!($($arg)+)
        );
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        $crate::println!("\x1b[34m[INFO][{}] {}\x1b[0m",
            env!("CARGO_PKG_NAME"),
            format_args!($($arg)+)
        );
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        $crate::println!("\x1b[32m[DEBUG][{}] {}\x1b[0m",
            env!("CARGO_PKG_NAME"),
            format_args!($($arg)+)
        );
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {
        $crate::println!("\x1b[90m[TRACE][{}] {}\x1b[0m",
            env!("CARGO_PKG_NAME"),
            format_args!($($arg)+)
        );
    };
}
