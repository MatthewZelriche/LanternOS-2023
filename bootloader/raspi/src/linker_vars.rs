extern "C" {
    pub static __PG_SIZE: u8;
    pub static __STACK_SIZE: u8;
    pub static __KERNEL_VIRT_START: u8;
    pub static __BL_STACK_END: u8;
    pub static __BL_STACK: u8;
    pub static __BL_START: u8;
    pub static __BL_END: u8;
}

#[macro_export]
macro_rules! linker_var {
    ($a:expr) => {
        unsafe { (&$a as *const u8) as u64 }
    };
}
