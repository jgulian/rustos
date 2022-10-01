use core::panic::PanicInfo;
use crate::kprintln;

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    let sussy = "            (
       (      )     )
         )   (    (
        (          `
    .-\"\"^\"\"\"^\"\"^\"\"\"^\"\"-.
  (//\\\\//\\\\//\\\\//\\\\//\\\\//)
   ~\\^^^^^^^^^^^^^^^^^^/~
     `================`

    The pi is overdone.

---------- PANIC ----------";
    kprintln!("{}", sussy);

    match panic_info.location() {
        Some(location) => {
            kprintln!("FILE: {}", location.file());
            kprintln!("LINE: {}", location.line());
            kprintln!("COL: {}", location.column());
        },
        None => { kprintln!("unknown location"); }
    }

    kprintln!("");

    match panic_info.message() {
        Some(message) => {
            kprintln!("{}", message);
        },
        None => {}
    }



    loop {}
}
