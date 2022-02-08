//! Support for the CSRNG hardware block on OpenTitan
//!
//! <https://docs.opentitan.org/hw/ip/csrng/doc>

use kernel::hil::entropy::{Client32, Continue, Entropy32};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

register_structs! {
    pub CsRngRegisters {
        (0x00 => intr_state: ReadWrite<u32, INTR::Register>),
        (0x04 => intr_enable: ReadWrite<u32, INTR::Register>),
        (0x08 => intr_test: WriteOnly<u32, INTR::Register>),
        (0x0C => alert_test: WriteOnly<u32>),
        (0x10 => regwen: ReadWrite<u32, REGWEN::Register>),
        (0x14 => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x18 => cmd_req: WriteOnly<u32, COMMAND::Register>),
        (0x1C => sw_cmd_sts: ReadOnly<u32, SW_CMD_STS::Register>),
        (0x20 => genbits_vld: ReadOnly<u32, GENBIT_VLD::Register>),
        (0x24 => genbits: ReadOnly<u32>),
        (0x28 => int_state_num: ReadWrite<u32>),
        (0x2C => int_state_val: ReadOnly<u32>),
        (0x30 => hw_exc_sts: ReadWrite<u32>),
        (0x34 => err_code: ReadOnly<u32>),
        (0x38 => err_code_test: ReadWrite<u32>),
        (0x3C => sel_tracking_sm: WriteOnly<u32>),
        (0x40 => tracking_sm_obs: ReadOnly<u32>),
        (0x44 => @END),
    }
}

register_bitfields![u32,
    INTR [
        CMD_REQ_DONE OFFSET(0) NUMBITS(1) [],
        ENTROPY_REQ OFFSET(1) NUMBITS(1) [],
        HW_INST_EXC OFFSET(2) NUMBITS(1) [],
        FATAL_ERR OFFSET(3) NUMBITS(1) [],
    ],
    REGWEN [
        REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CTRL [
        ENABLE OFFSET(0) NUMBITS(4) [
            ENABLE = 0xA,
            DISABLE = 0x5,
        ],
        SW_APP_ENABLE OFFSET(4) NUMBITS(4) [
            ENABLE = 0xA,
            DISABLE = 0x5,
        ],
        READ_INT_STATE OFFSET(8) NUMBITS(4) [
            ENABLE = 0xA,
            DISABLE = 0x5,
        ],
    ],
    COMMAND [
        ACMD OFFSET(0) NUMBITS(4) [
            INSTANTIATE = 1,
            RESEED = 2,
            GENERATE = 3,
            UPDATE = 4,
            UNINSTANTIATE = 5,
        ],
        CLEN OFFSET(4) NUMBITS(4) [],
        FLAGS OFFSET(8) NUMBITS(4) [],
        GLEN OFFSET(12) NUMBITS(19) [],
    ],
    GENBIT_VLD [
        GENBITS_VLD OFFSET(0) NUMBITS(1) [],
    ],
    SW_CMD_STS [
        CMD_RDY OFFSET(0) NUMBITS(1) [],
        CMD_STS OFFSET(1) NUMBITS(1) [],
    ],
];

pub struct CsRng<'a> {
    registers: StaticRef<CsRngRegisters>,

    client: OptionalCell<&'a dyn Client32>,
}

struct CsRngIter<'a, 'b: 'a>(&'a CsRng<'b>);

impl Iterator for CsRngIter<'_, '_> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.0.registers.genbits_vld.is_set(GENBIT_VLD::GENBITS_VLD) {
            Some(self.0.registers.genbits.get())
        } else {
            None
        }
    }
}

impl<'a> CsRng<'a> {
    pub const fn new(base: StaticRef<CsRngRegisters>) -> CsRng<'a> {
        CsRng {
            registers: base,
            client: OptionalCell::empty(),
        }
    }

    fn enable_interrupts(&self) {
        self.registers.intr_enable.write(
            INTR::CMD_REQ_DONE::SET
                + INTR::ENTROPY_REQ::CLEAR
                + INTR::HW_INST_EXC::SET
                + INTR::FATAL_ERR::SET,
        );
    }

    fn disable_interrupts(&self) {
        self.registers.intr_state.write(
            INTR::CMD_REQ_DONE::SET
                + INTR::ENTROPY_REQ::SET
                + INTR::HW_INST_EXC::SET
                + INTR::FATAL_ERR::SET,
        );

        self.registers.intr_enable.write(
            INTR::CMD_REQ_DONE::CLEAR
                + INTR::ENTROPY_REQ::CLEAR
                + INTR::HW_INST_EXC::CLEAR
                + INTR::FATAL_ERR::CLEAR,
        );
    }

    pub fn handle_interrupt(&self) {
        let irqs = self.registers.intr_state.extract();
        self.disable_interrupts();

        if irqs.is_set(INTR::HW_INST_EXC) {
            self.client.map(move |client| {
                client.entropy_available(&mut (0..0), Err(ErrorCode::FAIL));
            });
            return;
        }

        if irqs.is_set(INTR::FATAL_ERR) {
            self.client.map(move |client| {
                client.entropy_available(&mut (0..0), Err(ErrorCode::FAIL));
            });
            return;
        }

        if irqs.is_set(INTR::CMD_REQ_DONE) {
            if self
                .client
                .map(move |client| client.entropy_available(&mut CsRngIter(self), Ok(())))
                == Some(Continue::More)
            {
                // We need more
                if let Err(e) = self.get() {
                    self.client.map(move |client| {
                        client.entropy_available(&mut (0..0), Err(e));
                    });
                }
            }
        }
    }
}

impl<'a> Entropy32<'a> for CsRng<'a> {
    fn set_client(&'a self, client: &'a dyn Client32) {
        self.client.set(client);
    }

    fn get(&self) -> Result<(), ErrorCode> {
        self.disable_interrupts();

        if !self.registers.regwen.is_set(REGWEN::REGWEN) {
            // Registers are read only
            return Err(ErrorCode::FAIL);
        }

        self.registers.ctrl.write(
            CTRL::ENABLE::ENABLE + CTRL::READ_INT_STATE::ENABLE + CTRL::SW_APP_ENABLE::ENABLE,
        );

        self.registers.cmd_req.write(
            COMMAND::ACMD::INSTANTIATE
                + COMMAND::FLAGS.val(0)
                + COMMAND::CLEN.val(0)
                + COMMAND::GLEN.val(0),
        );
        while !self.registers.sw_cmd_sts.is_set(SW_CMD_STS::CMD_RDY) {}

        self.disable_interrupts();
        self.enable_interrupts();

        // Get 256 bits of entropy
        self.registers
            .cmd_req
            .write(COMMAND::ACMD::GENERATE + COMMAND::FLAGS.val(0) + COMMAND::GLEN.val(0x2));

        Ok(())
    }

    fn cancel(&self) -> Result<(), ErrorCode> {
        self.disable_interrupts();

        self.registers.cmd_req.write(COMMAND::ACMD::UNINSTANTIATE);

        Ok(())
    }
}
