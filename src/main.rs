#![cfg_attr(target_arch = "arm", feature(core_intrinsics))]
#![feature(lang_items)]
#![feature(asm)]
#![feature(unique)]
#![feature(const_fn)]
#![feature(linkage)]
#![feature(compiler_builtins_lib)]

#![no_std]

#[macro_use]
extern crate bitflags;
extern crate spin;
extern crate compiler_builtins;

use core::ptr::Unique;

#[macro_use]
mod serial;
mod lang_items;

use core::str;

mod m0;
use m0::{PerilpM0, M0};

extern crate rk3399_tools;

extern crate rockchip;
use rockchip::i2c::{I2C, I2CTrait};

mod clock_init;
use clock_init::setup_clocks;

const MAX_WAIT_COUNT:u32 = 1000;

#[repr(C)]
struct rk3399_pmusgrf_regs {
	ddr_rgn_con:[u32; 35],
	reserved:[u32; 0x1fe5],
	soc_con8:u32,
	soc_con9:u32,
	soc_con10:u32,
	soc_con11:u32,
	soc_con12:u32,
	soc_con13:u32,
	soc_con14:u32,
	soc_con15:u32,
	reserved1:[u32; 3],
	soc_con19:u32,
	soc_con20:u32,
	soc_con21:u32,
	soc_con22:u32,
	reserved2:[u32; 0x29],
	perilp_con:[u32; 9],
	reserved4:[u32; 7],
	perilp_status:u32,
	reserved5:[u32; 0xfaf],
	soc_con0:u32,
	soc_con1:u32,
	reserved6:[u32; 0x3e],
	pmu_con:[u32; 9],
	reserved7:[u32; 0x17],
	fast_boot_addr:u32,
	reserved8:[u32; 0x1f],
	efuse_prg_mask:u32,
	efuse_read_mask:u32,
	reserved9:[u32; 0x0e],
	pmu_slv_con0:u32,
	pmu_slv_con1:u32,
	reserved10:[u32; 0x771],
	soc_con3:u32,
	soc_con4:u32,
	soc_con5:u32,
	soc_con6:u32,
	soc_con7:u32,
	reserved11:[u32; 8],
	soc_con16:u32,
	soc_con17:u32,
	soc_con18:u32,
	reserved12:[u32; 0xdd],
	slv_secure_con0:u32,
	slv_secure_con1:u32,
	reserved13:u32,
	slv_secure_con2:u32,
	slv_secure_con3:u32,
	slv_secure_con4:u32,
}

