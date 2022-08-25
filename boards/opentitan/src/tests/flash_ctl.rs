//! Test the opentitan Flash Controller
use crate::tests::run_kernel_op;
use crate::PERIPHERALS;
use capsules::virtual_flash::FlashUser;
use core::cell::Cell;
use core::ops::IndexMut;
use kernel::debug;
use kernel::hil;
use kernel::hil::flash::Client;
use kernel::hil::flash::Flash;
use kernel::hil::flash::HasClient;
use kernel::static_init;
use kernel::utilities::cells::TakeCell;

struct FlashCtlCallBack {
    read_pending: Cell<bool>,
    write_pending: Cell<bool>,
    op_buf: TakeCell<'static, lowrisc::flash_ctrl::LowRiscPage>,
    cmp_buf: TakeCell<'static, lowrisc::flash_ctrl::LowRiscPage>,
    ret_buf: TakeCell<'static, [u8]>,
}

impl<'a> FlashCtlCallBack {
    fn new(
        buf: &'static mut lowrisc::flash_ctrl::LowRiscPage,
        cbuf: &'static mut lowrisc::flash_ctrl::LowRiscPage,
    ) -> Self {
        FlashCtlCallBack {
            read_pending: Cell::new(false),
            write_pending: Cell::new(false),
            op_buf: TakeCell::new(buf),
            cmp_buf: TakeCell::new(cbuf),
            ret_buf: TakeCell::empty(),
        }
    }

    fn reset(&self) {
        self.read_pending.set(false);
        self.write_pending.set(false);
    }
}

impl<'a, F: hil::flash::Flash> hil::flash::Client<F> for FlashCtlCallBack {
    fn read_complete(&self, buffer: &'static mut F::Page, error: hil::flash::Error) {
        if self.read_pending.get() {
            assert_eq!(error, hil::flash::Error::CommandComplete);
            self.ret_buf.replace(buffer.as_mut());
            self.read_pending.set(false);
        }
    }

    fn write_complete(&self, buffer: &'static mut F::Page, error: hil::flash::Error) {
        if self.write_pending.get() {
            assert_eq!(error, hil::flash::Error::CommandComplete);
            self.ret_buf.replace(buffer.as_mut());
            self.write_pending.set(false);
        }
    }

    fn erase_complete(&self, error: hil::flash::Error) {
        assert_eq!(error, hil::flash::Error::CommandComplete);
    }
}

unsafe fn static_init_block() -> &'static FlashCtlCallBack {
    let mut buf = static_init!(
        lowrisc::flash_ctrl::LowRiscPage,
        lowrisc::flash_ctrl::LowRiscPage::default()
    );
    let mut cbuf = static_init!(
        lowrisc::flash_ctrl::LowRiscPage,
        lowrisc::flash_ctrl::LowRiscPage::default()
    );

    let mut val: u8 = 0;

    for i in 0..lowrisc::flash_ctrl::PAGE_SIZE {
        val = val.wrapping_add(10);
        buf[i] = 0xAA;
        cbuf[i] = 0xAA;
    }
    static_init!(FlashCtlCallBack, FlashCtlCallBack::new(buf, cbuf))
}

#[test_case]
fn flash_ctl_write_page() {
    let perf = unsafe { PERIPHERALS.unwrap() };
    let flash_ctl = &perf.flash_ctrl;

    let cb = unsafe { static_init_block() };
    cb.reset();

    debug!("Start page write....");

    #[cfg(feature = "hardware_tests")]
    {
        let buf = cb.op_buf.take().unwrap();
        flash_ctl.set_client(cb);
        run_kernel_op(100);
        // Lets do a page erase
        assert!(flash_ctl.erase_page(5).is_ok());
        run_kernel_op(100);

        // Do Page Write
        assert!(flash_ctl.write_page(5, buf).is_ok());
        cb.write_pending.set(true);
        run_kernel_op(100);
        // OP Complete, buffer recovered.
        assert!(!cb.write_pending.get());
        cb.reset();

        // Read the same page
        let buf = cb.cmp_buf.take().unwrap();
        assert!(flash_ctl.read_page(5, buf).is_ok());
        cb.read_pending.set(true);
        run_kernel_op(100);
        assert!(!cb.read_pending.get());
        cb.reset();

        // Compare r/w buffer
        let buf = cb.ret_buf.take().unwrap()
        panic!("rc: {:?}", cb.ret_buf.take().unwrap());
    }

    run_kernel_op(100);
    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn flash_ctl_read_page() {
    debug!("Start page write....");
    run_kernel_op(100);
}

#[test_case]
fn flash_ctl_erase_page() {
    debug!("Start page erase....");
    run_kernel_op(100);
}
