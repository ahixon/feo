extern crate core;
extern crate rk3399_tools;

pub struct PerilpM0 { }

pub trait M0 {
    fn setup(&mut self, pmusgrf: &rk3399_tools::PMUSGRF,
        pmucru: &rk3399_tools::PMUCRU, start: u32);

    fn on(&mut self, pmucru: &rk3399_tools::PMUCRU);
}

// WMSK_BIT(x)       => BIT(x + 16)          => 1 << (x + 16)
// BIT_WITH_WMASK(x) => BIT(x) | WMSK_BIT(x) => (1 << x) | (1 << (x + 16))
// BITS_WITH_WMASK(x, y, z) -> 
impl M0 for PerilpM0 {
	fn setup(&mut self, pmusgrf: &rk3399_tools::PMUSGRF,
        pmucru: &rk3399_tools::PMUCRU, start: u32) {

        // put PMU M0 into secure mode
		pmusgrf.pmu_con0.write(|w| unsafe { w.
            sgrf_pmu_cm0_mst_ctrl().clear_bit().
            write_mask().bits(1 << 7)
        });

        // m0_init also puts secure master for perilp
        // but there's sometyhing fishy going on:
        // docs say sgrf_con_perim0_secure_ctrl is [13] for PERILP
        // code does [12] for PMU
        // secure master table has:
        // [12] - perlip
        // [13] - pmu
        //
        // maybe they're flipped around?
        // but then why are there duplitate secure settings?
        //
        // let's go with the code for now...
        // sets to 0
        pmusgrf.soc_con6.write(|w| unsafe { w.
            write_enable().bits(1 << 12)
        });

        // middle 16 bits
        pmusgrf.pmu_con3.write(|w| unsafe { w.
            pmu_remap_flash_rom_mid().bits((start >> 12) as u16).
            write_mask().bits(0xffff)
        });

        // high 4 bits
        pmusgrf.pmu_con7.write(|w| unsafe { w.
            pmu_remap_flash_rom_high().bits((start >> 28) as u8).
            write_mask().bits(0xf)
        });

        // writes 0x2 to this?
        // m0_init also disables clk_center1 but probably a bug
        // but surely we just want to set first bit to 1?
        pmucru.pmucru_gatedis_con0.modify(|_, w| w.
            clk_pmum0_gating_dis().clear_bit().
            clk_center1_gating_dis().set_bit()  // FIXME: do we need this?
        );

        // FIXME: do we actually need this? find out what it does!
        // write_volatile::<u32>(PMUCRU_CLKGATE_CON(28), 1 << (16 + 5));

    }

    fn on (&mut self, pmucru: &rk3399_tools::PMUCRU) {
        // enable clocks
        pmucru.pmucru_clkgate_con2.write(|w| w.
            fclk_cm0s_en().clear_bit().
            sclk_cm0s_en().clear_bit().
            hclk_cm0s_en().clear_bit().
            dclk_cm0s_en().clear_bit()
        );

    	// pull hresetn_cm0s_pmu high
        pmucru.pmucru_softrst_con0.write(|w| unsafe { w.
            hresetn_cm0s_pmu_req().clear_bit().
            write_mask().bits(1 << 2)
        });

    	// sleep for 5 usecs?
    	for _ in 1..99999 {
    		unsafe { asm!("nop"); }
    	}

        // now pull poresetn_cm0s_pmu high
        pmucru.pmucru_softrst_con0.write(|w| unsafe { w.
            poresetn_cm0s_pmu_req().clear_bit().
            write_mask().bits(1 << 5)
        });
    }
}