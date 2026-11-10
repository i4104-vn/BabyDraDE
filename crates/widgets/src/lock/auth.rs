//! PAM authentication helper for the BabyDra screen locker.
//! Integrates with the system's "login" PAM service to verify passwords.

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};
use std::sync::OnceLock;

pub const PAM_PROMPT_ECHO_OFF: c_int = 1;
pub const PAM_PROMPT_ECHO_ON: c_int = 2;
pub const PAM_SUCCESS: c_int = 0;

#[repr(C)]
pub struct pam_message {
    pub msg_style: c_int,
    pub msg: *const c_char,
}

#[repr(C)]
pub struct pam_response {
    pub resp: *mut c_char,
    pub resp_retcode: c_int,
}

#[repr(C)]
pub struct pam_conv {
    pub conv: Option<
        unsafe extern "C" fn(
            num_msg: c_int,
            msg: *mut *mut pam_message,
            resp: *mut *mut pam_response,
            appdata_ptr: *mut c_void,
        ) -> c_int,
    >,
    pub appdata_ptr: *mut c_void,
}

type PamStartFn = unsafe extern "C" fn(
    service_name: *const c_char,
    user: *const c_char,
    pam_conversation: *const pam_conv,
    pamh: *mut *mut c_void,
) -> c_int;

type PamAuthenticateFn = unsafe extern "C" fn(pamh: *mut c_void, flags: c_int) -> c_int;

type PamEndFn = unsafe extern "C" fn(pamh: *mut c_void, pam_status: c_int) -> c_int;

struct PamLib {
    pam_start: PamStartFn,
    pam_authenticate: PamAuthenticateFn,
    pam_end: PamEndFn,
}

unsafe impl Send for PamLib {}
unsafe impl Sync for PamLib {}

static PAM: OnceLock<Option<PamLib>> = OnceLock::new();

fn get_pam() -> Option<&'static PamLib> {
    PAM.get_or_init(|| {
        unsafe {
            // Dynamically load libpam to avoid hard build-time link dependency on -lpam
            let lib_names = [
                c"libpam.so.0".as_ptr(),
                c"libpam.so".as_ptr(),
            ];

            let mut handle = std::ptr::null_mut();
            for name in lib_names {
                handle = libc::dlopen(name, libc::RTLD_LAZY);
                if !handle.is_null() {
                    break;
                }
            }

            if handle.is_null() {
                let err = libc::dlerror();
                let msg = if !err.is_null() {
                    std::ffi::CStr::from_ptr(err).to_string_lossy().into_owned()
                } else {
                    "unknown dlopen error".to_string()
                };
                tracing::warn!("Failed to dynamic-load PAM library: {}", msg);
                return None;
            }

            let start_sym = libc::dlsym(handle, c"pam_start".as_ptr());
            let auth_sym = libc::dlsym(handle, c"pam_authenticate".as_ptr());
            let end_sym = libc::dlsym(handle, c"pam_end".as_ptr());

            if start_sym.is_null() || auth_sym.is_null() || end_sym.is_null() {
                tracing::warn!("Failed to resolve PAM symbols from library");
                libc::dlclose(handle);
                return None;
            }

            Some(PamLib {
                pam_start: std::mem::transmute::<*mut c_void, PamStartFn>(start_sym),
                pam_authenticate: std::mem::transmute::<*mut c_void, PamAuthenticateFn>(auth_sym),
                pam_end: std::mem::transmute::<*mut c_void, PamEndFn>(end_sym),
            })
        }
    })
    .as_ref()
}

/// PAM conversation callback function.
///
/// # Safety
/// Caller must ensure pointers conform to PAM specification.
pub unsafe extern "C" fn pam_conversation_fn(
    num_msg: c_int,
    msg: *mut *mut pam_message,
    resp: *mut *mut pam_response,
    appdata_ptr: *mut c_void,
) -> c_int {
    if num_msg <= 0 {
        return 0;
    }

    let resps =
        libc::malloc(num_msg as usize * std::mem::size_of::<pam_response>()) as *mut pam_response;
    if resps.is_null() {
        return 4; // PAM_BUF_ERR
    }
    std::ptr::write_bytes(resps, 0, num_msg as usize);

    let password_ptr = appdata_ptr as *const c_char;

    for i in 0..num_msg {
        let msg_ptr = *msg.add(i as usize);
        let msg_style = (*msg_ptr).msg_style;

        if msg_style == PAM_PROMPT_ECHO_OFF || msg_style == PAM_PROMPT_ECHO_ON {
            let resp_str = libc::strdup(password_ptr);
            (*resps.add(i as usize)).resp = resp_str;
            (*resps.add(i as usize)).resp_retcode = 0;
        }
    }

    *resp = resps;
    PAM_SUCCESS
}

/// Verifies user credentials using the system PAM "login" service.
pub fn verify_password(username: &str, password: &str) -> bool {
    let Some(pam) = get_pam() else {
        tracing::error!("PAM dynamic library is not available; password verification aborted");
        return false;
    };

    let (Ok(username_c), Ok(password_c)) = (CString::new(username), CString::new(password)) else {
        return false;
    };

    let conv = pam_conv {
        conv: Some(pam_conversation_fn),
        appdata_ptr: password_c.as_ptr() as *mut c_void,
    };

    let mut pamh: *mut c_void = std::ptr::null_mut();

    unsafe {
        let Ok(service_c) = CString::new("login") else {
            return false;
        };
        let start_status = (pam.pam_start)(service_c.as_ptr(), username_c.as_ptr(), &conv, &mut pamh);

        if start_status != PAM_SUCCESS {
            return false;
        }

        let auth_status = (pam.pam_authenticate)(pamh, 0);
        (pam.pam_end)(pamh, auth_status);

        auth_status == PAM_SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pam_symbols_or_graceful_fallback() {
        // Calling get_pam should never panic; on standard Linux it finds PAM, or returns None cleanly.
        let _pam = get_pam();
    }
}
