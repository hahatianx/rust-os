use bitflags::bitflags;

bitflags! {

    #[derive(Debug)]
    pub struct CpuFlags: u32 {
        const CARRY_FLAG = 1 << 0;
        const _ = 1 << 1;
        const PARITY_FLAG = 1 << 2;
        const _ = 1 << 3;
        const AUXILIARY_CARRY_FLAG = 1 << 4;
        const ZERO_FLAG = 1 << 6;
        const SIGN_FLAG = 1 << 7;
        const TRAP_FLAG = 1 << 8;
        const INTERRUPT_ENABLE_FLAG = 1 << 9;
        const DIRECTION_FLAG = 1 << 10;
        const OVERFLOW_FLAG = 1 << 11;
        const IO_PRIVILEGE_LEVEL = 3 << 12;
        const NESTED_TASK_FLAG = 1 << 14;
        const MODE_FLAG = 1 << 15;

        // EFLAGS not implenmented
    }

}