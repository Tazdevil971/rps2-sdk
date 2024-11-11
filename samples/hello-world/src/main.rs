#![no_std]
use rps2::dbg;

struct DetonatingBomb;

fn test() {
    dbg!("Bruh2");
    panic!("Yeet");
    dbg!("Bruh3");
}

fn test1() {
    let _bomb = DetonatingBomb;
    dbg!("Bruh1");
    test();
    dbg!("Bruh4");
}

fn main() {
    rps2::kprintln!("Cacati Adriano");

    dbg!("We are here");

    let _ = rps2::panic::catch_unwind(|| {
        dbg!("Also here?");
        test1();
        dbg!("But never here!");
    });

    dbg!("And now here? Magic");

    funny_colors();
}

fn slope(i: u32) -> u32 {
    let i = i % 360;

    if i <= 60 {
        i * 255 / 60
    } else if i < 180 {
        255
    } else if i < 240 {
        ((300 - i) * 255) / 60
    } else {
        0
    }
}

fn funny_colors() {
    const GS_PMODE: *mut u64 = 0x12000000 as _;
    const GS_DISPFB1: *mut u64 = 0x12000070 as _;
    const GS_DISPLAY1: *mut u64 = 0x12000080 as _;
    const GS_DISPFB2: *mut u64 = 0x12000090 as _;
    const GS_DISPLAY2: *mut u64 = 0x120000a0 as _;
    const GS_BGCOLOR: *mut u64 = 0x120000e0 as _;
    const GS_CSR: *mut u64 = 0x12001000 as _;

    unsafe {
        GS_CSR.write_volatile(1 << 9);
        rps2::arch::syncp();
        GS_CSR.write_volatile(0);
    }

    unsafe {
        rps2::os::gs_put_imr(0x00007f00);
        rps2::os::set_gs_crt(0x00, 0x03, 0x00);
    }

    // Step 5: Disable all color outputs from the GS
    unsafe {
        rps2::interrupt_disable_guard!();

        GS_PMODE.write_volatile(0x000000a5);
        GS_DISPFB1.write_volatile(0x00009400);
        GS_DISPFB2.write_volatile(0x00009400);
        GS_DISPLAY1.write_volatile(0xfff9ff38014338);
        GS_DISPLAY2.write_volatile(0xfff9ff38014338);
    }

    // Step 6: Show different colors on the screen
    for i in 0.. {
        let h = i;
        let r = slope(h + 240);
        let g = slope(h + 120);
        let b = slope(h);

        let bgcolor = r | (g << 8) | (b << 16);
        unsafe {
            GS_BGCOLOR.write_volatile(bgcolor as _);
        }

        for _ in 0..200000 {
            unsafe {
                rps2::arch::sync();
            }
        }
    }
}
