use kernel::utilities::StaticRef;
use lowrisc::spi_host::SpiHostRegisters;

pub const SPIHOST0_BASE: StaticRef<SpiHostRegisters> =
    unsafe { StaticRef::new(0x4006_0000 as *const SpiHostRegisters) };

pub const SPIHOST1_BASE: StaticRef<SpiHostRegisters> =
    unsafe { StaticRef::new(0x4007_0000 as *const SpiHostRegisters) };
