use serial;

#[lang = "panic_fmt"]
#[linkage = "weak"]
unsafe extern "C" fn panic_fmt(
    _args: ::core::fmt::Arguments,
    _file: &'static str,
    _line: u32,
) -> ! {
    print!("panicked at '");
    serial::print(_args);
    println!("', {}:{}", _file, _line);

    asm!("brk #0" :::: "volatile");

    loop {}
}

#[lang = "start"]
pub extern "C" fn start(
    main: fn(),
    _argc: isize,
    _argv: *const *const u8,
) -> isize {
    main(); 0
}

#[lang = "eh_personality"] extern fn eh_personality() {}
