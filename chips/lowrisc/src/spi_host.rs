//! Serial Peripheral Interface (SPI) Driver
use core::cell::Cell;
use core::cmp;
use core::option::Option;
use kernel::debug;
use kernel::hil;
use kernel::hil::spi::SpiMaster;
use kernel::hil::spi::{ClockPhase, ClockPolarity};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
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
        (0x010 => ctrl: ReadWrite<u32, ctrl::Register>),
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
        TX_WATERMARK OFFSET(8) NUMBITS(8) [],
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
    registers: StaticRef<SpiHostRegisters>,
    client: OptionalCell<&'static dyn hil::spi::SpiMasterClient>,
    chip_select: OptionalCell<&'static dyn hil::gpio::Pin>,
    initialized: Cell<bool>,
    busy: Cell<bool>,
    tx_buf: TakeCell<'static, [u8]>,
    rx_buf: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    rx_len: Cell<usize>,
    tx_offset: Cell<usize>,
    rx_offset: Cell<usize>,
}

impl SpiHost {
    pub const fn new(base: StaticRef<SpiHostRegisters>) -> Self {
        SpiHost {
            registers: base,
            client: OptionalCell::empty(),
            chip_select: OptionalCell::empty(),
            initialized: Cell::new(false),
            busy: Cell::new(false),
            tx_buf: TakeCell::empty(),
            rx_buf: TakeCell::empty(),
            tx_len: Cell::new(0),
            rx_len: Cell::new(0),
            tx_offset: Cell::new(0),
            rx_offset: Cell::new(0),
        }
    }

    #[inline(never)]
    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let irq = regs.intr_state.extract();
        let mut is_test;

