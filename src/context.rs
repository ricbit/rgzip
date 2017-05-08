pub static mut VERBOSE : u8 = 0;
pub static mut SINK: u8 = 1;
pub static mut SOURCE: u8 = 0;
pub static mut BUFFER: u8 = 1;
pub static mut ADAPTER: u8 = 0;

macro_rules! get_context {
    ($var: expr) => {
        unsafe {
            $var
        }
    }
}

macro_rules! verbose {
    ( $level: expr, $( $x:expr ),* ) => {
        unsafe {
            if VERBOSE >= $level {
                println!(
                    $(
                        $x,
                    )*
                );
            }
        }
    };
}
