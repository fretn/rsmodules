macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

macro_rules! executable(
    () => ({
        let module = module_path!();
        if &module[0..3] == "uu_" {
            &module[3..]
        } else {
            module
        }
    })
);

macro_rules! pipe_write(
    ($fd:expr, $($args:tt)+) => (
        match write!($fd, $($args)+) {
            Ok(_) => true,
            Err(f) => {
                if f.kind() == ::std::io::ErrorKind::BrokenPipe {
                    false
                } else {
                    panic!("{}", f)
                }
            }
        }
    )
);

macro_rules! pipe_writeln(
    ($fd:expr, $($args:tt)+) => (
        match writeln!($fd, $($args)+) {
            Ok(_) => true,
            Err(f) => {
                if f.kind() == ::std::io::ErrorKind::BrokenPipe {
                    false
                } else {
                    panic!("{}", f)
                }
            }
        }
    )
);

macro_rules! show_error(
    ($($args:tt)+) => ({
        pipe_write!(&mut ::std::io::stderr(), "{} error: ", executable!());
        pipe_writeln!(&mut ::std::io::stderr(), $($args)+);
    })
);

macro_rules! show_warning(
    ($($args:tt)+) => ({
        pipe_write!(&mut ::std::io::stderr(), "{} warning: ", executable!());
        pipe_writeln!(&mut ::std::io::stderr(), $($args)+);
    })
);

macro_rules! crash(
    ($exitcode:expr, $($args:tt)+) => ({
        show_error!($($args)+);
        ::std::process::exit($exitcode)
    })
);

macro_rules! exit(
    ($exitcode:expr) => ({
        ::std::process::exit($exitcode)
    })
);

macro_rules! crash_if_err(
    ($exitcode:expr, $exp:expr) => (
        match $exp {
            Ok(m) => m,
            Err(f) => crash!($exitcode, "{}", f),
        }
    )
);