        if irq.is_set(intr::ERROR) {
            debug!("[TOCK_ERR: Error Interrupt Set]");
            let err_status = regs.err_status.extract();
            is_test = true;
            self.clear_err_interrupt();
            if err_status.is_set(err_status::CMDBUSY) {
                is_test = false;
                debug!("TOCK_ERR: CMDBUSY")
                //unimplemented!();
            }
            if err_status.is_set(err_status::OVERFLOW) {
                is_test = false;
                unimplemented!();
            }
            if err_status.is_set(err_status::UNDERFLOW) {
                is_test = false;
                unimplemented!();
            }
            if err_status.is_set(err_status::CMDINVAL) {
                is_test = false;
                unimplemented!();
            }
            if err_status.is_set(err_status::CSIDINVAL) {
                is_test = false;
                unimplemented!();
            }
            if err_status.is_set(err_status::ACCESSINVAL) {
                is_test = false;
                unimplemented!();
            }
            if is_test {
                self.clear_tests();
                debug!("TOCK_ERR: Test Error Interrupt");
            }
        }
        if irq.is_set(intr::SPI_EVENT) {
            debug!("[TOCK_EV: Event Interrupt Set]");
            self.clear_event_interrupt();
            let status = regs.status.extract();
            is_test = true;

            if status.is_set(status::RXFULL) {
                is_test = false;
                unimplemented!();
            }
            //This could be set at init, so only follow through
            //once a transfer has started (is_busy())
            if status.is_set(status::TXEMPTY) && self.is_busy() {
                debug!("TOCK_EV: IRQ TX Empty");
                is_test = false;
                self.continue_transfer();
            }
            if status.is_set(status::RXWM) {
                is_test = false;
                unimplemented!();
            }
            if status.is_set(status::TXWM) {
                is_test = false;
                unimplemented!();
            }
            if status.is_set(status::READY) {
                is_test = false;
                debug!("TOCK_EV: IRQ READY");
                //unimplemented!();
            }
            if status.is_set(status::ACTIVE) {
                is_test = false;
                unimplemented!();
            }
            if is_test {
                self.clear_tests();
                debug!("TOCK_EV: Test Event Interrupt");
            }
        }
    }

    //Determine if transfer complete or if we need to keep
    //writing from an offset.
    fn continue_transfer(&self) {
        let regs = self.registers;
        let rx_buf = self.rx_buf.take().unwrap();
        let mut val32: u32;
        let mut val8: u8;
        let mut shift_mask;
        let read_cycles = self.div_up(self.rx_len.get() - self.rx_offset.get(), 4);
        //Receive rx_data
        for _n in 0..read_cycles {
            val32 = regs.rx_data.read(rx_data::DATA);
            shift_mask = 0xFF;
            for i in 0..4 {
                //TODO: Code here can be optimized
                val8 = ((val32 & shift_mask) >> i * 8) as u8;
                debug!("READ {}", val8);
                rx_buf[self.rx_offset.get()] = val8;
                self.rx_offset.set(self.rx_offset.get() + 1);
                shift_mask = shift_mask << 8;
            }
        }
        //Transfer was complete */
        if self.tx_offset.get() == self.tx_len.get() {
            self.client.take().unwrap().read_write_done(
                self.tx_buf.take().unwrap(),
                Some(rx_buf),
                self.tx_len.get(),
                Ok(()),
            );
            debug!("TOCK: Transfer Complete");
            self.reset_internal_state();
        } else {
            debug!("TOCK: Continue Transfer");
            //self.rx_buf.replace(rx_buf);
            //Theres more to transfer, continue writing from the offset
            //self.spi_transfer_progress();
        }
    }

    fn spi_transfer_progress(&self) {
        let tx_len = self.tx_len.get();
        let tx_buf = self.tx_buf.take().unwrap();
        let mut tx_offset = self.tx_offset.get();
        let tx_offset_start = tx_offset;
        let regs = self.registers;
        let mut val: u32 = 0;
        let mut write_cmplt: bool = false;

        while !regs.status.is_set(status::TXFULL) {
            for i in 0..4 {
                //Shift 4 bytes from tx_buf to fifo tx access reg value
                val = val | ((u32::from(tx_buf[tx_offset as usize]) << i * 8) as u32);
                tx_offset += 1;

                if tx_offset >= tx_len {
                    write_cmplt = true;
                    break;
                }
            }
            //Transfer to FIFO access registers
            regs.tx_data.write(tx_data::DATA.val(val));
            val = 0;
            //We have written all of TX buffer, but TXFIFO is not full
            if write_cmplt {
                break;
            }
        }
        //Hold tx_buf for offset transfer continue
        self.tx_buf.replace(tx_buf);
        self.tx_offset.set(tx_offset as usize);
        //Wait for status ready to be set before continuing
        while !regs.status.is_set(status::READY) {}
        //Set command register to init transfer
        self.start_transceive((tx_offset - tx_offset_start) as u32);
        debug!("TOCK: Transfer Continue OK {}", tx_offset - tx_offset_start);
    }

    fn min_of_three(&self, i: usize, j: usize, k: usize) -> usize {
        cmp::min(i, cmp::min(j, k))
    }

    fn start_transceive(&self, tx_len: u32) {
        let regs = self.registers;
        //Direction (3) -> Bidirectional TX/RX
        regs.command
            .write(command::LEN.val(tx_len) + command::DIRECTION.val(3));
    }

    fn reset_internal_state(&self) {
        self.clear_spi_busy();
        self.tx_len.set(0);
        self.rx_len.set(0);
        self.tx_offset.set(0);
        self.rx_offset.set(0);

        debug_assert!(self.tx_buf.is_none());
        debug_assert!(self.rx_buf.is_none());
    }

    fn enable_spi_host(&self) {
        let regs = self.registers;
        //Enables the SPI host
        regs.ctrl.modify(ctrl::SPIEN::SET);
    }

    fn reset_spi_ip(&self) {
        let regs = self.registers;
        //IP to reset state
        regs.ctrl.modify(ctrl::SW_RST::SET);
    }

    #[allow(dead_code)]
    fn enable_err_interrupt(&self) {
        let regs = self.registers;
        regs.intr_enable.modify(intr::ERROR::SET);
    }

    #[allow(dead_code)]
    fn enable_event_interrupt(&self) {
        let regs = self.registers;
        regs.intr_enable.modify(intr::SPI_EVENT::SET);
    }

    #[allow(dead_code)]
    fn disable_err_interrupt(&self) {
        let regs = self.registers;
        regs.intr_enable.write(intr::ERROR::CLEAR);
    }
    #[allow(dead_code)]
    fn disable_event_interrupt(&self) {
        let regs = self.registers;
        regs.intr_enable.modify(intr::SPI_EVENT::CLEAR);
    }

    fn enable_interrupts(&self) {
        let regs = self.registers;
        regs.intr_enable
            .modify(intr::ERROR::SET + intr::SPI_EVENT::SET);
    }
    #[allow(dead_code)]
    fn disable_interrupts(&self) {
        let regs = self.registers;
        regs.intr_enable
            .modify(intr::ERROR::CLEAR + intr::SPI_EVENT::CLEAR);
    }

    fn clear_err_interrupt(&self) {
        let regs = self.registers;
        regs.intr_state.modify(intr::ERROR::CLEAR);
    }

    fn clear_event_interrupt(&self) {
        let regs = self.registers;
        regs.intr_state.modify(intr::SPI_EVENT::CLEAR);
    }

    fn test_error_interrupt(&self) {
        let regs = self.registers;
        regs.intr_test.write(intr::ERROR::SET);
    }

    fn clear_tests(&self) {
        let regs = self.registers;
        regs.intr_test
            .write(intr::ERROR::CLEAR + intr::SPI_EVENT::CLEAR);
    }

    fn test_event_interrupt(&self) {
        let regs = self.registers;
        regs.intr_test.write(intr::SPI_EVENT::SET);
    }

    fn event_enable(&self) {
        let regs = self.registers;
        //regs.event_en.modify(event_en::READY::SET);
        regs.event_en.modify(event_en::TXEMPTY::SET);
    }

    fn err_enable(&self) {
        let regs = self.registers;
        regs.err_en.modify(err_en::CMDBUSY::SET);
    }

    fn set_spi_busy(&self) {
        self.busy.set(true);
    }

    fn clear_spi_busy(&self) {
        self.busy.set(false);
    }

    //Divide a/b and return a value rounded up
    fn div_up(&self, a: usize, b: usize) -> usize {
        (a + (b - 1)) / b
    }
}

