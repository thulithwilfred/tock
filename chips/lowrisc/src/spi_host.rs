//! Serial Peripheral Interface (SPI) Driver
use core::option::Option;
//use kernel::debug;
use kernel::hil;
use kernel::hil::spi::{ClockPhase, ClockPolarity};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

//TODO TEST: make ci-job-chips
register_structs! {
    pub SpiHostRegisters {
        //SPI: Interrupt State Register
        (0x000 => intr_state: ReadWrite<u32, intr::Register>),
        //SPI: Interrupt Enable Register
        (0x004 => intr_enable: ReadWrite<u32, intr::Register>),
        //SPI: Interrupt Test Register
        (0x008 => intr_test: WriteOnly<u32, intr::Register>),
        //SPI: Alert Test Register
        (0x00c => alert_test: WriteOnly<u32, alert_test::Register>),
        //SPI: Control register
        (0x010 => control: ReadWrite<u32, ctrl::Register>),
        //SPI: Status register
        (0x014 => status: ReadOnly<u32, status::Register>),
        //SPI: Configuration options register.
        (0x018 => config_opts: ReadWrite<u32, conf_opts::Register>),
        //SPI: Chip-Select ID
        (0x01c => csid: ReadWrite<u32, csid_ctrl::Register>),
        //SPI: Command Register
        (0x020 => command: WriteOnly<u32, command::Register>),
        //SPI: Received Data
        (0x024 => rx_data: ReadWrite<u32, rx_data::Register>),
        //SPI: Transmit Data
        (0x028 => tx_data: WriteOnly<u32, tx_data::Register>),
        //SPI: Controls which classes of errors raise an interrupt.
        (0x02c => err_en: ReadWrite<u32, err_en::Register>),
        //SPI: Indicates that any errors that have occurred
        (0x030 => err_status: ReadWrite<u32, err_status::Register>),
        //SPI: Controls which classes of SPI events raise an interrupt
        (0x034 => event_en: ReadWrite<u32, event_en::Register>),
        (0x38 => @END),
    }
}

register_bitfields![u32,
    intr [
        ERROR OFFSET(0) NUMBITS(1) [],
        SPI_EVENT OFFSET(1) NUMBITS(1) [],
    ],
    alert_test [
        FETAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    ctrl [
        RX_WATERMARK OFFSET(0) NUMBITS(8) [],
        TX_WATERMARK OFFSET(15) NUMBITS(8) [],
        //28:16 RESERVED
        OUTPUT_EN OFFSET(29) NUMBITS(1) [],
        SW_RST OFFSET(30) NUMBITS(1) [],
        SPIEN OFFSET(31) NUMBITS(1) []
    ],
    status [
        TXQD OFFSET(0) NUMBITS(8) [],
        RXQD OFFSET(15) NUMBITS(8) [],
        CMDQD OFFSET(16) NUMBITS(1) [],
        RXWM OFFSET(20) NUMBITS(1) [],
        BYTEORDER OFFSET(22) NUMBITS(1) [],
        RXSTALL OFFSET(23) NUMBITS(1) [],
        RXEMPTY OFFSET(24) NUMBITS(1) [],
        RXFULL OFFSET(25) NUMBITS(1) [],
        TXWM OFFSET(26) NUMBITS(1) [],
        TXSTALL OFFSET(27) NUMBITS(1) [],
        TXEMPTY OFFSET(28) NUMBITS(1) [],
        TXFULL OFFSET(29) NUMBITS(1) [],
        ACTIVE OFFSET(30) NUMBITS(1) [],
        READY OFFSET(31) NUMBITS(1) [],
    ],
    conf_opts [
        CLKDIV_0 OFFSET(0) NUMBITS(16) [],
        CSNIDLE_0 OFFSET(16) NUMBITS(3) [],
        CSNTRAIL_0 OFFSET(20) NUMBITS(3) [],
        CSNLEAD_0 OFFSET(24) NUMBITS(3) [],
        //28 Reserved
        FULLCYC_0 OFFSET(29) NUMBITS(1) [],
        CPHA_0 OFFSET(30) NUMBITS(1) [],
        CPOL_0 OFFSET(31) NUMBITS(1) [],
    ],
    csid_ctrl [
        CSID OFFSET(0) NUMBITS(32) [],
    ],
    command [
        LEN OFFSET(0) NUMBITS(8) [],
        CSAAT OFFSET(9) NUMBITS(1) [],
        SPEED OFFSET(10) NUMBITS(2) [],
        DIRECTION OFFSET(12) NUMBITS(2) [],
    ],
    rx_data [
        DATA OFFSET(0) NUMBITS(32) [],
    ],
    tx_data [
        DATA OFFSET(0) NUMBITS(32) [],
    ],
    err_en [
        CMDBUSY OFFSET(0) NUMBITS(1) [],
        OVERFLOW OFFSET(1) NUMBITS(1) [],
        UNDERFLOW OFFSET(2) NUMBITS(1) [],
        CMDINVAL OFFSET(3) NUMBITS(1) [],
        CSIDINVAL OFFSET(4) NUMBITS(1) [],
    ],
    err_status [
        CMDBUSY OFFSET(0) NUMBITS(1) [],
        OVERFLOW OFFSET(1) NUMBITS(1) [],
        UNDERFLOW OFFSET(2) NUMBITS(1) [],
        CMDINVAL OFFSET(3) NUMBITS(1) [],
        CSIDINVAL OFFSET(4) NUMBITS(1) [],
        ACCESSINVAL OFFSET(5) NUMBITS(1) [],
    ],
    event_en [
        RXFULL OFFSET(0) NUMBITS(1) [],
        TXEMPTY OFFSET(1) NUMBITS(1) [],
        RXWM OFFSET(2) NUMBITS(1) [],
        TXWM OFFSET(3) NUMBITS(1) [],
        READY OFFSET(4) NUMBITS(1) [],
        IDLE OFFSET(5) NUMBITS(1) [],
    ],
];

pub struct SpiHost {
    _registers: StaticRef<SpiHostRegisters>,
    _client: OptionalCell<&'static dyn hil::spi::SpiSlaveClient>,
}

impl SpiHost {
    pub const fn new(base: StaticRef<SpiHostRegisters>) -> Self {
        SpiHost {
            _registers: base,
            _client: OptionalCell::empty(),
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
