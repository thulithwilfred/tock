//! Interface for reading, writing, and erasing flash storage pages.
//!
//! Operates on single pages. The page size is set by the associated type
//! `page`. Here is an example of a page type and implementation of this trait:
//!
//! ```rust
//! use core::ops::{Index, IndexMut};
//!
//! use kernel::hil;
//! use kernel::ErrorCode;
//!
//! // Size in bytes
//! const PAGE_SIZE: u32 = 1024;
//!
//! struct NewChipPage(pub [u8; PAGE_SIZE as usize]);
//!
//! impl Default for NewChipPage {
//!     fn default() -> Self {
//!         Self {
//!             0: [0; PAGE_SIZE as usize],
//!         }
//!     }
//! }
//!
//! impl NewChipPage {
//!     fn len(&self) -> usize {
//!         self.0.len()
//!     }
//! }
//!
//! impl Index<usize> for NewChipPage {
//!     type Output = u8;
//!
//!     fn index(&self, idx: usize) -> &u8 {
//!         &self.0[idx]
//!     }
//! }
//!
//! impl IndexMut<usize> for NewChipPage {
//!     fn index_mut(&mut self, idx: usize) -> &mut u8 {
//!         &mut self.0[idx]
//!     }
//! }
//!
//! impl AsMut<[u8]> for NewChipPage {
//!     fn as_mut(&mut self) -> &mut [u8] {
//!         &mut self.0
//!     }
//! }
//!
//! struct NewChipStruct {};
//!
//! impl<'a, C> hil::flash::HasClient<'a, C> for NewChipStruct {
//!     fn set_client(&'a self, client: &'a C) { }
//! }
//!
//! impl hil::flash::Flash for NewChipStruct {
//!     type Page = NewChipPage;
//!
//!     fn read_page(&self, page_number: usize, buf: &'static mut Self::Page) -> Result<(), (ErrorCode, &'static mut Self::Page)> { Err((ErrorCode::FAIL, buf)) }
//!     fn write_page(&self, page_number: usize, buf: &'static mut Self::Page) -> Result<(), (ErrorCode, &'static mut Self::Page)> { Err((ErrorCode::FAIL, buf)) }
//!     fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode> { Err(ErrorCode::FAIL) }
//! }
//! ```
//!
//! A user of this flash interface might look like:
//!
//! ```rust
//! use kernel::utilities::cells::TakeCell;
//! use kernel::hil;
//!
//! pub struct FlashUser<'a, F: hil::flash::Flash + 'static> {
//!     driver: &'a F,
//!     buffer: TakeCell<'static, F::Page>,
//! }
//!
//! impl<'a, F: hil::flash::Flash> FlashUser<'a, F> {
//!     pub fn new(driver: &'a F, buffer: &'static mut F::Page) -> FlashUser<'a, F> {
//!         FlashUser {
//!             driver: driver,
//!             buffer: TakeCell::new(buffer),
//!         }
//!     }
//! }
//!
//! impl<'a, F: hil::flash::Flash> hil::flash::Client<F> for FlashUser<'a, F> {
//!     fn read_complete(&self, buffer: &'static mut F::Page, error: hil::flash::Error) {}
//!     fn write_complete(&self, buffer: &'static mut F::Page, error: hil::flash::Error) { }
//!     fn erase_complete(&self, error: hil::flash::Error) {}
//! }
//! ```

use crate::ErrorCode;

/// Flash errors returned in the callbacks.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Success.
    CommandComplete,

    /// An error occurred during the flash operation.
    FlashError,

    /// A flash memory protection violation was detected
    FlashMPError,
}

pub trait HasClient<'a, C> {
    /// Set the client for this flash peripheral. The client will be called
    /// when operations complete.
    fn set_client(&'a self, client: &'a C);
}

/// A page of writable persistent flash memory.
pub trait Flash {
    /// Type of a single flash page for the given implementation.
    type Page: AsMut<[u8]> + Default;

    /// Read a page of flash into the buffer.
    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)>;

    /// Write a page of flash from the buffer.
    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)>;

    /// Erase a page of flash by setting every byte to 0xFF.
    fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode>;
}

