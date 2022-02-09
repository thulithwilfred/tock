use crate::tests::run_kernel_op;
use crate::PERIPHERALS;
use core::cell::Cell;
use kernel::static_init;
use kernel::utilities::cells::TakeCell;
use kernel::{debug, ErrorCode};
use kernel::hil::spi::{SpiMasterClient, SpiMaster};
use kernel::hil::spi::{ClockPhase, ClockPolarity};

struct SpiHostCallback {
    transfer_done: Cell<bool>,
    tx_len: Cell<usize>,
    tx_data: TakeCell<'static, [u8]>,
    rx_data: TakeCell<'static, [u8]>,
}

impl<'a> SpiHostCallback {
    fn new(tx_data: &'static mut [u8], rx_data: &'static mut [u8]) -> Self {
        SpiHostCallback {
            transfer_done: Cell::new(false),
            tx_len: Cell::new(0),
            tx_data: TakeCell::new(tx_data),
            rx_data: TakeCell::new(rx_data),
        }
    }

    fn reset(&self) {
        self.transfer_done.set(false);
        self.tx_len.set(0);
    }
}

impl<'a> SpiMasterClient for SpiHostCallback {
    fn read_write_done(&self, tx_data: &'static mut [u8], rx_done: Option<&'static mut [u8]>, tx_len: usize, rc: Result<(), ErrorCode>) { 
        //Transfer Complete
        assert_eq!(tx_len, self.tx_len.get());

        //Capture Buffers
        self.tx_data.replace(tx_data);
        if rx_done.is_none() == false {
            self.tx_data.replace(rx_done.unwrap());
        } else {
            //Sad Reacts Only
            panic!("RX Buffer Lost");
        }
        self.transfer_done.set(true);
    }
}

unsafe fn static_init_test_cb() -> &'static SpiHostCallback {
    let tx_data = static_init!([u8; 64], [64; 64]);
    //Some Data
    let rx_data = static_init!(
        [u8; 64],
        [
            0xdc, 0x55, 0x51, 0x5e, 0x30, 0xac, 0x50, 0xc7, 0x65, 0xbd, 0xe, 0x2, 0x82, 0xf7, 0x8b,
            0xe1, 0xef, 0xd1, 0xb, 0xdc, 0xa8, 0xba, 0xe1, 0xfa, 0x11, 0x3f, 0xf6, 0xeb, 0xaf,
            0x58, 0x57, 0x40, 0xdc, 0x55, 0x51, 0x5e, 0x30, 0xac, 0x50, 0xc7, 0x65, 0xbd, 0xe, 0x2, 0x82, 0xf7, 0x8b,
            0xe1, 0xef, 0xd1, 0xb, 0xdc, 0xa8, 0xba, 0xe1, 0xfa, 0x11, 0x3f, 0xf6, 0xeb, 0xaf,
            0x58, 0x57, 0x40,
        ]
    );

    static_init!(
        SpiHostCallback,
        SpiHostCallback::new(tx_data, rx_data)
    )
}

#[test_case]
fn spi_host_transfer() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let spi_host = &perf.spi_host0;

    let cb = unsafe { static_init_test_cb() };
 
    debug!("[SPI] Setup spi_host0 transfer... ");
    run_kernel_op(100);

    spi_host.set_client(cb);
    cb.reset();

    let tx = cb.tx_data.take().unwrap();
    let rx = cb.rx_data.take().unwrap();
    cb.tx_len.set(tx.len());

    //Set SPI_HOST0 Configs
    spi_host.specify_chip_select(0);
    spi_host.set_rate(100000);
    spi_host.set_polarity(ClockPolarity::IdleLow);
    spi_host.set_phase(ClockPhase::SampleLeading);


    assert_eq!(spi_host.read_write_bytes(tx, Some(rx),  cb.tx_len.get()), Ok(()));

    run_kernel_op(10000);
    #[cfg(feature = "hardware_tests")]
    assert_eq!(cb.transfer_done.get(), true);

    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(100);
}