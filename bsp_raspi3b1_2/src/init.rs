use crate::errors::Errcode;

// First to be called
//    Initialize default values everywhere
//    Disable all devices by default
//    Set all pins as input by default
//    Init IRQ controller and Timer
//    Init allocator
//    Init GPU
pub fn init_bsp() -> Result<(), Errcode> {
    init_drivers()?;
    Ok(())
}

fn init_drivers() -> Result<(), Errcode> {
    Ok(())
}

fn init_allocator() -> Result<(), Errcode> {
    todo!();
}

fn disable_all_devices() -> Result<(), Errcode> {
    todo!();
}

fn init_irq_controller() -> Result<(), Errcode> {
    todo!();
}

fn init_timer() -> Result<(), Errcode> {
    todo!();
}

// IRQ register

// pub fn register_irq(nb: u32, handler: Box<dyn FnOnce()>) {
//     todo!();
// }