/// Implement `Client` to receive callbacks from `Flash`.
pub trait Client<F: Flash> {
    /// Flash read complete.
    fn read_complete(&self, read_buffer: &'static mut F::Page, error: Error);

    /// Flash write complete.
    fn write_complete(&self, write_buffer: &'static mut F::Page, error: Error);

    /// Flash erase complete.
    fn erase_complete(&self, error: Error);
}

// *** Interfaces for hardware with flash memory protection support ***

/// Define the basic region permissions for flash memory protection.
#[derive(PartialEq, Debug)]
pub struct FlashMPBasicConfig {
    /// Region can be read.
    pub read_en: bool,
    /// Region can be programmed.
    pub write_en: bool,
}

/// Memory access control protection for flash. For hardware that supports
/// flash memory protection, this can be used to implement the relevant functionality.
///
/// Requires the Flash interface to have been implemented as well.
pub trait FlashMemoryProtection: Flash {
    /// Configure and enable the specified flash memory protection configuration
    ///
    /// # Arguments
    ///
    /// * `page_number` - Defines the starting page number to apply configuration to
    /// * `num_pages` - Number of pages the configs are applied to (region size)
    /// * `region_num` - The configuration region number associated with this region
    /// * `mp_perms` - Specifies the permissions to set
    fn set_region_perms(
        &self,
        page_number: usize,
        num_pages: usize,
        region_num: usize,
        mp_perms: &FlashMPBasicConfig,
    ) -> Result<(), ErrorCode>;

    /// Read the flash memory protection configuration bounded by the specified region
    ///
    /// # Arguments
    ///
    /// * `region_num` - The configuration region number associated with this region
    fn read_region_perms(&self, region_num: usize) -> Result<FlashMPBasicConfig, ErrorCode>;

    /// Get the number of configuration regions supported by this hardware
    ///
    /// Note: Indexing typically starts with 0, this returns the total
    /// number of configuration registers.
    /// Example: if retval is 8, index 7 is the upper limit.
    fn get_num_regions(&self) -> Result<u32, ErrorCode>;

    /// Check if the specified `region_num` is locked by hardware
    ///
    /// # Arguments
    ///
    /// * `region_num` - The configuration region number associated with this region
    fn is_region_locked(&self, region_num: usize) -> Result<bool, ErrorCode>;

    /// Lock the configuration
    /// If supported by hardware, locks the config bounded by `region_num`
    /// such that no further modifications can be made until the next system reset.
    ///
    /// # Arguments
    ///
    /// * `region_num` - The configuration region number associated with this region
    fn lock_region_cfg(&self, region_num: usize) -> Result<(), ErrorCode>;
}

// *** Interfaces for hardware with advanced flash memory protection support ***

/// Defines region permissions for flash memory protection.
/// With support to control more advanced features.
#[derive(PartialEq, Debug)]
pub struct FlashMPAdvConfig {
    /// Region can be read.
    pub read_en: bool,
    /// Region can be programmed.
    pub write_en: bool,
    /// Region can be erased
    pub erase_en: bool,
    /// Region is scramble enabled
    pub scramble_en: bool,
    /// Region has ECC enabled
    pub ecc_en: bool,
    /// Region is high endurance enabled
    pub he_en: bool,
}

/// Extends FlashMemoryProtection to interface advanced flash
/// memory protection controllers with more control/functionality.
/// For devices that only support r/w permissions only use `FlashMemoryProtection`.
///
/// Requires the Flash and FlashMemoryProtection interfaces to have been implemented as well.
pub trait FlashMemoryProtectionAdvanced: FlashMemoryProtection + Flash {
    /// Setup the specified flash memory protection configuration
    ///
    /// # Arguments
    ///
    /// * `page_number` - Defines the starting page number to apply configuration to
    /// * `num_pages` - Number of pages the configs are applied to (region size)
    /// * `region_num` - The configuration region number associated with this region
    /// * `mp_perms` - Specifies the permissions to set
    fn set_adv_region_perms(
        &self,
        page_number: usize,
        num_pages: usize,
        region_num: usize,
        mp_perms: &FlashMPAdvConfig,
    ) -> Result<(), ErrorCode>;

    /// Read the flash memory protection configuration bounded by the specified region
    ///
    /// # Arguments
    ///
    /// * `region_num` - The configuration region number associated with this region
    fn read_adv_region_perms(&self, region_num: usize) -> Result<FlashMPAdvConfig, ErrorCode>;
}
