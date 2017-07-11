extern crate core;
use core::ptr::{read_volatile, write_volatile};
use core::fmt::Write;
use serial;

pub struct PerilpM0 {
	// pub peripheral_base: u32,
	// address_ranges: [
	// 	((0x00000000, 0x1FFFFFFF), "WT"),
	// 	((0x20000000, 0x3FFFFFFF), "WBWA"),
	// 	((0x40000000, 0x5FFFFFFF), "XN"),
	// 	((0x60000000, 0x7FFFFFFF), "WBWA"),
	// 	((0x80000000, 0x9FFFFFFF), "WT"),
	// 	((0xA0000000, 0xDFFFFFFF), "XN"),
	// ]
}

const MMIO_BASE:u32 = 0xF8000000;

const PMU_GRF:u32 = 	0xFF32_0000;
const PMU_SGRF:u32 = 	0xFF33_0000;
const PMU_CRU:u32 = 	0xFF75_0000;


const SGRF_PERILP_CON_BASE:*mut u32 = (PMU_SGRF + 0x8100) as *mut u32;
unsafe fn SGRF_PERILP_CON (n: isize) -> *mut u32 { SGRF_PERILP_CON_BASE.offset(n) }

const SGRF_SOC_CON_BASE:*mut u32 = (PMU_SGRF + 0xc000) as *mut u32;
unsafe fn SGRF_SOC_CON (n: isize) -> *mut u32 { SGRF_SOC_CON_BASE.offset(n) }

const PMUCRU_CLKGATE_CON_BASE:*mut u32 = (PMU_CRU + 0x100) as *mut u32;
unsafe fn PMUCRU_CLKGATE_CON(n: isize) -> *mut u32 { PMUCRU_CLKGATE_CON_BASE.offset(n)}

const PMUCRU_CLKSEL_CON_BASE:*mut u32 = (PMU_CRU + 0x080) as *mut u32;
unsafe fn PMUCRU_CLKSEL_CON(n:isize) -> *mut u32 { PMUCRU_CLKSEL_CON_BASE.offset(n) }

const PMUCRU_SOFTRST_CON_BASE:*mut u32 = (PMU_CRU + 0x110) as *mut u32;
unsafe fn PMUCRU_SOFTRST_CON(n:isize) -> *mut u32 {PMUCRU_SOFTRST_CON_BASE.offset(n) }

const SGRF_PMU_CON_BASE:*mut u32 = (PMU_SGRF + 0xc100) as *mut u32;
unsafe fn SGRF_PMU_CON(n:isize) -> *mut u32 {SGRF_PMU_CON_BASE.offset(n) }

const PMUCRU_GATEDIS_CON0:*mut u32 = (PMU_CRU + 0x130) as *mut u32;

const PMUGRF_SOC_STATUS0:*mut u32 = (PMU_CRU + 0x150) as *mut u32; // correct for rk3366?

/// Gives you the the u32 that would enable the bit in the write-mask for the given bit.
/// High 16-bits are write-mask.
// same as WMSK_BIT
fn write_mask_bit (bit: u8) -> u32 { 
	(1 << (bit + 16))
}

// same as BIT_WITH_WMASK
fn bit_with_writemask_set (bit: u8) -> u32 {
	(1 << bit) | write_mask_bit (bit)
}

// same as BITS_WITH_WMASK
fn bits_with_writemask_set (bits: u32, writemask_bits: u32, shift: u32) -> u32 {
	(bits << shift) | (writemask_bits << (shift + 16))
}

// pub unsafe fn set_bitmask_volatile<T: core::ops::BitAnd>(dst: *mut T, bitmask: T) {
// 	write_volatile (dst, read_volatile::<T> (dst) & bitmask)
// }

pub unsafe fn set_bitmask_volatile(dst: *mut u32, bitmask: u32) {
	write_volatile (dst, read_volatile (dst) & bitmask)
}

pub trait M0 {
    unsafe fn setup(&mut self, start: u32);
    unsafe fn on(&mut self);
}

impl M0 for PerilpM0 {
	unsafe fn setup(&mut self, start: u32) {
		// disable security on M0
		//	sgrf_pmu_con0[7] = 1 for PMU
		// 	sgrf_soc_con6[13] = 1 for PERILPM0
		// code also does something else but I think it's a bug that's meant to turn off PERILPM0 security too

		// write_volatile::<u32>(SGRF_SOC_CON(6), bit_with_writemask_set(13));
		write_volatile::<u32>(SGRF_PMU_CON(0), bit_with_writemask_set(7));
		write_volatile::<u32>(SGRF_SOC_CON(6), bit_with_writemask_set(12));

		// remap the 0x00000000-0x1FFFFFFF region to point to `addr`
		// middle 26 bits of address
		//write_volatile::<u32>(SGRF_PERILP_CON(3), bits_with_writemask_set((start >> 12) & 0xffff, 0xffff, 0));
		write_volatile::<u32>(SGRF_PMU_CON(3), bits_with_writemask_set((start >> 12) & 0xffff, 0xffff, 0));

		// high 4 bits of address
        //write_volatile::<u32>(SGRF_PERILP_CON(7), bits_with_writemask_set((start >> 28) & 0xf, 0xf, 0));
        write_volatile::<u32>(SGRF_PMU_CON(7), bits_with_writemask_set((start >> 28) & 0xf, 0xf, 0));

        // set clk_pmum0_gating_dis to 0 (enables clocking gate)
        // though, we want bit 13 (0 indexed; clk_perilpm0_gating_dis)
        //set_bitmask_volatile(PMUCRU_GATEDIS_CON0, 1 << 13);
        set_bitmask_volatile(PMUCRU_GATEDIS_CON0, 0x02);

        // then it selects 24MHz clock divider source
        // and clears the dividers..?

        // then disables hclk_noc_pmu

        // we'll.. do the same for now....
        write_volatile::<u32>(PMUCRU_CLKSEL_CON(0),
        	bit_with_writemask_set(15) | bits_with_writemask_set(0x0, 0b11111, 8));

       	write_volatile::<u32>(PMUCRU_CLKGATE_CON(2), bit_with_writemask_set(5));
    }

    unsafe fn on (&mut self) {
    	// turn on clocks
    	// enables dclk_cm0s, hclk_cm0s, sclk_cm0s, fclk_cm0s clocks
    	write_volatile::<u32>(PMUCRU_CLKGATE_CON(2), bits_with_writemask_set(0, 0b1111, 0));

    	// then do a reset
    	// resets hresetn_cm0s_pmu
    	write_volatile::<u32>(PMUCRU_SOFTRST_CON(0), bits_with_writemask_set(0, 0b100, 0));

    	// sleep for 5 usecs?
    	for i in 1..99999 {
    		asm!("nop");
    	}

    	// println!("Triggering PORESETn and HRESETn...");

    	// resets hresetn_cm0s_pmu and poresetn_cm0s_pmu
    	//
    	// from ARM manual:
    	// HRESETn: Power on reset for the HCLK domain.
    	//			Must not be the same as core HCLK reset
    	//			(SYSRESETn).
    	// PORESETn Resets the entire processor system with
    	//			the exception of SWJ-DP
	    write_volatile::<u32>(PMUCRU_SOFTRST_CON(0), bits_with_writemask_set(0, 0b100100, 0));

	    // println!("PMU status bits: {:?}", read_volatile (PMUGRF_SOC_STATUS0));
    }
}