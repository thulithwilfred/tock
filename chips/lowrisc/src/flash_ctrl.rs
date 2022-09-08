//! Flash Controller

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};

use kernel::hil;
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

register_structs! {
    pub FlashCtrlRegisters {
        (0x000 => intr_state: ReadWrite<u32, INTR::Register>),
        (0x004 => intr_enable: ReadWrite<u32, INTR::Register>),
        (0x008 => intr_test: WriteOnly<u32, INTR::Register>),
        (0x00C => alert_test: WriteOnly<u32>),
        (0x010 => disable: ReadWrite<u32>),
        (0x014 => exec: ReadWrite<u32>),
        (0x018 => init: ReadWrite<u32, INIT::Register>),
        (0x01C => ctrl_regwen: ReadOnly<u32, CTRL_REGWEN::Register>),
        (0x020 => control: ReadWrite<u32, CONTROL::Register>),
        (0x024 => addr: ReadWrite<u32, ADDR::Register>),
        (0x028 => prog_type_en: ReadWrite<u32, PROG_TYPE_EN::Register>),
        (0x02c => erase_suspend: ReadWrite<u32, ERASE_SUSPEND::Register>),
        (0x030 => region_cfg_regwen: [ReadWrite<u32, REGION_CFG_REGWEN::Register>; 8]),
        (0x050 => mp_region_cfg: [ReadWrite<u32, MP_REGION_CFG::Register>; 8]),
        (0x070 => mp_region: [ReadWrite<u32, MP_REGION::Register>; 8]),
        (0x090 => default_region: ReadWrite<u32, DEFAULT_REGION::Register>),

        (0x094 => bank0_info0_regwen: [ReadWrite<u32, BANK_INFO_REGWEN::Register>; 10]),
        (0x0BC => bank0_info0_page_cfg: [ReadWrite<u32, BANK_INFO_PAGE_CFG::Register>; 10]),
        (0x0E4 => bank0_info1_regwen: ReadWrite<u32, BANK_INFO_REGWEN::Register>),
        (0x0E8 => bank0_info1_page_cfg: ReadWrite<u32, BANK_INFO_PAGE_CFG::Register>),
        (0x0EC => bank0_info2_regwen: [ReadWrite<u32, BANK_INFO_REGWEN::Register>; 2]),
        (0x0F4 => bank0_info2_page_cfg: [ReadWrite<u32, BANK_INFO_PAGE_CFG::Register>; 2]),

        (0x0FC => bank1_info0_regwen: [ReadWrite<u32, BANK_INFO_REGWEN::Register>; 10]),
        (0x124 => bank1_info0_page_cfg: [ReadWrite<u32, BANK_INFO_PAGE_CFG::Register>; 10]),
        (0x14C => bank1_info1_regwen: ReadWrite<u32, BANK_INFO_REGWEN::Register>),
        (0x150 => bank1_info1_page_cfg: ReadWrite<u32, BANK_INFO_PAGE_CFG::Register>),
        (0x154 => bank1_info2_regwen: [ReadWrite<u32, BANK_INFO_REGWEN::Register>; 2]),
        (0x15C => bank1_info2_page_cfg: [ReadWrite<u32, BANK_INFO_PAGE_CFG::Register>; 2]),

        (0x164 => bank_cfg_regwen: ReadWrite<u32, BANK_CFG_REGWEN::Register>),
        (0x168 => mp_bank_cfg_shadowed: ReadWrite<u32, MP_BANK_CFG::Register>),
        (0x16C => op_status: ReadWrite<u32, OP_STATUS::Register>),
        (0x170 => status: ReadOnly<u32, STATUS::Register>),
        (0x174 => err_code: ReadOnly<u32>),
        (0x178 => std_fault_status: ReadOnly<u32>),
        (0x17C => fault_status: ReadOnly<u32>),
        (0x180 => err_addr: ReadOnly<u32>),
        (0x184 => ecc_single_err_cnt: ReadOnly<u32>),
        (0x188 => ecc_single_addr: [ReadOnly<u32>; 2]),
        (0x190 => phy_alert_cfg: ReadOnly<u32>),
        (0x194 => phy_status: ReadOnly<u32, PHY_STATUS::Register>),
        (0x198 => scratch: ReadWrite<u32, SCRATCH::Register>),
        (0x19C => fifo_lvl: ReadWrite<u32, FIFO_LVL::Register>),
        (0x1A0 => fifo_rst: ReadWrite<u32, FIFO_RST::Register>),
        (0x1A4 => curr_fifo_lvl: WriteOnly<u32>),
        (0x1A8 => prog_fifo: WriteOnly<u32>),
        (0x1AC => rd_fifo: ReadOnly<u32>),
        (0x1B0=> @END),
    }
}

