use psm::psm_stack_manipulation;

psm_stack_manipulation! {
    yes {
        #[cfg(not(any(target_family = "wasm", target_os = "hermit")))]
        #[path = "alloc.rs"]
        mod heap_struct;

        #[cfg(any(target_family = "wasm", target_os = "hermit"))]
        #[path = "mmap.rs"]
        mod heap_struct;
    }

    no {
        
    }
}