impl hil::spi::SpiMaster for SpiHost {
    type ChipSelect = &'static dyn hil::gpio::Pin;

    fn init(&self) -> Result<(), ErrorCode> {
        debug!("SPI: Init");
        let regs = self.registers;

        self.reset_spi_ip();
        self.enable_spi_host();
        self.event_enable();
        self.err_enable();
        self.initialized.set(true);

        self.enable_interrupts();
        //self.test_error_interrupt();
        //self.test_event_interrupt();
        Ok(())
    }

    fn set_client(&self, client: &'static dyn hil::spi::SpiMasterClient) {
        debug!("SPI: Set Client");
        self.client.set(client);
    }

    fn is_busy(&self) -> bool {
        debug!("SPI: Is Busy");
        self.busy.get()
    }

    fn read_write_bytes(
        &self,
        tx_buf: &'static mut [u8],
        rx_buf: Option<&'static mut [u8]>,
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8], Option<&'static mut [u8]>)> {
        debug!("SPI: R/W Bytes");
        debug_assert!(self.initialized.get());
        debug_assert!(!self.busy.get());
        debug_assert!(self.tx_buf.is_none());
        debug_assert!(self.rx_buf.is_none());
        let regs = self.registers;

        // Clear (set to low) chip-select
        if self.chip_select.is_none() {
            return Err((ErrorCode::NODEVICE, tx_buf, rx_buf));
        }

        if self.is_busy() || regs.status.is_set(status::TXFULL) {
            return Err((ErrorCode::BUSY, tx_buf, rx_buf));
        }

        // Call is ignored, if the pin is not I/O
        self.chip_select.map(|cs| cs.clear());

        let tx_len: u16 = cmp::min(len, tx_buf.len()) as u16;
        self.tx_len.set(tx_len as usize);
        //RX Len is updated at the end based on rx_buf.len()
        self.rx_len.set(tx_len as usize);
        let mut tx_offset: u16 = 0;
        let mut val: u32 = 0;
        let mut write_cmplt: bool = false;

        //We are committing to the transfer now
        self.set_spi_busy();

