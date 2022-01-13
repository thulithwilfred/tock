//! Serial Peripheral Interface (SPI) Driver
use core::option::Option;
use kernel::debug;
use kernel::hil;
use kernel::hil::spi::{ClockPhase, ClockPolarity};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

//TODO SPI: Update reg
//TODO Add resreves for gaps
//TODO TEST: make ci-job-chips
register_structs! {
    pub SpiHostRegisters {
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

//TODO SPI: Update these to match
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

pub struct SpiHost {
    _registers: StaticRef<SpiHostRegisters>,

    client: OptionalCell<&'static dyn hil::spi::SpiSlaveClient>,
}

impl SpiHost {
    pub const fn new(base: StaticRef<SpiHostRegisters>) -> Self {
        SpiHost {
            _registers: base,
            client: OptionalCell::empty(),
        }
    }
    //TODO CB: handle spi interrupts
    pub fn handle_interrupt(&self) {
        unimplemented!();
    }
}

//TODO: SpiMaster (top) hil for Spihost
impl hil::spi::SpiMaster for SpiHost {
    type ChipSelect = u32;

    fn init(&self) -> Result<(), ErrorCode> {
        unimplemented!();
    }

    fn set_client(&self, _client: &'static dyn hil::spi::SpiMasterClient) {
        unimplemented!();
    }

    fn is_busy(&self) -> bool {
        unimplemented!();
    }

    fn read_write_bytes(
        &self,
        _write_buffer: &'static mut [u8],
        _read_buffer: Option<&'static mut [u8]>,
        _len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8], Option<&'static mut [u8]>)> {
        unimplemented!();
    }

    fn write_byte(&self, _val: u8) -> Result<(), ErrorCode> {
        unimplemented!();
    }

    fn read_byte(&self) -> Result<u8, ErrorCode> {
        unimplemented!();
    }

    fn read_write_byte(&self, _val: u8) -> Result<u8, ErrorCode> {
        unimplemented!();
    }

    fn specify_chip_select(&self, _cs: Self::ChipSelect) -> Result<(), ErrorCode> {
        unimplemented!();
    }

    fn set_rate(&self, _rate: u32) -> Result<u32, ErrorCode> {
        unimplemented!();
    }

    fn get_rate(&self) -> u32 {
        unimplemented!();
    }

    fn set_polarity(&self, _polarity: ClockPolarity) -> Result<(), ErrorCode> {
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
    fn hold_low(&self) {
        unimplemented!();
    }
    fn release_low(&self) {
        unimplemented!();
    }
}

impl hil::spi::SpiMasterDevice for SpiHost {
    fn set_client(&self, _client: &'static dyn hil::spi::SpiMasterClient) {
        unimplemented!();
    }

    fn configure(
        &self,
        _cpol: ClockPolarity,
        _cpal: ClockPhase,
        _rate: u32,
    ) -> Result<(), ErrorCode> {
        unimplemented!();
    }

    fn read_write_bytes(
        &self,
        _write_buffer: &'static mut [u8],
        _read_buffer: Option<&'static mut [u8]>,
        _len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8], Option<&'static mut [u8]>)> {
        unimplemented!();
    }

    fn set_rate(&self, _rate: u32) -> Result<(), ErrorCode> {
        unimplemented!();
    }

    fn get_rate(&self) -> u32 {
        unimplemented!();
    }

    fn set_polarity(&self, _polarity: ClockPolarity) -> Result<(), ErrorCode> {
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
