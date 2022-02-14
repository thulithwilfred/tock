//! Serial Peripheral Interface (SPI) Host Driver
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
    initialized: Cell<bool>,
    busy: Cell<bool>,
    cs_active_after: Cell<bool>,
    chip_select: Cell<u32>,
    cpu_clk: Cell<u32>,
    tsclk: Cell<u32>,
    tx_buf: TakeCell<'static, [u8]>,
    rx_buf: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    rx_len: Cell<usize>,
    tx_offset: Cell<usize>,
    rx_offset: Cell<usize>,
}
// SPI Host Command Direction: Bidirectional
const SPI_HOST_CMD_BIDIRECTIONAL: u32 = 3;
// SPI Host Command Speed: Standard SPI
const SPI_HOST_CMD_STANDARD_SPI: u32 = 0;

impl SpiHost {
    pub const fn new(base: StaticRef<SpiHostRegisters>, cpu_clk: u32) -> Self {
        SpiHost {
            registers: base,
            client: OptionalCell::empty(),
            initialized: Cell::new(false),
            busy: Cell::new(false),
            cs_active_after: Cell::new(false),
            chip_select: Cell::new(0),
            cpu_clk: Cell::new(cpu_clk),
            tsclk: Cell::new(0),
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
        self.disable_interrupts();
        if irq.is_set(intr::ERROR) {
            //TODO If any ret err
            debug!("[TOCK_ERR: Error Interrupt Set]");
            let err_status = regs.err_status.extract();
            is_test = true;
            if err_status.is_set(err_status::CMDBUSY) {
                is_test = false;
                regs.err_status.modify(err_status::CMDBUSY::CLEAR);
                debug!("TOCK_ERR: CMDBUSY")
            }
            if err_status.is_set(err_status::OVERFLOW) {
                is_test = false;
                regs.err_status.modify(err_status::OVERFLOW::CLEAR);
                //unimplemented!("OVERFLOW");
                debug!("TOCK_ERR: OFLOW")
            }
            if err_status.is_set(err_status::UNDERFLOW) {
                is_test = false;
                regs.err_status.modify(err_status::UNDERFLOW::CLEAR);
                //unimplemented!("UNDERFLOW");
                debug!("TOCK_ERR: UFLOW")
            }
            if err_status.is_set(err_status::CMDINVAL) {
                is_test = false;
                regs.err_status.modify(err_status::CMDINVAL::CLEAR);
                unimplemented!("COMMAND INVALID");
            }
            if err_status.is_set(err_status::CSIDINVAL) {
                is_test = false;
                regs.err_status.modify(err_status::CSIDINVAL::CLEAR);
                unimplemented!("CSID INVALID");
            }
            if err_status.is_set(err_status::ACCESSINVAL) {
                is_test = false;
                regs.err_status.modify(err_status::ACCESSINVAL::CLEAR);
                unimplemented!("ACCESSINVAL");
            }
            if is_test {
                self.clear_tests();
                debug!("TOCK_ERR: Test Error Interrupt");
            }
            //TODO Can clear everything here
            //Specified to be cleared, after err_status is cleared.
            self.clear_err_interrupt();
        }
        if irq.is_set(intr::SPI_EVENT) {
            debug!("[TOCK_EV: Event Interrupt Set]");
            let status = regs.status.extract();
            is_test = true;
            self.clear_event_interrupt();

            if status.is_set(status::RXFULL) {
                debug!("TOCK_EV: RXFULL");
                //is_test = false;
                //unimplemented!();
            }
            //This could be set at init, so only follow through
            //once a transfer has started (is_busy())
            if status.is_set(status::TXEMPTY) && self.is_busy() {
                debug!("TOCK_EV: TX Empty");
                is_test = false;
                //self.enable_tx_interrupt();
                self.continue_transfer();
                //self.show_debug();
            }

            if status.is_set(status::RXWM) {
                is_test = false;
                debug!("TOCK_EV: RXWM");
                //unimplemented!();
            }
            if status.is_set(status::TXWM) && self.is_busy() {
                is_test = false;      
                debug!("TOCK_EV: TXWM");         
                // self.enable_tx_interrupt();
                // self.continue_transfer();
                //unimplemented!();
            }
            if status.is_set(status::READY) {
                is_test = false;
                debug!("TOCK_EV: IRQ READY");
                //unimplemented!();
            }
            if status.is_set(status::ACTIVE) {
                is_test = false;
                debug!("TOCK_EV: ACTIVE");
                self.enable_interrupts();
                //unimplemented!();
            }
            if is_test {
                self.clear_tests();
                debug!("TOCK_EV: Test Event Interrupt");
            }
        }
        //self.enable_interrupts();
    }

    //Determine if transfer complete or if we need to keep
    //writing from an offset.
    fn continue_transfer(&self) {
        let regs = self.registers;
        let rx_buf = self.rx_buf.take().unwrap();
        let mut val32: u32;
        let mut val8: u8;
        let mut shift_mask;
        let rx_len = self.tx_offset.get() - self.rx_offset.get();
        let read_cycles = self.div_up(rx_len, 4);
        
        //Receive rx_data (Only 4byte reads are supported)
        for _n in 0..read_cycles {
            val32 = regs.rx_data.read(rx_data::DATA);
            shift_mask = 0xFF;
            for i in 0..4 {
                if self.rx_offset.get() >= self.rx_len.get() {
                    break;
                }
                val8 = ((val32 & shift_mask) >> i * 8) as u8;
                rx_buf[self.rx_offset.get()] = val8;
                self.rx_offset.set(self.rx_offset.get() + 1);
                shift_mask = shift_mask << 8;
            }
        }
        //Transfer was complete */
        if self.tx_offset.get() == self.tx_len.get() {
            self.client.map(|client| match self.tx_buf.take() {
                None => (),
                Some(tx_buf) => {
                    client.read_write_done(tx_buf, Some(rx_buf), self.tx_len.get(), Ok(()))
                }
            });
            debug!("TOCK: Transfer Complete");
            self.show_debug();
            self.disable_tx_interrupt();
            self.reset_internal_state();
        } else {
            debug!("TOCK: Continue Transfer");
            self.rx_buf.replace(rx_buf);
            //Theres more to transfer, continue writing from the offset
            self.spi_transfer_progress();
        }
    }

    /// Continue SPI transfer from offset point
    fn spi_transfer_progress(&self) {
        let tx_buf = self.tx_buf.take().unwrap();
        let tx_offset_start = self.tx_offset.get();
        let regs = self.registers;
        let mut t_byte: u32;
        let mut tx_slice: [u8; 4];
 
        assert_eq!(regs.status.read(status::TXQD), 0);
        assert_eq!(regs.status.read(status::ACTIVE), 0);
        
        let mut d_me = 0;
        while !regs.status.is_set(status::TXFULL) && regs.status.read(status::TXQD) < 64{
            
            tx_slice = [0, 0, 0, 0];
            for n in 0..4 {
                if self.tx_offset.get() >= self.tx_len.get() {
                    debug!("XXX: 1 {} {}", self.tx_offset.get() , tx_offset_start);
                    break;
                } 
                tx_slice[n] = tx_buf[self.tx_offset.get()];       
                self.tx_offset.set(self.tx_offset.get() + 1);
            }
            t_byte =  u32::from_le_bytes(tx_slice);
            regs.tx_data.write(tx_data::DATA.val(t_byte));
            d_me += 1;

            //Transfer Complete in one-shot
            if self.tx_offset.get() >= self.tx_len.get() {
                break;
            }

        }

        debug!("DEBUG: 2 {} {} {}\n", self.tx_offset.get(), tx_offset_start, d_me );
        //self.show_debug();

        //Hold tx_buf for offset transfer continue
        self.tx_buf.replace(tx_buf);

        //Set command register to init transfer
        self.start_transceive((self.tx_offset.get() - tx_offset_start) as u32);

        debug!(
            "TOCK: Transfer Continue OK {}",
            (self.tx_offset.get() - tx_offset_start)
        );
    }

    fn show_debug(&self) {
        let regs = self.registers;
        //debug!("[TOCK]: Start Trasnfer");
        //debug!("[TOCK]: SPE_EN {} OUT_EN {}", regs.ctrl.read(ctrl::SPIEN),  regs.ctrl.read(ctrl::OUTPUT_EN));
        debug!("--TOCK_EV: TWXM {}", regs.ctrl.read(ctrl::TX_WATERMARK));
        debug!("TOCK_EV: TXQD {}", regs.status.read(status::TXQD));
        //debug!("TOCK_EV: CMDQD {}", regs.status.read(status::CMDQD));
        debug!("TOCK_EV: TXSTALL {}", regs.status.read(status::TXSTALL));
        debug!("TOCK_EV: TXEMPTY {}", regs.status.read(status::TXEMPTY));
        debug!("TOCK_EV: ACTIVE {}", regs.status.read(status::ACTIVE));
        debug!("TOCK_EV: READY {}", regs.status.read(status::READY));
    }
    /// Issue a command to start SPI transaction
    /// Currently only Bi-Directional transactions are supported
    // TODO: Remove tx_len arg to this, we don't care
    fn start_transceive(&self, tx_len: u32) {
        let regs = self.registers;
        //8-bits that describe command transfer len (cannot exceed 255)
        let num_transfer_bytes: u32;
        //TXQD holds number of 32bit words
        let txfifo_num_bytes = regs.status.read(status::TXQD) * 4;

        if txfifo_num_bytes > u8::MAX as u32 {
            num_transfer_bytes = u8::MAX as u32 ;
        } else {
            num_transfer_bytes = txfifo_num_bytes;
        }

        debug!("YY TXQD {}", regs.status.read(status::TXQD));
        regs.command.write(
            command::LEN.val(num_transfer_bytes) +
            command::DIRECTION.val(SPI_HOST_CMD_BIDIRECTIONAL) +
            command::CSAAT::CLEAR + command::SPEED.val(SPI_HOST_CMD_STANDARD_SPI));

        self.enable_interrupts();
    }

    /// Reset the soft internal state, should be called once
    /// a spi transaction has been completed.
    fn reset_internal_state(&self) {
        self.clear_spi_busy();
        self.tx_len.set(0);
        self.rx_len.set(0);
        self.tx_offset.set(0);
        self.rx_offset.set(0);

        debug_assert!(self.tx_buf.is_none());
        debug_assert!(self.rx_buf.is_none());
    }

    /// Enable SPI_HOST IP
    fn enable_spi_host(&self) {
        let regs = self.registers;
        //Enables the SPI host
        regs.ctrl.modify(ctrl::SPIEN::SET + ctrl::OUTPUT_EN::SET);
    }

    /// Reset SPI Host
    #[allow(dead_code)]
    fn reset_spi_ip(&self) {
        let regs = self.registers;
        //IP to reset state
        regs.ctrl.modify(ctrl::SW_RST::SET);

        //Wait for status ready to be set before continuing
        while regs.status.is_set(status::ACTIVE) {}
        //Wait for both FIFOs to completely drain
        while regs.status.read(status::TXQD) != 0 && regs.status.read(status::RXQD) != 0 {}
        //Clear Reset
        regs.ctrl.modify(ctrl::SW_RST::CLEAR);
    }

    /// Enable both event/err IRQ
    fn enable_interrupts(&self) {
        let regs = self.registers;
        regs.intr_enable
            .modify(intr::ERROR::SET + intr::SPI_EVENT::SET);
    }

    /// Disable both event/err IRQ
    fn disable_interrupts(&self) {
        let regs = self.registers;
        regs.intr_enable
            .write(intr::ERROR::CLEAR + intr::SPI_EVENT::CLEAR);
    }

    /// Clear the error IRQ
    fn clear_err_interrupt(&self) {
        let regs = self.registers;
        regs.intr_state.modify(intr::ERROR::CLEAR);
    }

    /// Clear the event IRQ
    fn clear_event_interrupt(&self) {
        let regs = self.registers;
        regs.intr_state.modify(intr::SPI_EVENT::CLEAR);
    }
    /// Will generate a `test` interrupt on the error irq
    #[allow(dead_code)]
    fn test_error_interrupt(&self) {
        let regs = self.registers;
        regs.intr_test.write(intr::ERROR::SET);
    }
    /// Clear test interrupts
    fn clear_tests(&self) {
        let regs = self.registers;
        regs.intr_test
            .write(intr::ERROR::CLEAR + intr::SPI_EVENT::CLEAR);
    }

    /// Will generate a `test` interrupt on the event irq
    #[allow(dead_code)]
    fn test_event_interrupt(&self) {
        let regs = self.registers;
        regs.intr_test.write(intr::SPI_EVENT::SET);
    }

    /// Enable required `event interrupts`
    fn event_enable(&self) {
        let regs = self.registers;
        regs.event_en.write(event_en::TXEMPTY::SET);
    }


    fn disable_tx_interrupt(&self) {
        let regs = self.registers;
        regs.event_en.modify(event_en::TXEMPTY::CLEAR);
    }

    /// Enable the event interrupt and enable TXEMPTY
    /// TXEMPTY will call back once the TXFIFO has been drained
    fn enable_tx_interrupt(&self) {
        let regs = self.registers;
        regs.intr_enable.modify(intr::SPI_EVENT::SET);
        regs.event_en.modify(event_en::TXEMPTY::SET);
    }

    /// Enable required error interrupts
    fn err_enable(&self) {
        let regs = self.registers;
        regs.err_en
            .modify(err_en::CMDBUSY::SET + err_en::CMDINVAL::SET + err_en::CSIDINVAL::SET + err_en::OVERFLOW::SET + err_en::UNDERFLOW::SET);
    }

    fn set_spi_busy(&self) {
        self.busy.set(true);
    }

    fn clear_spi_busy(&self) {
        self.busy.set(false);
    }

    /// Divide a/b and return a value always rounded
    /// up to the nearest integer
    fn div_up(&self, a: usize, b: usize) -> usize {
        (a + (b - 1)) / b
    }

    /// Calculate the scaler based on a specified tsclk rate
    /// This scaler will pre-scale the cpu_clk and must be <= cpu_clk/2
    fn calculate_tsck_scaler(&self, rate: u32) -> Result<u16, ErrorCode> {
        if rate > self.cpu_clk.get() / 2 {
            return Err(ErrorCode::NOSUPPORT);
        }
        //Divide and truncate
        let mut scaler: u32 = (self.cpu_clk.get() / (2 * rate)) - 1;

        //Increase scaler if the division was not exact, ensuring that it does not overflow
        //or exceed divider specification where tsck is at most <= Tclk/2
        if self.cpu_clk.get() % (2 * rate) != 0 && scaler != 0xFF {
            scaler += 1;
        }
        Ok(scaler as u16)
    }
}

impl hil::spi::SpiMaster for SpiHost {
    type ChipSelect = u32;