        //We push to TXFIFO until notified that it's full
        while !regs.status.is_set(status::TXFULL) {
            for i in 0..4 {
                //Shift 4 bytes from tx_buf to fifo tx access reg value
                val = val | ((u32::from(tx_buf[tx_offset as usize]) << i * 8) as u32);
                tx_offset += 1;

                if tx_offset >= tx_len {
                    write_cmplt = true;
                    break;
                }
            }
            //Transfer to FIFO access registers
            regs.tx_data.write(tx_data::DATA.val(val));
            val = 0;
            //We have written all of TX buffer, but TXFIFO is not full
            if write_cmplt {
                break;
            }
        }

        //Hold rx_buf for reading when tx_cmplt
        if rx_buf.is_some() {
            let rx_buf_t = rx_buf.unwrap();
            self.rx_len
                .set(cmp::min(tx_len as usize, rx_buf_t.len()) as usize);
            //Can only receive as buffer allows or transfer len
            // self.rx_len
            //     .set(self.min_of_three(len, tx_offset as usize - 4, rx_buf_t.len()));
            self.rx_buf.replace(rx_buf_t);
        }
        //Hold tx_buf for offset transfer continue
        self.tx_buf.replace(tx_buf);
        self.tx_offset.set(tx_offset as usize);

        //Wait for status ready to be set before continuing
        while !regs.status.is_set(status::READY) {}

        //Set command register to init transfer
        self.start_transceive(self.tx_offset.get() as u32);
        debug!("TOCK: R/W Bytes OK");
        Ok(())
    }

    fn write_byte(&self, _val: u8) -> Result<(), ErrorCode> {
        debug_assert!(self.initialized.get());
        //Use `read_write_bytes()` instead.
        return Err(ErrorCode::NODEVICE);
    }

    fn read_byte(&self) -> Result<u8, ErrorCode> {
        debug_assert!(self.initialized.get());
        //Use `read_write_bytes()` instead.
        return Err(ErrorCode::NODEVICE);
    }

    fn read_write_byte(&self, val: u8) -> Result<u8, ErrorCode> {
        debug_assert!(self.initialized.get());
        //Use `read_write_bytes()` instead.
        return Err(ErrorCode::NODEVICE);
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) -> Result<(), ErrorCode> {
        debug_assert!(self.initialized.get());
        cs.make_output();
        cs.set();
        self.chip_select.set(cs);
        Ok(())
    }

    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode> {
        //unimplemented!("SPI: Set Rate {:?}", _rate);
        debug_assert!(self.initialized.get());
        Ok(rate)
    }

    fn get_rate(&self) -> u32 {
        debug_assert!(self.initialized.get());
        let _rc = 0;
        unimplemented!("SPI: Get Rate");
    }

    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode> {
        debug_assert!(self.initialized.get());
        let regs = self.registers;

        match polarity {
            ClockPolarity::IdleLow => regs.config_opts.write(conf_opts::CPOL_0::CLEAR),
            ClockPolarity::IdleHigh => regs.config_opts.write(conf_opts::CPOL_0::SET),
        };
        Ok(())
    }

    fn get_polarity(&self) -> ClockPolarity {
        debug_assert!(self.initialized.get());
        let regs = self.registers;

        match regs.config_opts.read(conf_opts::CPOL_0) {
            0 => ClockPolarity::IdleLow,
            1 => ClockPolarity::IdleHigh,
            _ => unreachable!(),
        }
    }

    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode> {
        debug_assert!(self.initialized.get());
        let regs = self.registers;

        match phase {
            ClockPhase::SampleLeading => regs.config_opts.write(conf_opts::CPHA_0::CLEAR),
            ClockPhase::SampleTrailing => regs.config_opts.write(conf_opts::CPHA_0::SET),
        };
        Ok(())
    }

    fn get_phase(&self) -> ClockPhase {
        debug_assert!(self.initialized.get());
        let regs = self.registers;

        match regs.config_opts.read(conf_opts::CPHA_0) {
            1 => ClockPhase::SampleTrailing,
            0 => ClockPhase::SampleTrailing,
            _ => unreachable!(),
        };
        unimplemented!("SPI: Get Phase");
    }

    fn hold_low(&self) {
        unimplemented!("SPI: Hold Low");
    }

    fn release_low(&self) {
        unimplemented!("SPI: Hold Low");
    }
}
