use aarch64_cpu::{
    asm::barrier,
    registers::{SCTLR_EL1, TCR_EL1, TTBR0_EL1, TTBR1_EL1},
};

use raspi_paging::PageTableRoot;
use tock_registers::interfaces::{ReadWriteable, Writeable};

pub fn init_mmu(ttbr0: &PageTableRoot, ttbr1: &PageTableRoot) {
    // TODO: Set up MAIR. Currently everything is mapped to DEVICE memory/strongly ordered

    TTBR0_EL1.set_baddr(ttbr0.lvl0_table.as_ptr() as u64);
    TTBR1_EL1.set_baddr(ttbr1.lvl0_table.as_ptr() as u64);

    // Set TTBR0 to entire possible memory range
    let t0sz = (64 - 48) as u64;
    let t1sz = (64 - 48) as u64;

    // 4KiB granule, no caching
    TCR_EL1.write(TCR_EL1::IPS::Bits_48 + TCR_EL1::T0SZ.val(t0sz) + TCR_EL1::T1SZ.val(t1sz));
    barrier::isb(barrier::SY);
    SCTLR_EL1
        .modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::NonCacheable + SCTLR_EL1::I::NonCacheable);
    barrier::isb(barrier::SY);
}
