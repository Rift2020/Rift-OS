use core::panic::PanicInfo;
use crate::eprintln;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        eprintln!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        eprintln!("Panicked: {}", info.message().unwrap());
    }
    loop {}
}
