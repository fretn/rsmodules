// These macros where stolen from the uu coreutils project
// https://github.com/uutils/coreutils
/*
Copyright (c) Jordi Boggiano

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/










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
