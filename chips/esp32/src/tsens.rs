use core::cell::Cell;
use kernel::debug;
use kernel::hil::sensors::{TemperatureClient, TemperatureDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

pub const APB_BASE: StaticRef<APBRegisters> =
    unsafe { StaticRef::new(0x6002_6000 as *const APBRegisters) };

register_structs! {
    pub APBRegisters {
        (0x00 => saradc_ctrl: ReadWrite<u32>),
        (0x04 => saradc_ctrl2: ReadWrite<u32>),
        (0x08 => saradc_filter_ctrl1: ReadWrite<u32>),
        (0x0C => _reserved1: ReadOnly<u32>),
        (0x10 => _reserved2: ReadOnly<u32>),
        (0x14 => _reserved3: ReadOnly<u32>),
        (0x18 => saradc_sar_patt_tab1: ReadWrite<u32>),
        (0x1C => saradc_sar_patt_tab2: ReadWrite<u32>),
        (0x20 => saradc_onetime_sample: ReadWrite<u32>),
        (0x24 => saradc_apb_adc_ctrl: ReadWrite<u32>),
        (0x28 => saradc_filter_ctrl0: ReadWrite<u32>),
        (0x2C => saradc_data_status1: ReadOnly<u32>),
        (0x30 => saradc_data_status2: ReadOnly<u32>),
        (0x34 => saradc_threas0_ctrl: ReadWrite<u32>),
        (0x38 => saradc_threas1_ctrl: ReadWrite<u32>),
        (0x3C => saradc_threas_ctrl: ReadWrite<u32>),
        (0x40 => saradc_int_en: ReadWrite<u32>),
        (0x44 => saradc_int_raw: ReadOnly<u32>),
        (0x48 => saradc_int_status: ReadOnly<u32>),
        (0x4C => saradc_int_clr: WriteOnly<u32>),
        (0x50 => saradc_dma_conf: ReadWrite<u32>),
        (0x54 => saradc_apb_adc_clkm_conf: ReadWrite<u32>),
        (0x58 => saradc_apb_tsens_ctrl1: ReadWrite<u32>),
        (0x5C => saradc_apb_tsens_ctrl2: ReadWrite<u32>),
        (0x60 => saradc_cali: ReadWrite<u32>),
        (0x64 => @END),
        // (0x3FC => saradc_version: ReadWrite<u32>),
    }
}

register_bitfields![u32,
    FIFO [
        RXFIFO_RD_BYTE OFFSET(0) NUMBITS(8) [],
    ],
];

pub struct tsens_ctl<'a> {
    registers: StaticRef<APBRegisters>,
    temperature_client: OptionalCell<&'a dyn TemperatureClient>,
}

impl<'a> tsens_ctl<'a> {
    pub const fn new(base: StaticRef<APBRegisters>) -> Self {
        tsens_ctl {
            registers: base,
            temperature_client: OptionalCell::empty(),
        }
    }
}

impl<'a> TemperatureDriver<'a> for tsens_ctl<'a> {
    fn set_client(&self, client: &'a dyn TemperatureClient) {
        self.temperature_client.set(client);
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        // if !self.pending_humidity.get() {
        //     self.start_reading()
        // } else {
        //     Ok(())
        // }
        self.registers.saradc_apb_tsens_ctrl1.set(0x200000);
        for _i in 0..10000 {}
        let x = self.registers.saradc_apb_tsens_ctrl1.get();
        debug!("YO: {:?}", x);
        Ok(())
    }
}
