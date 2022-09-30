use kernel::utilities::StaticRef;
use lowrisc::pwm::PWMRegisters;

pub const PWM_BASE: StaticRef<PWMRegisters> =
    unsafe { StaticRef::new(0x4045_0000 as *const PWMRegisters) };
