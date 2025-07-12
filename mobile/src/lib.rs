use metamesh_identity::{generate_seed_identity, recover_from_mnemonic};
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

#[repr(C)]
pub struct CSeedIdentity {
    pub mnemonic: *mut c_char,
    pub private_key: *mut c_char,
    pub public_key: *mut c_char,
    pub seed_id: *mut c_char,
}

#[no_mangle]
pub extern "C" fn metamesh_generate_identity() -> *mut CSeedIdentity {
    let identity = generate_seed_identity();
    
    Box::into_raw(Box::new(CSeedIdentity {
        mnemonic: CString::new(identity.mnemonic).unwrap().into_raw(),
        private_key: CString::new(identity.private_key).unwrap().into_raw(),
        public_key: CString::new(identity.public_key).unwrap().into_raw(),
        seed_id: CString::new(identity.seed_id).unwrap().into_raw(),
    }))
}

#[no_mangle]
pub extern "C" fn metamesh_recover_identity(sentence: *const c_char) -> *mut CSeedIdentity {
    let c_str = unsafe { CStr::from_ptr(sentence) };
    let sentence_str = c_str.to_str().unwrap();
    let identity = recover_from_mnemonic(sentence_str);
    
    Box::into_raw(Box::new(CSeedIdentity {
        mnemonic: CString::new(identity.mnemonic).unwrap().into_raw(),
        private_key: CString::new(identity.private_key).unwrap().into_raw(),
        public_key: CString::new(identity.public_key).unwrap().into_raw(),
        seed_id: CString::new(identity.seed_id).unwrap().into_raw(),
    }))
}

#[no_mangle]
pub extern "C" fn metamesh_free_identity(identity: *mut CSeedIdentity) {
    if !identity.is_null() {
        unsafe {
            let identity = Box::from_raw(identity);
            let _ = CString::from_raw(identity.mnemonic);
            let _ = CString::from_raw(identity.private_key);
            let _ = CString::from_raw(identity.public_key);
            let _ = CString::from_raw(identity.seed_id);
        }
    }
}