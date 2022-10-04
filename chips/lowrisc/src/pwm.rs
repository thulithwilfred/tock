//! PWM Driver
//! TODO: Need to implement PinMux to test this on Verilator
//!       which only exposes GPIO0.
//! Come back to this when PinMux is implemented

use kernel::platform;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use kernel::hil;
use core::cell::Cell;

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
        (0x02c => duty_cycle: [ReadWrite<u32, DUTY_CYCLE::Register>; 6]),
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
    /// # Arguments
    /// in_clk: peripheral clock
    /// fpwm: pwm suggested clock
    /// dc_resn_bits: num bits that define dc_resn (dc_resn + 1)
    fn calc_clk_div(&self, in_clk: u32, fpwm: u32, dc_resn_bits: u32) -> Result<u32, ErrorCode> {
        if dc_resn_bits > 16 || fpwm >= in_clk{
            return Err(ErrorCode::INVAL);
        }
        // Only allowed 27 bits for clk_div
        let clk_div_max: u32 = 134217727; // 2^27 - 1
        let clk_div: u32 = (in_clk / ((2 << dc_resn_bits) * (fpwm))) - 1;
        
        if clk_div > clk_div_max {
            return Err(ErrorCode::INVAL);
        }

        Ok(clk_div)
    }

    fn pwm_setup(&self, fpwm: u32, duty_cycle: usize) -> Result<(), ErrorCode> {
        let regs = self.registers;
        
        if regs.regwen.is_set(REGWEN::REGWEN) {
            // CFG Locked
            return Err(ErrorCode::NOSUPPORT);
        }

        if fpwm == 0 {
            for i in 0..6 {
                // Stop all 6 channels
                self.pwm_chan_stop(i);
            }
            return Ok(());
        }

        let in_clk = 2_500_000;

        // Disables and resets the phase counter
        regs.cfg.modify(CFG::CNTR_EN::CLEAR);

        // Solve for CLK_DIV to get desired frequency with DC_RESN=8 (default)
        if let Ok(clk_div) = self.calc_clk_div(in_clk as u32, fpwm, 7 + 1) {
            // Found matching config
            regs.cfg.modify(CFG::CLK_DIV.val(clk_div as u32) + CFG::DC_RESN.val(7));
        } else {
            // Cannot achieve desired config
            return Err(ErrorCode::INVAL);
        }
        
        // Enable the PWM phase counter
        regs.cfg.modify(CFG::CNTR_EN::SET);

        Ok(())
    }

    fn pwm_chan_start(&self, channel: usize, duty_cycle: u32) -> Result<(), ErrorCode> {
        if channel >= PWM_MAX_CHANS {
            // Channel index start from 0 
            return Err(ErrorCode::INVAL);
        }

        let regs = self.registers;

        regs.pwm_param[channel].write(PWM_PARAM::PHASE_DELAY.val(0x00) + PWM_PARAM::BLINK_EN_0::CLEAR + PWM_PARAM::HTBT_EN_0::CLEAR);
        regs.duty_cycle[channel].write(DUTY_CYCLE::A_0.val(duty_cycle) + DUTY_CYCLE::B_0.val(duty_cycle));

        match channel {
            0 => regs.pwm_en.modify(PWM_EN::EN_0::SET),
            1 => regs.pwm_en.modify(PWM_EN::EN_1::SET),
            2 => regs.pwm_en.modify(PWM_EN::EN_2::SET),
            3 => regs.pwm_en.modify(PWM_EN::EN_3::SET),
            4 => regs.pwm_en.modify(PWM_EN::EN_4::SET),
            5 => regs.pwm_en.modify(PWM_EN::EN_5::SET),
            // Unreachable
            _ => {},
        }

        Ok(())
    }

    fn pwm_chan_stop(&self, channel: usize) -> Result<(), ErrorCode> {
        if channel >= PWM_MAX_CHANS {
            // Channel index start from 0 
            return Err(ErrorCode::INVAL);
        }
        let regs = self.registers;

        match channel {
            0 => regs.pwm_en.modify(PWM_EN::EN_0::CLEAR),
            1 => regs.pwm_en.modify(PWM_EN::EN_1::CLEAR),
            2 => regs.pwm_en.modify(PWM_EN::EN_2::CLEAR),
            3 => regs.pwm_en.modify(PWM_EN::EN_3::CLEAR),
            4 => regs.pwm_en.modify(PWM_EN::EN_4::CLEAR),
            5 => regs.pwm_en.modify(PWM_EN::EN_5::CLEAR),
            // Unreachable
            _ => {},
        }

        Ok(())
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
        // Setup: Internal configuration to achieve desired results
        self.pwm_setup(frequency_hz as u32, duty_cycle)?;
        // Start: On specified channel with requested duty cycle
        self.pwm_chan_start(*channel, duty_cycle as u32)?;
        Ok(())
    }

    fn stop(&self, channel: &Self::Pin) -> Result<(), ErrorCode> {
        self.pwm_chan_stop(*channel)?;
        Ok(())
    }

    fn get_maximum_frequency_hz(&self) -> usize {
        unimplemented!();
    }

    fn get_maximum_duty_cycle(&self) -> usize {
        unimplemented!();
    }
}
