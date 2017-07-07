use hal;
use core::any::{Any, TypeId};
use core::ops::Deref;
use core::ptr;

use rk3399_tools::{PMUCRU, CRU, GRF, PMUGRF};



#[allow(non_camel_case_types)]
pub enum PLLSource {
    Slow_24MHz_28MHz,
    Normal,
    DeepSlow_32_768MHz
}

pub struct PLLConfiguration {
    pub source:PLLSource
}

pub trait PLL {
    type Time;
    fn get_config(&self) -> PLLConfiguration;

    /// update clock speed, may fail if any of the dependent peripherals
    /// cannot use the new clock configuration
    // TODO: use proper Error result type
    fn set_config(&self, PLLConfiguration) -> Result<(), ()>;
}


pub struct ClockManager {

}

impl ClockManager {
    // each clock has a parent, except the top level PLLs
    // each clock has some rate determined by its configuration (ie divider/multipler rate, and
    // selected input mux if any), as well as the ability to be gated on or off.
    // each clock may be active, depending on its gating state, or one of its parents'.
    //
    // a lower level clock in the tree might set some restriction on its rate
    //
    // any clock can be reconfigured, and any devices using the clock must be notified
    // before and after any clock rate change, in case they need to reconfigure themselves
    // (eg if the UART's parent clock changes, then the UART needs to change its own divider
    // to keep itself outputting at the configured baud rate). to take the UART example,
    // it needs to pause transmission, ACK the clock change, let the clock change happen,
    // then reconfigure its clock, then resume transmission.
    //
    // some devices may not permit changing the clock rate after they've been activated,
    // or the new clockrate may place the peripheral in an invalid state. as such, active
    // peripherals that share that clock may decline a clock rate change.
    //
    // clock dependency tree worst-case can be determined statically, but if dynamically
    // adjustable, can only be known at runtime.
    //
    // get 

    // GPLL
}