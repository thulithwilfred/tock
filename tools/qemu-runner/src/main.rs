use std::process::Command;

use rexpect::errors::Error;
use rexpect::session::PtySession;
use rexpect::spawn;

fn kill_qemu(p: &mut PtySession) -> Result<(), Error> {
    p.send_control('a')?;
    p.send("x")?;
    p.flush()?;

    Ok(())
}

fn hifive1() -> Result<(), Error> {
    // First, build the board if needed
    // n.b. rexpect's `exp_eof` does not actually block main thread, so use
    // the standard Rust process library mechanism instead.
    let mut build = Command::new("make")
        .arg("-C")
        .arg("../../boards/hifive1")
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let mut p = spawn("make qemu -C ../../boards/hifive1", Some(3_000))?;

    p.exp_string("HiFive1 initialization complete.")?;
    p.exp_string("Entering main loop.")?;

    // Test completed, kill QEMU
    kill_qemu(&mut p)?;

    p.exp_string("QEMU: Terminated")?;
    Ok(())
}

fn earlgrey_cw310() -> Result<(), Error> {
    // First, build the board if needed
    // n.b. rexpect's `exp_eof` does not actually block main thread, so use
    // the standard Rust process library mechanism instead.
    let mut build = Command::new("make")
        .arg("-C")
        .arg("../../boards/opentitan/earlgrey-cw310")
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    // Get canonicalized path to opentitan rom
    let mut rom_path = std::env::current_exe().unwrap();
    rom_path.pop(); // strip exe file
    rom_path.pop(); // strip /debug
    rom_path.pop(); // strip /target
    rom_path.push("opentitan-boot-rom.elf");

    let mut p = spawn(
        "make qemu -C ../../boards/opentitan/earlgrey-cw310",
        Some(10_000),
    )?;

    p.exp_string("OpenTitan initialisation complete. Entering main loop")?;

    // Test completed, kill QEMU
    kill_qemu(&mut p)?;

    p.exp_string("QEMU: Terminated")?;
    Ok(())
}

fn main() {
    println!("Tock qemu-runner starting...");
    println!("");
    println!("Running hifive1 tests...");
    hifive1().unwrap_or_else(|e| panic!("hifive1 job failed with {}", e));
    println!("hifive1 SUCCESS.");
    println!("");
    println!("Running earlgrey_cw310 tests...");
    earlgrey_cw310().unwrap_or_else(|e| panic!("earlgrey_cw310 job failed with {}", e));
    println!("earlgrey_cw310 SUCCESS.");
}