    fn init(&self) -> Result<(), ErrorCode> {
        //debug!("SPI: Init");

        //self.reset_spi_ip();
        self.event_enable();
        self.err_enable();
        
        self.enable_interrupts();

        // self.test_error_interrupt();
        // self.test_event_interrupt();
        self.enable_spi_host();
        self.initialized.set(true);

        Ok(())
    }

    fn set_client(&self, client: &'static dyn hil::spi::SpiMasterClient) {
        //debug!("SPI: Set Client");
        self.client.set(client);
    }

    fn is_busy(&self) -> bool {
        //debug!("SPI: Is Busy");
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
        
        if self.is_busy() || regs.status.is_set(status::TXFULL) {
            return Err((ErrorCode::BUSY, tx_buf, rx_buf));
        }

        self.tx_len.set(cmp::min(len, tx_buf.len()));

        let mut t_byte: u32;
        let mut tx_slice: [u8; 4];
        //We are committing to the transfer now
        self.set_spi_busy();

        //TODO: I think this is bug in OT, where the `first` word written 
        // (while TXEMPTY) to TX_DATA is dropped/ignored and not added to TX_FIFO (TXQD = 0).
        // The following write (0x00), works around this `bug`. 
        // Could be Verilator specific
        regs.tx_data.write(tx_data::DATA.val(0x00));
        assert_eq!(regs.status.read(status::TXQD), 0);

        while !regs.status.is_set(status::TXFULL) && regs.status.read(status::TXQD) < 64{
            tx_slice = [0, 0, 0, 0];
            for n in 0..4 {
                if self.tx_offset.get() >= self.tx_len.get() {
                    break;
                }
                tx_slice[n] = tx_buf[self.tx_offset.get()];
                self.tx_offset.set(self.tx_offset.get() + 1);
            }         
            t_byte =  u32::from_le_bytes(tx_slice);
            regs.tx_data.write(tx_data::DATA.val(t_byte));

            //Transfer Complete in one-shot
            if self.tx_offset.get() >= self.tx_len.get() {
                break;
            }
        }

        //Hold tx_buf for offset transfer continue
        self.tx_buf.replace(tx_buf);

        //Hold rx_buf for later
        if rx_buf.is_some() {
            let rx_buf_t = rx_buf.unwrap();
            self.rx_len
                .set(cmp::min(self.tx_len.get() as usize, rx_buf_t.len()) as usize);
            self.rx_buf.replace(rx_buf_t);
        }

        //Set command register to init transfer
        self.start_transceive((self.tx_offset.get()) as u32);

        debug!("TOCK: R/W Bytes OK {}", self.tx_offset.get());
        Ok(())
    }

