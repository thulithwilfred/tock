use core::cell::Cell;
use core::cmp;
use core::option::Option;
use kernel::debug;
use kernel::hil;
use kernel::hil::spi::SpiMaster;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use kernel::static_init;
use crate::tests::run_kernel_op;
use crate::PERIPHERALS;
use kernel::hil::spi::SpiMasterClient;

struct SpiHostCallback {
    transfer_done: Cell<bool>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
}

impl SpiMasterClient for SpiHostCallback {
    fn read_write_done(&self, tx_bytes: &'static mut [u8], rx_bytes: Option<&'static mut [u8]>, tx_num_bytes: usize, rc: Result<(), ErrorCode>) { 
        //No Errors
        assert_eq!(rc, Ok(()));
        //Sent request len vs actual transmit length
        assert_eq!(self.tx_len.get(), tx_num_bytes);
        //Dont loose buffers
        self.tx_buffer.replace(tx_bytes);
        //Should be safe to unwrap here
        self.rx_buffer.replace(rx_bytes.unwrap());
        //debug!("\n\nTEST OUT: TX: {:?} \n\n RX: {:?}", tx_bytes, rx_bytes);
    }
}

impl<'a> SpiHostCallback {
    fn new(tx_data: &'static mut [u8; 32], tx_len: usize, rx_data: &'static mut [u8; 32], rx_len: usize) -> Self {
        SpiHostCallback {
            transfer_done: Cell::new(false),
            tx_buffer: TakeCell::new(tx_data),
            tx_len: Cell::new(tx_len),
            rx_buffer: TakeCell::new(rx_data),
            rx_len: Cell::new(rx_len)
        }
    }

    fn reset(&self) {
        self.transfer_done.set(false);
    }
}


unsafe fn static_init_test_cb() -> &'static SpiHostCallback {
    let rx_data = static_init!([u8; 32], [32; 32]);
    let tx_data = static_init!(
        [u8; 32],
        [
            0xdc, 0x55, 0x51, 0x5e, 0x30, 0xac, 0x50, 0xc7, 0x65, 0xbd, 0xe, 0x2, 0x82, 0xf7, 0x8b,
            0xe1, 0xef, 0xd1, 0xb, 0xdc, 0xa8, 0xba, 0xe1, 0xfa, 0x11, 0x3f, 0xf6, 0xeb, 0xaf,
            0x58, 0x57, 0x01,
        ]
    );

    static_init!(
        SpiHostCallback,
        SpiHostCallback::new(tx_data, tx_data.len(),rx_data, rx_data.len()),
    )
}

//Test SPI R/W BYTES
#[test_case]
fn spi_host_read_write() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let callback = unsafe { static_init_test_cb() };
    let spi = &perf.spi_host0;

    debug!("load spi_host tx buffer... ");
    run_kernel_op(100);

    spi.set_client(callback);
    callback.reset();

    let rx = callback.rx_buffer.take().unwrap();
    let rx_len = callback.rx_len.get();
    let tx = callback.tx_buffer.take().unwrap();
    
    assert_eq!(spi.read_write_bytes(tx, Some(rx), rx_len), Ok(()));
    
    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(100);
}

//Interrupt 