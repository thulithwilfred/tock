//! Serial Peripheral Interface (SPI) Driver
use core::option::Option;
use kernel::debug;
use kernel::hil;
use kernel::hil::spi::{ClockPhase, ClockPolarity, SpiSlaveClient};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

register_structs! {
    pub SpiDeviceRegisters {
        (0x000 => intr_state: ReadWrite<u32, INTR::Register>),
        (0x004 => intr_enable: ReadWrite<u32, INTR::Register>),
        (0x008 => intr_test: WriteOnly<u32, INTR::Register>),
        (0x00C => control: ReadWrite<u32, CONTROL::Register>),
        (0x010 => cfg: ReadWrite<u32, CFG::Register>),
        (0x014 => fifo_level: ReadWrite<u32, FIFO_LEVEL::Register>),
        (0x018 => async_fifo_level: ReadOnly<u32, ASYNC_FIFO_LEVEL::Register>),
        (0x01C => status: ReadOnly<u32, STATUS::Register>),
        (0x020 => rxf_ptr: ReadWrite<u32, RXF_PTR::Register>),
        (0x024 => txf_ptr: ReadWrite<u32, TXF_PTR::Register>),
        (0x028 => rxf_addr: ReadWrite<u32, RXF_ADDR::Register>),
        (0x02C => txf_addr: ReadWrite<u32, TXF_ADDR::Register>),
        (0x030 => _reserved0),
        (0x800 => buffer: [ReadWrite<u32>; 512]),
        (0x1000 => @END),
    }
}

register_bitfields![u32,
    INTR [
        RXF OFFSET(0) NUMBITS(1) [],
        RXLVL OFFSET(1) NUMBITS(1) [],
        TXLVL OFFSET(2) NUMBITS(1) [],
        RXERR OFFSET(3) NUMBITS(1) [],
        RXOVERFLOW OFFSET(4) NUMBITS(1) [],
        TXUNDERFLOW OFFSET(5) NUMBITS(1) []
    ],
    CONTROL [
        ABORT OFFSET(0) NUMBITS(1) [],
        MODE OFFSET(4) NUMBITS(2) [],
        RST_TXFIFO OFFSET(16) NUMBITS(1) [],
        RST_RXFIFO OFFSET(17) NUMBITS(2) []
    ],
    CFG [
        CPOL OFFSET(0) NUMBITS(1) [],
        CPHA OFFSET(1) NUMBITS(1) [],
        TX_ORDER OFFSET(2) NUMBITS(1) [],
        RX_ORDER OFFSET(3) NUMBITS(1) [],
        TIMER_V OFFSET(8) NUMBITS(8) []
    ],
    FIFO_LEVEL [
        RXLVL OFFSET(0) NUMBITS(16) [],
        TXLVL OFFSET(16) NUMBITS(16) []
    ],
    ASYNC_FIFO_LEVEL [
        RXLVL OFFSET(0) NUMBITS(8) [],
        TXLVL OFFSET(16) NUMBITS(8) []
    ],
    STATUS [
        RXF_FULL OFFSET(0) NUMBITS(1) [],
        RXF_EMPTY OFFSET(1) NUMBITS(1) [],
        TXF_FULL OFFSET(2) NUMBITS(1) [],
        TXF_EMPTY OFFSET(3) NUMBITS(1) [],
        ABORT_DONE OFFSET(4) NUMBITS(1) [],
        CSB OFFSET(5) NUMBITS(1) []
    ],
    RXF_PTR [
        RPTR OFFSET(0) NUMBITS(16) [],
        WPTR OFFSET(16) NUMBITS(16) []
    ],
    TXF_PTR [
        RPTR OFFSET(0) NUMBITS(16) [],
        WPTR OFFSET(16) NUMBITS(16) []
    ],
    RXF_ADDR [
        BASE OFFSET(0) NUMBITS(16) [],
        LIMIT OFFSET(16) NUMBITS(16) []
    ],
    TXF_ADDR [
        BASE OFFSET(0) NUMBITS(16) [],
        LIMIT OFFSET(16) NUMBITS(16) []
    ]
];

pub struct SpiDevice {
    _registers: StaticRef<SpiDeviceRegisters>,

    client: OptionalCell<&'static dyn hil::spi::SpiSlaveClient>,
}

impl SpiDevice {
    pub const fn new(base: StaticRef<SpiDeviceRegisters>) -> Self {
        SpiDevice {
            _registers: base,
            client: OptionalCell::empty(),
        }
    }
}

impl hil::spi::SpiSlave for SpiDevice {
    fn init(&self) -> Result<(), ErrorCode> {
        unimplemented!();
    }

    fn has_client(&self) -> bool {
        unimplemented!();
    }

    fn set_client(&self, client: Option<&'static dyn hil::spi::SpiSlaveClient>) {
        if client.is_some() {
            self.client.set(client.unwrap());
        } else {
            self.client.take();
        }
    }

    fn set_write_byte(&self, _write_byte: u8) {
        unimplemented!();
    }

    fn read_write_bytes(
        &self,
        _write_buffer: Option<&'static mut [u8]>,
        _read_buffer: Option<&'static mut [u8]>,
        _len: usize,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<&'static mut [u8]>,
            Option<&'static mut [u8]>,
        ),
    > {
        unimplemented!();
    }

    fn set_polarity(&self, _polarity: ClockPolarity) -> Result<(), ErrorCode> {
        debug!("Crashing and burning...\n\n");
        unimplemented!();
    }

    fn get_polarity(&self) -> ClockPolarity {
        unimplemented!();
    }

    fn set_phase(&self, _phase: ClockPhase) -> Result<(), ErrorCode> {
        unimplemented!();
    }

    fn get_phase(&self) -> ClockPhase {
        unimplemented!();
    }
}

impl hil::spi::SpiSlaveDevice for SpiDevice {
    fn configure(&self, _cpol: ClockPolarity, _cpal: ClockPhase) -> Result<(), ErrorCode> {
        unimplemented!();
    }

    fn read_write_bytes(
        &self,
        _write_buffer: Option<&'static mut [u8]>,
        _read_buffer: Option<&'static mut [u8]>,
        _len: usize,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<&'static mut [u8]>,
            Option<&'static mut [u8]>,
        ),
    > {
        unimplemented!();
    }

    fn set_client(&self, _client: &'static dyn SpiSlaveClient) {
        unimplemented!();
    }

    fn set_polarity(&self, _polarity: ClockPolarity) -> Result<(), ErrorCode> {
        debug!("Crashing and burning...the sequel\n\n");
        let res: Result<(), ErrorCode> = Ok(());
        return res;
        //unimplemented!();
    }
    fn get_polarity(&self) -> hil::spi::ClockPolarity {
        unimplemented!();
    }
    fn set_phase(&self, _phase: ClockPhase) -> Result<(), ErrorCode> {
        unimplemented!();
    }
    fn get_phase(&self) -> hil::spi::ClockPhase {
        unimplemented!();
    }
}
