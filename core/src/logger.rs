/// 彩色日志宏，S-mode 标签
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        $crate::println!("\x1b[31m[ERROR][S] {}\x1b[0m", format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        $crate::println!("\x1b[93m[WARN][S] {}\x1b[0m", format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        $crate::println!("\x1b[34m[INFO][S] {}\x1b[0m", format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        $crate::println!("\x1b[32m[DEBUG][S] {}\x1b[0m", format_args!($($arg)+));
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {
        $crate::println!("\x1b[90m[TRACE][S] {}\x1b[0m", format_args!($($arg)+));
    };
}
