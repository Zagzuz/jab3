/// Unwraps Option if Some() else substitutes provided expression
#[macro_export]
macro_rules! ward {
    ($opt:expr, $x:expr) => {
        match $opt {
            Some(result) => result,
            None => $x,
        };
    };
}

/// To be used in a loop. Log and continue if error.
#[macro_export]
macro_rules! skip {
    ($x:expr $(,)?) => {
        match $x {
            Ok(result) => result,
            Err(err) => {
                log::error!("{}", err);
                continue;
            }
        }
    };
}

/// To use in a loop. Log and break if error.
#[macro_export]
macro_rules! stop {
    ($x:expr $(,)?) => {
        match $x {
            Ok(result) => result,
            Err(err) => {
                log::error!("{}", err);
                break;
            }
        }
    };
}

#[macro_export]
macro_rules! to_eyre {
    ($opt:expr, $msg:literal) => {
        match $opt {
            Some(value) => Ok(value),
            None => Err(eyre::eyre!($msg))
        }
        // $opt.ok_or_else(eyre::eyre!($msg));
    };
    ($opt:expr, $fmt:expr, $($arg:tt)*) => {
        $opt.ok_or_else(eyre::eyre!($fmt, $($arg)*));
    };
}
