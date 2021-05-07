use std::ffi::CStr;
use std::os::raw::{c_char, c_double, c_void};

type TDLibClient = *mut c_void;

pub struct Client {
    instance: TDLibClient,
}

#[link(name = "tdjson")]
extern "C" {
    fn td_json_client_create() -> TDLibClient;
    fn td_json_client_send(client: TDLibClient, request: *const c_char);
    fn td_json_client_receive(client: TDLibClient, timeout: c_double) -> *mut c_char;
    fn td_json_client_destroy(client: TDLibClient);
}

impl Client {
    pub fn new() -> Self {
        let client = unsafe { td_json_client_create() };
        Client { instance: client }
    }

    pub fn next_update(&self) {
        let response = unsafe {
            match td_json_client_receive(self.instance, 10.0)
                .as_ref()
                .map(|response| CStr::from_ptr(response).to_string_lossy().into_owned()) {
                None => {
                    None
                }
                Some(contents) => {
                    Some(contents)
                }
            }
        };

        if let Some(response) = response {
            println!("{}", response);
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe { td_json_client_destroy(self.instance) }
    }
}