register_bitfields![u32,
    INTR [
        PROG_EMPTY OFFSET(0) NUMBITS(1) [],
        PROG_LVL OFFSET(1) NUMBITS(1) [],
        RD_FULL OFFSET(2) NUMBITS(1) [],
        RD_LVL OFFSET(3) NUMBITS(1) [],
        OP_DONE OFFSET(4) NUMBITS(1) [],
        OP_ERROR OFFSET(5) NUMBITS(1) []
    ],
    INIT [
        VAL OFFSET(0) NUMBITS(1) []
    ],
    CTRL_REGWEN [
        EN OFFSET(0) NUMBITS(1) []
    ],
    CONTROL [
        START OFFSET(0) NUMBITS(1) [],
        OP OFFSET(4) NUMBITS(2) [
            READ = 0,
            PROG = 1,
            ERASE = 2
        ],
        PROG_SEL OFFSET(6) NUMBITS(1) [
            NORMAL = 0,
            REPAIR = 1,
        ],
        ERASE_SEL OFFSET(7) NUMBITS(1) [
            PAGE = 0,
            BANK = 1
        ],
        PARTITION_SEL OFFSET(8) NUMBITS(1) [
            // data partition - this is the portion of flash that is
            //     accessible both by the host and by the controller.
            DATA = 0,
            // info partition - this is the portion of flash that is
            //     only accessible by the controller.
            INFO = 1
        ],
        INFO_SEL OFFSET(9) NUMBITS(2) [],
        NUM OFFSET(16) NUMBITS(12) []
    ],
    PROG_TYPE_EN [
        NORMAL OFFSET(0) NUMBITS(1) [],
        REPAIR OFFSET(1) NUMBITS(1) [],
    ],
    ERASE_SUSPEND [
        REQ OFFSET(0) NUMBITS(1) [],
    ],
    ADDR [
        START OFFSET(0) NUMBITS(32) []
    ],
    REGION_CFG_REGWEN [
        REGION OFFSET(0) NUMBITS(1) []
    ],
    MP_REGION_CFG [
        // These config register fields require a special value of
        // 0x6 (0110) to set, or 0x9 (1001) to reset
        EN OFFSET(0) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        RD_EN OFFSET(4) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        PROG_EN OFFSET(8) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        ERASE_EN OFFSET(12) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        SCRAMBLE_EN OFFSET(16) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        ECC_EN OFFSET(20) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        HE_EN OFFSET(24) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
    ],
    MP_REGION [
        BASE OFFSET(0) NUMBITS(9) [],
        SIZE OFFSET(9) NUMBITS(9) []
    ],
    BANK_INFO_REGWEN [
        REGION OFFSET(0) NUMBITS(1) [
            Locked = 0,
            Enabled =1,
        ]
    ],
    BANK_INFO_PAGE_CFG [
        EN OFFSET(0) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        RD_EN OFFSET(4) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        PROG_EN OFFSET(8) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        ERASE_EN OFFSET(12) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        SCRAMBLE_EN OFFSET(16) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        ECC_EN OFFSET(20) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        HE_EN OFFSET(24) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
    ],
    BANK_CFG_REGWEN [
        BANK OFFSET(0) NUMBITS(1) []
    ],
    DEFAULT_REGION [
        RD_EN OFFSET(0) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        PROG_EN OFFSET(4) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        ERASE_EN OFFSET(8) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        SCRAMBLE_EN OFFSET(12) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        ECC_EN OFFSET(16) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        HE_EN OFFSET(20) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
    ],
    MP_BANK_CFG [
        ERASE_EN_0 OFFSET(0) NUMBITS(1) [],
        ERASE_EN_1 OFFSET(1) NUMBITS(1) []
    ],
    OP_STATUS [
        DONE OFFSET(0) NUMBITS(1) [],
        ERR OFFSET(1) NUMBITS(1) []
    ],
    STATUS [
        RD_FULL OFFSET(0) NUMBITS(1) [],
        RD_EMPTY OFFSET(1) NUMBITS(1) [],
        PROG_FULL OFFSET(2) NUMBITS(1) [],
        PROG_EMPTY OFFSET(3) NUMBITS(1) [],
        INIT_WIP OFFSET(4) NUMBITS(1) [],
    ],
    PHY_STATUS [
        INIT_WIP OFFSET(0) NUMBITS(1) [],
        PROG_NORMAL_AVAIL OFFSET(1) NUMBITS(1) [],
        PROG_REPAIR_AVAIL OFFSET(2) NUMBITS(1) []
    ],
    SCRATCH [
        DATA OFFSET(0) NUMBITS(32) []
    ],
    FIFO_LVL [
        PROG OFFSET(0) NUMBITS(5) [],
        RD OFFSET(8) NUMBITS(5) []
    ],
    FIFO_RST [
        EN OFFSET(0) NUMBITS(1) []
    ]
];

