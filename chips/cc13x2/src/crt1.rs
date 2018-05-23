use cortexm4::{generic_isr, nvic, svc_handler, systick_handler};

extern "C" {
    // Symbols defined in linker file
    static mut _erelocate: u32;
    static mut _etext: u32;
    static mut _ezero: u32;
    static mut _srelocate: u32;
    static mut _szero: u32;
    fn reset_handler();

    // _estack is not really a function but it makes types work
    // Never invoke it!!
    fn _estack();
}





