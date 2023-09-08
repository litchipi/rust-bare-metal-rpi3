pub static SPI: SpiDriver = SpiDriver::init();

pub struct SpiDriver;
impl SpiDriver {
    const fn init() -> SpiDriver {
        SpiDriver {}
    }
}