pub const PAGE_SIZE: usize = 2048;
pub const FLASH_WORD_SIZE: usize = 8;
pub const FLASH_PAGES_PER_BANK: usize = 256;
// The programming windows size in words (32bit)
pub const FLASH_PROG_WINDOW_SIZE: usize = 16;
pub const FLASH_PROG_WINDOW_MASK: u32 = 0xFFFFFFF0;

pub struct LowRiscPage(pub [u8; PAGE_SIZE as usize]);

impl Default for LowRiscPage {
    fn default() -> Self {
        Self {
            0: [0; PAGE_SIZE as usize],
        }
    }
}

impl Index<usize> for LowRiscPage {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for LowRiscPage {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl AsMut<[u8]> for LowRiscPage {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

#[derive(PartialEq)]
enum FlashBank {
    BANK0 = 0,
    BANK1 = 1,
}

#[derive(PartialEq, Clone, Copy)]
pub enum FlashRegion {
    REGION0 = 0,
    REGION1 = 1,
    REGION2 = 2,
    REGION3 = 3,
    REGION4 = 4,
    REGION5 = 5,
    REGION6 = 6,
    REGION7 = 7,
}

pub struct FlashCtrl<'a> {
    registers: StaticRef<FlashCtrlRegisters>,
    flash_client: OptionalCell<&'a dyn hil::flash::Client<FlashCtrl<'a>>>,
    data_configured: Cell<bool>,
    info_configured: Cell<bool>,
    read_buf: TakeCell<'static, LowRiscPage>,
    read_index: Cell<usize>,
    write_buf: TakeCell<'static, LowRiscPage>,
    write_index: Cell<usize>,
    write_word_addr: Cell<usize>,
    region_num: FlashRegion,
}

impl<'a> FlashCtrl<'a> {
    pub fn new(base: StaticRef<FlashCtrlRegisters>, region_num: FlashRegion) -> Self {
        FlashCtrl {
            registers: base,
            flash_client: OptionalCell::empty(),
            data_configured: Cell::new(false),
            info_configured: Cell::new(false),
            read_buf: TakeCell::empty(),
            read_index: Cell::new(0),
            write_buf: TakeCell::empty(),
            write_index: Cell::new(0),
            write_word_addr: Cell::new(0),
            region_num,
        }
    }

    fn enable_interrupts(&self) {
        // Enable relevent interrupts
        self.registers.intr_enable.write(
            INTR::PROG_EMPTY::SET
                + INTR::PROG_LVL::CLEAR
                + INTR::RD_FULL::CLEAR
                + INTR::RD_LVL::SET
                + INTR::OP_DONE::SET
                + INTR::OP_ERROR::SET,
        );
    }

    fn disable_interrupts(&self) {
        // Disable and clear all interrupts
        self.registers.intr_enable.set(0x00);
        self.registers.intr_state.set(0xFFFF_FFFF);
    }

