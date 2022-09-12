//! Chip specific configuration.
//!
//! This file includes configuration values for different implementations and
//! uses of the same earlgrey chip. For example, running the chip on an FPGA
//! requires different parameters from running it in a verilog simulator.
//! Additionally, chips on different platforms can be used differently, so this
//! also permits changing values like the UART baud rate to enable better
//! debugging on platforms that can support it.
//!
//! The configuration used is selected via Cargo features specified when the
//! board is compiled.

/// Earlgrey configuration based on the target device.
pub struct Config<'a> {
    /// Identifier for the platform. This is useful for debugging to confirm the
    /// correct configuration of the chip is being used.
    pub name: &'a str,
    /// The clock speed of the CPU in Hz.
    pub cpu_freq: u32,
    /// The clock speed of the peripherals in Hz.
    pub peripheral_freq: u32,
    /// The clock of the AON Timer
    pub aon_timer_freq: u32,
    /// The baud rate for UART. This allows for a version of the chip that can
    /// support a faster baud rate to use it to help with debugging.
    pub uart_baudrate: u32,
}

/// Config for running EarlGrey on the CW310 FPGA
#[cfg(any(feature = "config_fpga_cw310", not(feature = "config_disable_default")))]
pub const CONFIG: Config = Config {
    name: "fpga_cw310",
    cpu_freq: 10_000_000,
    peripheral_freq: 2_500_000,
    aon_timer_freq: 250_000,
    uart_baudrate: 115200,
};

/// Config for running EarlGrey in a verilog simulator.
#[cfg(feature = "config_sim_verilator")]
pub const CONFIG: Config = Config {
    name: "sim_verilator",
    cpu_freq: 500_000,
    peripheral_freq: 125_000,
    aon_timer_freq: 125_000,
    uart_baudrate: 7200,
};
