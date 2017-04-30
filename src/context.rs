pub static mut VERBOSE : u8 = 0;
pub static mut SINK: u8 = 1;

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
