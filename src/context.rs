pub static mut VERBOSE : u8 = 0;

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