    /// Calculates and returns the max num words that can be programmed without
    /// crossing the programming resolution window boundaries, which
    /// occur at every FLASH_PROG_WINDOW_SIZE words. Note, when setting
    /// the CONTROL::NUM, write (ret_val - 1).
    fn calculate_max_prog_len(&self, word_addr: u32, rem_bytes: u32) -> u32 {
        // Calculate and return the max window limit possible for this transaction in words
        let window_limit =
            ((word_addr + FLASH_PROG_WINDOW_SIZE as u32) & FLASH_PROG_WINDOW_MASK) - word_addr;
        let words_to_write = rem_bytes / 4;

        if words_to_write < window_limit {
            words_to_write
        } else {
            window_limit
        }
    }

    fn configure_data_partition(&self, num: FlashRegion) {
        self.registers.default_region.write(
            DEFAULT_REGION::RD_EN::Set
                + DEFAULT_REGION::PROG_EN::Set
                + DEFAULT_REGION::ERASE_EN::Set,
        );

        self.registers.mp_region_cfg[num as usize].write(
            MP_REGION_CFG::RD_EN::Set
                + MP_REGION_CFG::PROG_EN::Set
                + MP_REGION_CFG::ERASE_EN::Set
                + MP_REGION_CFG::SCRAMBLE_EN::Clear
                + MP_REGION_CFG::ECC_EN::Clear
                + MP_REGION_CFG::EN::Clear,
        );

        // Size and base are stored in different registers
        self.registers.mp_region[num as usize]
            .write(MP_REGION::BASE.val(FLASH_PAGES_PER_BANK as u32) + MP_REGION::SIZE.val(0x1));

        // Enable MP Region
        self.registers.mp_region_cfg[num as usize].modify(MP_REGION_CFG::EN::Set);
        self.data_configured.set(true);
    }

    fn configure_info_partition(&self, bank: FlashBank, num: FlashRegion) {
        if bank == FlashBank::BANK0 {
            self.registers.bank0_info0_page_cfg[num as usize].write(
                BANK_INFO_PAGE_CFG::RD_EN::Set
                    + BANK_INFO_PAGE_CFG::PROG_EN::Set
                    + BANK_INFO_PAGE_CFG::ERASE_EN::Set
                    + BANK_INFO_PAGE_CFG::SCRAMBLE_EN::Set
                    + BANK_INFO_PAGE_CFG::ECC_EN::Set
                    + BANK_INFO_PAGE_CFG::EN::Clear,
            );
            self.registers.bank0_info0_page_cfg[num as usize].modify(BANK_INFO_PAGE_CFG::EN::Set);
        } else if bank == FlashBank::BANK1 {
            self.registers.bank1_info0_page_cfg[num as usize].write(
                BANK_INFO_PAGE_CFG::RD_EN::Set
                    + BANK_INFO_PAGE_CFG::PROG_EN::Set
                    + BANK_INFO_PAGE_CFG::ERASE_EN::Set
                    + BANK_INFO_PAGE_CFG::SCRAMBLE_EN::Set
                    + BANK_INFO_PAGE_CFG::ECC_EN::Set
                    + BANK_INFO_PAGE_CFG::EN::Clear,
            );
            self.registers.bank1_info0_page_cfg[num as usize].modify(BANK_INFO_PAGE_CFG::EN::Set);
        } else {
            panic!("Unsupported bank");
        }
        self.info_configured.set(true);
    }

