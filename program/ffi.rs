use super::Program;

const fn assert_send<T: Send>() {}
const _: () = assert_send::<Program>();

#[no_mangle]
pub fn sample(this: *mut (), t: f64) -> (f64, f64) {
    let this = unsafe { &mut *(this as *mut Program) };
    this.sample(t)
}

#[no_mangle]
pub fn pad_in(this: *mut (), t: f64, input: types::launchpad_x::Input) {
    let this = unsafe { &mut *(this as *mut Program) };
    this.pad_in(t, input)
}
#[no_mangle]
pub fn pad_out(this: *mut (), t: f64) -> types::launchpad_x::Output {
    let this = unsafe { &mut *(this as *mut Program) };
    this.pad_out(t)
}

#[no_mangle]
pub fn ctrl_in(this: *mut (), t: f64, input: types::launch_control_xl::Input) {
    let this = unsafe { &mut *(this as *mut Program) };
    this.ctrl_in(t, input)
}
#[no_mangle]
pub fn ctrl_out(this: *mut (), t: f64) -> types::launch_control_xl::Output {
    let this = unsafe { &mut *(this as *mut Program) };
    this.ctrl_out(t)
}

#[no_mangle]
pub fn default() -> *mut () {
    let this = Program::default();
    Box::into_raw(Box::new(this)) as *mut ()
}

#[no_mangle]
pub fn serialize(this: *mut ()) -> Vec<u8> {
    let state: &Program = unsafe { &*(this as *mut Program) };
    serde_json::to_vec(state).unwrap()
}

#[no_mangle]
pub fn deserialize(bytes: &[u8]) -> Result<*mut (), Box<dyn std::error::Error>> {
    let state: Program = serde_json::from_reader(bytes)?;
    Ok(Box::into_raw(Box::new(state)) as *mut ())
}

#[no_mangle]
pub fn drop(this: *mut ()) {
    let _ = unsafe { Box::from_raw(this as *mut Program) };
}
