//! PWM Driver

use kernel::platform;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use kernel::hil;

// Refer: https://docs.opentitan.org/hw/ip/pwm/doc/#register-table
register_structs! {
    pub PWMRegisters {
        //PWM: Alert Test Register
        (0x000 => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        //PWM: Register write enable for all control registers [rw0c]
        (0x004 => regwen: ReadWrite<u32, REGWEN::Register>),
        //PWM: Configuration register
        (0x008 => cfg: ReadWrite<u32, CFG::Register>),
        //PWM: Enable PWM operation for each channel
        (0x00c => pwm_en: ReadWrite<u32, PWM_EN::Register>),
        //PWM: Invert the PWM output for each channel
        (0x010 => invert: ReadWrite<u32, INVERT::Register>),
        //PWM: Basic PWM Channel Parameters
        (0x014 => pwm_param: [ReadWrite<u32, PWM_PARAM::Register>; 6]),
        // PWM: Controls the duty_cycle of each channel.
        (0x02c => duty_cyle: [ReadWrite<u32, DUTY_CYCLE::Register>; 6]),
        // PWM: Controls the duty_cycle of each channel.
        (0x044 => blink_param: [ReadWrite<u32, BLINK_PARAM::Register>; 6]),
        (0x05C => @END),
    }
}

register_bitfields![u32,
    ALERT_TEST[
        FATAL_FAULT OFFSET(0) NUMBITS(1) []
    ],
    REGWEN[
        REGWEN OFFSET(0) NUMBITS(1) []
    ],
    CFG[
        CLK_DIV OFFSET(0) NUMBITS(27) [],
        DC_RESN OFFSET(27) NUMBITS(4) [],
        CNTR_EN OFFSET(31) NUMBITS(1) []
    ],
    PWM_EN[
        EN_0 OFFSET(0) NUMBITS(1) [],
        EN_1 OFFSET(1) NUMBITS(1) [],
        EN_2 OFFSET(2) NUMBITS(1) [],
        EN_3 OFFSET(3) NUMBITS(1) [],
        EN_4 OFFSET(4) NUMBITS(1) [],
        EN_5 OFFSET(5) NUMBITS(1) []
    ],
    INVERT[
        INVERT_0 OFFSET(0) NUMBITS(1) [],
        INVERT_1 OFFSET(1) NUMBITS(1) [],
        INVERT_2 OFFSET(2) NUMBITS(1) [],
        INVERT_3 OFFSET(3) NUMBITS(1) [],
        INVERT_4 OFFSET(4) NUMBITS(1) [],
        INVERT_5 OFFSET(5) NUMBITS(1) []
    ],
    PWM_PARAM[
        PHASE_DELAY OFFSET(0) NUMBITS(16) [],
        HTBT_EN_0 OFFSET(30) NUMBITS(1) [],
        BLINK_EN_0 OFFSET(31) NUMBITS(1) []
    ],
    DUTY_CYCLE[
        A_0 OFFSET(0) NUMBITS(16) [],
        B_0 OFFSET(16) NUMBITS(16) []
    ],
    BLINK_PARAM[
        X_0 OFFSET(0) NUMBITS(16) [],
        Y_0 OFFSET(16) NUMBITS(16) []
    ]
];

pub const PWM_MAX_CHANS: usize = 6;

pub struct PWMCtrl {
    registers: StaticRef<PWMRegisters>,
}

impl PWMCtrl {
    pub const fn new(
        base: StaticRef<PWMRegisters>,
    ) -> PWMCtrl {
        PWMCtrl {
            registers: base,
        }
    }
}

impl hil::pwm::Pwm for PWMCtrl{
    // Note: Pin for OpenTitan specifies the channel
    type Pin = usize;
    
    fn start(
        &self,
        channel: &Self::Pin,
        frequency_hz: usize,
        duty_cycle: usize,
    ) -> Result<(), ErrorCode> {
        unimplemented!();
    }

    fn stop(&self, pin: &Self::Pin) -> Result<(), ErrorCode> {
        unimplemented!();
    }

    fn get_maximum_frequency_hz(&self) -> usize {
        unimplemented!();
    }

    fn get_maximum_duty_cycle(&self) -> usize {
        unimplemented!();
    }
}