    pub fn handle_interrupt(&self) {
        let irqs = self.registers.intr_state.extract();

        self.disable_interrupts();

        if irqs.is_set(INTR::OP_ERROR) {
            self.registers.op_status.set(0);

            let read_buf = self.read_buf.take();
            if let Some(buf) = read_buf {
                // We were doing a read
                self.flash_client.map(move |client| {
                    client.read_complete(buf, hil::flash::Error::FlashError);
                });
            }

            let write_buf = self.write_buf.take();
            if let Some(buf) = write_buf {
                // We were doing a write
                self.flash_client.map(move |client| {
                    client.write_complete(buf, hil::flash::Error::FlashError);
                });
            }
        }

        if irqs.is_set(INTR::RD_LVL) {
            self.read_buf.map(|buf| {
                while !self.registers.status.is_set(STATUS::RD_EMPTY)
                    && self.read_index.get() < PAGE_SIZE
                {
                    let data = self.registers.rd_fifo.get().to_ne_bytes();
                    let buf_offset = self.read_index.get();

                    buf[buf_offset] = data[0];
                    buf[buf_offset + 1] = data[1];
                    buf[buf_offset + 2] = data[2];
                    buf[buf_offset + 3] = data[3];

                    self.read_index.set(buf_offset + 4);
                }
                self.enable_interrupts();
            });
        }

        if irqs.is_set(INTR::PROG_EMPTY) {
            self.write_buf.map(|buf| {
                let transaction_word_len = self.calculate_max_prog_len(
                    self.write_word_addr.get() as u32,
                    (buf.0.len() - self.write_index.get()) as u32,
                );

                let mut words_written = 0;

                // Issue program command to the controller
                self.registers.control.write(
                    CONTROL::OP::PROG
                        + CONTROL::PARTITION_SEL::DATA
                        + CONTROL::INFO_SEL::CLEAR
                        + CONTROL::NUM.val(transaction_word_len as u32 - 1)
                        + CONTROL::START::CLEAR,
                );

                // Set the address
                self.registers
                    .addr
                    .write(ADDR::START.val(self.write_word_addr.get() as u32 * 4));

                // Start the transaction
                self.registers.control.modify(CONTROL::START::SET);

                for i in 0..transaction_word_len {
                    if self.registers.status.is_set(STATUS::PROG_FULL) {
                        words_written = i;
                        break;
                    }
                    let buf_offset = self.write_index.get();
                    let data: u32 = buf[buf_offset] as u32
                        | (buf[buf_offset + 1] as u32) << 8
                        | (buf[buf_offset + 2] as u32) << 16
                        | (buf[buf_offset + 3] as u32) << 24;

                    self.registers.prog_fifo.set(data);

                    self.write_index.set(buf_offset + 4);
                    // loop only semi-inclusive
                    words_written = i + 1;
                }

                self.write_word_addr
                    .set(self.write_word_addr.get() + words_written as usize);
                self.enable_interrupts();
            });
        }

        if irqs.is_set(INTR::OP_DONE) {
            if self.registers.control.matches_all(CONTROL::OP::READ) {
                let read_buf = self.read_buf.take();
                if let Some(buf) = read_buf {
                    // We were doing a read
                    if self.read_index.get() >= buf.0.len() {
                        self.registers.op_status.set(0);
                        // We have all of the data, call the client
                        self.flash_client.map(move |client| {
                            client.read_complete(buf, hil::flash::Error::CommandComplete);
                        });
                    } else {
                        // Still waiting on data, keep waiting
                        self.read_buf.replace(buf);
                        self.enable_interrupts();
                    }
                }
            } else if self.registers.control.matches_all(CONTROL::OP::PROG) {
                let write_buf = self.write_buf.take();
                if let Some(buf) = write_buf {
                    // We were doing a write
                    if self.write_index.get() >= buf.0.len() {
                        self.registers.op_status.set(0);
                        // We sent all of the data, call the client
                        self.flash_client.map(move |client| {
                            client.write_complete(buf, hil::flash::Error::CommandComplete);
                        });
                    } else {
                        // Still writing data, keep trying
                        self.write_buf.replace(buf);
                        self.enable_interrupts();
                    }
                }
            } else if self.registers.control.matches_all(CONTROL::OP::ERASE) {
                self.flash_client.map(move |client| {
                    client.erase_complete(hil::flash::Error::CommandComplete);
                });
            }
        }
    }
}

impl<C: hil::flash::Client<Self>> hil::flash::HasClient<'static, C> for FlashCtrl<'_> {
    fn set_client(&self, client: &'static C) {
        self.flash_client.set(client);
    }
}

impl hil::flash::Flash for FlashCtrl<'_> {
    type Page = LowRiscPage;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        let addr = page_number * PAGE_SIZE;

        if !self.info_configured.get() {
            // The info partitions have no default access. Specifically set up a region.
            self.configure_info_partition(FlashBank::BANK1, self.region_num);
        }

        if !self.data_configured.get() {
            // If we aren't configured yet, configure now
            self.configure_data_partition(self.region_num);
        }

        // Enable interrupts and set the FIFO level
        self.enable_interrupts();
        self.registers.fifo_lvl.modify(FIFO_LVL::RD.val(0xF));

