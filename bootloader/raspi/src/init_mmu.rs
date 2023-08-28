use aarch64_cpu::{
    asm::barrier,
    registers::{MAIR_EL1, SCTLR_EL1, TCR_EL1, TTBR0_EL1, TTBR1_EL1},
};

use raspi::memory::page_table::Lvl0TableDescriptor;
use tock_registers::interfaces::{ReadWriteable, Writeable};

pub fn init_mmu(ttbr0: *const Lvl0TableDescriptor, ttbr1: *const Lvl0TableDescriptor) {
    MAIR_EL1.write(MAIR_EL1::Attr0_Device::nonGathering_nonReordering_noEarlyWriteAck);
    MAIR_EL1.write(
        MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc,
    );

    TTBR0_EL1.set_baddr(ttbr0 as u64);
    TTBR1_EL1.set_baddr(ttbr1 as u64);

    // Set to entire possible memory range
    let t0sz = (64 - 48) as u64;
    let t1sz = (64 - 48) as u64;

    // 4KiB granule, caching enabled
    TCR_EL1.write(TCR_EL1::IPS::Bits_48 + TCR_EL1::T0SZ.val(t0sz) + TCR_EL1::T1SZ.val(t1sz));
    barrier::isb(barrier::SY);
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    barrier::isb(barrier::SY);
}
