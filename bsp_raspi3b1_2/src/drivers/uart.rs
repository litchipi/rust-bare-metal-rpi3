pub static UART: UartDriver = UartDriver::init();

pub struct UartDriver;
impl UartDriver {
    const fn init() -> UartDriver {
        UartDriver {}
    }
}