        // Check control status before we commit
        if !self.registers.ctrl_regwen.is_set(CTRL_REGWEN::EN) {
            return Err((ErrorCode::BUSY, buf));
        }

        // Save the buffer
        self.read_buf.replace(buf);
        self.read_index.set(0);

        // Start the transaction
        self.registers.control.write(
            CONTROL::OP::READ
                + CONTROL::PARTITION_SEL::DATA
                + CONTROL::INFO_SEL::SET
                + CONTROL::NUM.val(((PAGE_SIZE / 4) - 1) as u32)
                + CONTROL::START::CLEAR,
        );

        // Set the address
        self.registers.addr.write(ADDR::START.val(addr as u32));

        // Start Transaction
        self.registers.control.modify(CONTROL::START::SET);

        Ok(())
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        let addr = page_number * PAGE_SIZE;

        if !self.info_configured.get() {
            // If we aren't configured yet, configure now
            // The info partitions have no default access. Specifically set up a region.
            self.configure_info_partition(FlashBank::BANK1, self.region_num);
        }

        if !self.data_configured.get() {
            // If we aren't configured yet, configure now
            self.configure_data_partition(self.region_num);
        }

        // Check control status before we commit
        if !self.registers.ctrl_regwen.is_set(CTRL_REGWEN::EN) {
            return Err((ErrorCode::BUSY, buf));
        }

        // Writes should not cross programming resolution window boundaries, which
        // occur at every FLASH_PROG_WINDOW_SIZE words.
        let word_address = addr / 4;

        let transaction_word_len =
            self.calculate_max_prog_len(word_address as u32, buf.0.len() as u32);

        self.registers.control.write(
            CONTROL::OP::PROG
                + CONTROL::PARTITION_SEL::DATA
                + CONTROL::INFO_SEL::CLEAR
                + CONTROL::NUM.val(transaction_word_len - 1)
                + CONTROL::START::CLEAR,
        );

        // Set the address
        self.registers.addr.write(ADDR::START.val(addr as u32));

        // Reset the write index
        self.write_index.set(0);

        // Start the transaction
        self.registers.control.modify(CONTROL::START::SET);

        let mut words_written = 0;

        // Write the data until we are full or have written all the data
        for i in 0..transaction_word_len {
            if self.registers.status.is_set(STATUS::PROG_FULL) {
                words_written = i;
                break;
            }
            let buf_offset = self.write_index.get();
            let data: u32 = buf[buf_offset] as u32
                | (buf[buf_offset + 1] as u32) << 8
                | (buf[buf_offset + 2] as u32) << 16
                | (buf[buf_offset + 3] as u32) << 24;

            self.registers.prog_fifo.set(data);

            self.write_index.set(buf_offset + 4);
            // loop only semi-inclusive
            words_written = i + 1;
        }

        self.write_word_addr
            .set((addr / 4) + words_written as usize);

        // Save the buffer
        self.write_buf.replace(buf);

        // Enable interrupts and set the FIFO level (interrupt when fully drained)
        self.enable_interrupts();
        self.registers.fifo_lvl.modify(FIFO_LVL::PROG.val(0x00));

        Ok(())
    }

    fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode> {
        let addr = page_number * PAGE_SIZE;

        if !self.data_configured.get() {
            // If we aren't configured yet, configure now
            self.configure_data_partition(self.region_num);
        }

        if !self.info_configured.get() {
            // If we aren't configured yet, configure now
            self.configure_info_partition(FlashBank::BANK1, self.region_num);
        }

        // Disable bank erase
        for _ in 0..2 {
            self.registers
                .mp_bank_cfg_shadowed
                .modify(MP_BANK_CFG::ERASE_EN_0::CLEAR + MP_BANK_CFG::ERASE_EN_1::CLEAR);
        }

        // Set the address
        self.registers.addr.write(ADDR::START.val(addr as u32));

        // Enable interrupts
        self.enable_interrupts();

        // Start the transaction
        self.registers.control.write(
            CONTROL::OP::ERASE
                + CONTROL::ERASE_SEL::PAGE
                + CONTROL::PARTITION_SEL::DATA
                + CONTROL::START::SET,
        );

        Ok(())
    }
}