    fn write_byte(&self, _val: u8) -> Result<(), ErrorCode> {
        debug_assert!(self.initialized.get());
        //Use `read_write_bytes()` instead.
        Err(ErrorCode::NODEVICE)
    }

    fn read_byte(&self) -> Result<u8, ErrorCode> {
        debug_assert!(self.initialized.get());
        //Use `read_write_bytes()` instead.
        Err(ErrorCode::NODEVICE)
    }

    fn read_write_byte(&self, _val: u8) -> Result<u8, ErrorCode> {
        debug_assert!(self.initialized.get());
        //Use `read_write_bytes()` instead.
        Err(ErrorCode::NODEVICE)
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) -> Result<(), ErrorCode> {
        debug_assert!(self.initialized.get());
        //debug!("SPI: S CS {:?}", cs);
        let regs = self.registers;

        //CSID will index the CONFIGOPTS multi-register
        regs.csid.write(csid_ctrl::CSID.val(cs));
        self.chip_select.set(cs);

        Ok(())
    }

    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode> {
        debug_assert!(self.initialized.get());
        let regs = self.registers;

        match self.calculate_tsck_scaler(rate) {
            Ok(scaler) => {
                //debug!("SPI: CLSCL: {:?}", scaler);
                regs.config_opts
                    .modify(conf_opts::CLKDIV_0.val(scaler as u32));
                self.tsclk.set(rate);
                return Ok(rate);
            }
            Err(e) => return Err(e),
        }
    }

