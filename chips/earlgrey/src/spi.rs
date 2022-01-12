use kernel::utilities::StaticRef;
use lowrisc::spi::SpiDeviceRegisters;

pub const SPIHOST0_BASE: StaticRef<SpiDeviceRegisters> =
    unsafe { StaticRef::new(0x4006_0000 as *const SpiDeviceRegisters) };