fn main() {
	// print!(serial, "\x1b[20h");

	let grf = unsafe { &*rk3399_tools::GRF.get() };
	println!("Chip version: {:x}", grf.grf_chip_id_addr.read().bits());

	// connect PMCU JTAG
	// first, setup IOMUX to select JTAG
	let pmugrf = unsafe { &*rk3399_tools::PMUGRF.get() };

	pmugrf.pmugrf_gpio1b_iomux.modify(|_, w| unsafe {
		w.
		gpio1b1_sel().bits(0). 	// pmum0jtag_tck
		gpio1b2_sel().bits(0)	// pmum0jtag_tms
	});

	// and enable SWD for the core
	// that is, set sgrf_mcu_dbgen to 1
	// lives in sgrf_pmu_con0[5], which ISN'T IN THE REF DOC :(
	let mut pmusgrf:*mut rk3399_pmusgrf_regs = 0xff33_0000  as *mut rk3399_pmusgrf_regs;
	unsafe {
		let mut pmu_con = &mut (*pmusgrf).pmu_con;
		let mut sgrf_pmu_con0:*mut u32 = &mut pmu_con[0];
		*sgrf_pmu_con0 = *sgrf_pmu_con0 | (1 << 5);
	}

	println!("SWD for PMU enabled\n");

	// start the M0
	// let addr:u32 = 0x250000;
	// println!("Starting M0 at 0x{:x}...", addr);
	// let mut littleguy = PerilpM0 { serial: 
	// 	serial::Uart16650 {
	// 		base: unsafe { 
	// 			Unique::new (0xFF1A0000 as *mut u8)  // UART2
	// 		}
	// 	}
	// };

	// unsafe {
	// 	littleguy.setup (addr);
	// 	littleguy.on ();
	// }

	// return;

	// setup_clocks();
	// println!("finished clock setup!\n");


	// try to read i2c
	// register 0x28 on rk808 should read back 0b00011111 = 31
	let i2c_regs = unsafe { &*rk3399_tools::I2C0.get() };
	let i2c = I2C(i2c_regs);
	println!("reading from RK808...");

	let mut request_data:[u8; 1] = [0; 1];
	let data = i2c.read_from(0x1b, 0x05, &mut request_data);
	println!("read back from device: {:?}", request_data);

	println!("i2c buf was: {:?}", i2c_regs.rki2c_rxdata[0].read().bits());

	return;

	// println!("OK, now I'll echo every line back at you!");

	// let mut buf:[u8; 128] = [0; 128];
	// loop {
	// 	let end = serial::STDOUT.lock().read_line(&mut buf);

	// 	let s = unsafe { str::from_utf8_unchecked(&buf[..end]) };
	// 	for tok in s.split(" ") {
	// 		print!("tok: {}\n", tok);
	// 	}
	// }

	// print_clocks();

	// okay, so PWRDN_CON seems to be used to turn on/off power domains
	// (after idling the bus via the PMU as well)
	// and PWRDN_ST is used to check the state

	let pmu = unsafe { &*rk3399_tools::PMU.get() };

	println!("gmac on? {:?}", pmu.pmu_pwrdn_st.read().pd_gmac_pwr_stat().bit_is_clear());

	// if already in state we want to transition to, we're done
	// otherwise...

	// if we want to turn it on, we call `pmu_power_domain_ctr`
	// which enables the power domain

	// now we handle the bus via `pmu_bus_idle_req`
	// if we're turning on, we request the bus go active
	// if we're turning off, we request the bus go idle:
	pmu.pmu_bus_idle_req.modify(|_, w| w.idle_req_gmac().bit(true));

	let mut bus_timeout = true;
	for _ in 1..MAX_WAIT_COUNT {
		let bus_state = pmu.pmu_bus_idle_st.read().idle_gmac().bit_is_set();
		let bus_ack = pmu.pmu_bus_idle_ack.read().idle_ack_gmac().bit_is_set();

		// while ((bus_state != bus_req || bus_ack != bus_req)
		// and bus_req = state ? bus_id : 0  (ie target for bit is 1 if turn off, or bit unset if turning on)
		if bus_ack || bus_state {
			bus_timeout = false;
			break;
		}
	}

	if bus_timeout {
		println!("had timeout while idling bus");
		println!("gmac bus state was idle? {:?}", pmu.pmu_bus_idle_st.read().idle_gmac().bit_is_set());
		println!("gmac bus state had idle acknoledge? {:?}", pmu.pmu_bus_idle_ack.read().idle_ack_gmac().bit_is_set());
	}

	// if we're powering on, we're done! it has power and the bus is back
	// if we're powering off, we finally need to disable the power domain:
	pmu.pmu_pwrdn_con.modify(|_, w| w.pd_gmac_pwrdwn_en().bit(true));

	unsafe { asm!("dsb sy"); }

	// now, keep checking to see if it actually turned off
	let mut pd_timeout = true;
	for _ in 1..MAX_WAIT_COUNT {
		let powered_off = pmu.pmu_pwrdn_st.read().pd_gmac_pwr_stat().bit_is_set();
		if powered_off {
			pd_timeout = false;
			break;
		}
	}

	if pd_timeout {
		println!("had timeout while disabling power domain");
		println!("pmu_pwrdn_st: {:?}", pmu.pmu_pwrdn_st.read().bits());
	}

	println!("gmac on? {:?}", pmu.pmu_pwrdn_st.read().pd_gmac_pwr_stat().bit_is_clear());

	// turn off a bunch of shit
	// pmu.pmu_pwrdn_con.modify(|_, w| unsafe { 
	// 	// USB PHY
	// 	w.pd_tcpd0_pwrdwn_en().bit(true)
	// 	.pd_tcpd1_pwrdwn_en().bit(true)

	// 	// unsure as to why turning this off
	// 	// screws everything up
	// 	// dunno if it turns off uart, halts the
	// 	// core or just messes up the bus
	// 	// could even be some weird silicon bug
	// 	//.pd_perihp_pwrdwn_en().bit(true)

	// 	.pd_rga_pwrdwn_en().bit(true)		// for LCD stuff
	// 	.pd_iep_pwrdwn_en().bit(true)		// image enhancement
	// 	.pd_vo_pwrdwn_en().bit(true)		// VOP (video out)
	// 	.pd_isp1_pwrdwn_en().bit(true)	// ISP 1
	// 	.pd_hdcp_pwrdwn_en().bit(true)	// HDCP

	// 	.pd_vdu_pwrdwn_en().bit(true)		// video decode unit
	// 	// vcodec has venc and vdec, which we DO need

	// 	.pd_gpu_pwrdwn_en().bit(true)		// GPU

	// 	// gigabit mac
	// 	// if you powerdown GMAC, then reading
	// 	// some of the power related registers causes
	// 	// aborts for some reason
	// 	.pd_gmac_pwrdwn_en().bit(true)

	// 	.pd_usb3_pwrdwn_en().bit(true)	// USB3
	// 	.pd_edp_pwrdwn_en().bit(true)		// DisplayPort
	// 	// .pd_sdioaudio_pwrdwn_en().bit(true)
	// 	.pd_sd_pwrdwn_en().bit(true)

	// 	// scu is snoop control unit i think
	// 	// for cache coherence

	// 	// cci is cache coherence interface

	// 	// in theory we can turn them all off
	// 	// though we might have to reconfigure the buses

	// 	// turn off the other little core
	// 	// FIXME: core0 has _en name from TRM but
	// 	// others don't.. wtf Rockchip
	// 	// .pd_a53_l0_pwrdwn_en().bit(true)
	// 	// on the plus side, don't disable it
	// 	// since we boot from that core!
	// 	.pd_a53_l1_pwrdwn().bit(true)
	// 	.pd_a53_l2_pwrdwn().bit(true)
	// 	.pd_a53_l3_pwrdwn().bit(true)

	// 	// turn off the big cores
	// 	.pd_a72_b0_pwrdwn_en().bit(true)
	// 	.pd_a72_b1_pwrdwn_en().bit(true)
	// });


	// print_clocks();

	// let grf = unsafe { &*rk3399_tools::GRF.get() };
	// let pmugrf = unsafe { &*rk3399_tools::PMUGRF.get() };
	// let gpio0 = unsafe { &*rk3399_tools::GPIO0.get() };
	// let gpio4 = unsafe { &*rk3399_tools::GPIO4.get() };

	// println!("switching SPDIF IOMUX to GPIO...");
	// grf.grf_gpio4c_iomux.modify(|_, w| unsafe { w.gpio4c5_sel().bits(0) });
	// pmugrf.pmugrf_gpio0b_iomux.modify(|_, w| unsafe { w.gpio0b5_sel().bits(0) });

	// println!("setting as GPIO output");
	// gpio4.gpio_swporta_ddr.modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 21)) });
	// // gpio4.gpio_swporta_dr.modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 21)) });

	// // turn on green led
	// gpio0.gpio_swporta_ddr.modify(|r, w| unsafe { w.bits(r.bits() | 1 << 13) });
	// gpio0.gpio_swporta_dr.modify(|r, w| unsafe { w.bits(r.bits() | 1 << 13) });

	// println!("all done! :)");
}