    fn get_rate(&self) -> u32 {
        debug_assert!(self.initialized.get());
        self.tsclk.get()
    }

    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode> {
        debug_assert!(self.initialized.get());
        //debug!("SPI: SET POL {:?}", polarity);
        let regs = self.registers;
        match polarity {
            ClockPolarity::IdleLow => regs.config_opts.modify(conf_opts::CPOL_0::CLEAR),
            ClockPolarity::IdleHigh => regs.config_opts.modify(conf_opts::CPOL_0::SET),
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
        //debug!("SPI: SET PHASE {:?}", phase);
        let regs = self.registers;
        match phase {
            ClockPhase::SampleLeading => regs.config_opts.modify(conf_opts::CPHA_0::CLEAR),
            ClockPhase::SampleTrailing => regs.config_opts.modify(conf_opts::CPHA_0::SET),
        };
        Ok(())
    }

    fn get_phase(&self) -> ClockPhase {
        debug_assert!(self.initialized.get());
        let regs = self.registers;

        match regs.config_opts.read(conf_opts::CPHA_0) {
            1 => return ClockPhase::SampleTrailing,
            0 => return ClockPhase::SampleLeading,
            _ => unreachable!(),
        };
    }

    fn hold_low(&self) {
        //This is taken into account when a command is issued during r/w bytes
        self.cs_active_after.set(true);
    }

    fn release_low(&self) {
        //This is taken into account when a command is issued during r/w bytes
        self.cs_active_after.set(false);
    }
}
