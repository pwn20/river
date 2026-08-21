#[cfg(all(target_os = "windows", feature = "win7"))]
#[allow(non_snake_case)]
mod win32_windows7_hooks
{
    const USE_DELAYED_LOG: bool = false; // Set to true to log all delay-load failures to delayload.log for debugging.

    unsafe extern "system"
    {
        fn LoadLibraryA(lpLibFileName: *const u8) -> *mut std::ffi::c_void;
    }

    type PfnDliHook = unsafe extern "C" fn(
        dliNotify: u32,
        pdli: *const DelayLoadInfo,
    ) -> *mut std::ffi::c_void;

    // 0x00 0x00 0xC3 = xor eax,eax ; ret  (returns 0 in rax, x86_64)
    // A safe-ish "do nothing and return 0" stub.
    //static STUB_CODE: [u8; 3] = [0x31, 0xC0, 0xC3];

    // Mirrors the real DelayLoadProc union from delayimp.h:
    //   typedef struct DelayLoadProc {
    //       BOOL fImportByName;
    //       union { LPCSTR szProcName; DWORD dwOrdinal; };
    //   } DelayLoadProc;
    #[repr(C)]
    pub struct DelayLoadProc {
        pub f_import_by_name: i32,              // 0  (BOOL)
        pub _pad: u32,                          // 4  (alignment padding before the union)
        pub proc_name_or_ordinal: *const i8,    // 8  (LPCSTR if by-name, else low 32 bits = ordinal)
    }

    #[repr(C)]
    pub struct DelayLoadInfo {
        pub cb: u32,                            // 0
        pub _pad1: u32,                         // 4
        pub pidd: *const std::ffi::c_void,      // 8
        pub ppfn: *mut *const std::ffi::c_void, // 16
        pub szDll: *const i8,                   // 24
        pub dlp: DelayLoadProc,                 // 32  (was wrongly split into dwTickCount/lpv)
        pub hmodCur: *mut std::ffi::c_void,     // 48  (was wrongly read as cbAll)
        pub pfnCur: *mut std::ffi::c_void,      // 56  (this is what the old code misread as dlpszProcName!)
        pub dwLastError: u32,                   // 64
        pub _pad2: u32,                         // 68
    }
    // Total size: exactly 72 bytes.

    // Real values from delayimp.h - the old constants (4 and 5) were off by one,
    // which was the actual cause of the crash: see comment on delay_load_failure_hook below.
    //const DLI_FAIL_LOAD_LIB: u32 = 3;
    const DLI_FAIL_GET_PROC: u32 = 4;

    unsafe extern "C" fn stub_fn() -> usize { 0 }

    unsafe extern "C" fn delay_load_failure_hook(
        dliNotify: u32,
        pdli: *const DelayLoadInfo,
    ) -> *mut std::ffi::c_void {
        if pdli.is_null() {
            return std::ptr::null_mut();
        }

        unsafe {
            let info = &*pdli;
            
            let dll = if info.szDll.is_null() {
                "NULL".to_string()
            } else {
                std::ffi::CStr::from_ptr(info.szDll).to_string_lossy().into_owned()
            };
            
            let proc = if info.dlp.f_import_by_name == 0 {
                // Imported by ordinal: the union holds a DWORD ordinal, not a pointer.
                format!("#{}", info.dlp.proc_name_or_ordinal as usize)
            } else if info.dlp.proc_name_or_ordinal.is_null() {
                "NULL".to_string()
            } else {
                std::ffi::CStr::from_ptr(info.dlp.proc_name_or_ordinal)
                    .to_string_lossy()
                    .into_owned()
            };

            let log_msg = format!(
                "notify={} cb={} dll={} proc={}\n",
                dliNotify, info.cb, dll, proc
            );

            if USE_DELAYED_LOG
            {
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("Y:/Documents/Programming/Rust/river/delayload.log")
                {
                    use std::io::Write;
                    let _ = f.write_all(log_msg.as_bytes());
                    let _ = f.flush(); 
                }
            }

            // 1. If a specific function is missing from a DLL that DID load (dliFailGetProc),
            // return our no-op stub AS THE FUNCTION POINTER ITSELF. This is the common case
            // on Win7: shcore.dll/combase.dll exist but lack the newer export.
            if dliNotify == DLI_FAIL_GET_PROC {
                return stub_fn as *mut std::ffi::c_void;
            }

            // 2. If the DLL itself failed to load (dliFailLoadLib), return a valid HMODULE
            // so the delay-load helper proceeds to GetProcAddress on it (which will then
            // fail and route back through case 1 above, producing our safe stub).
            //
            // NOTE: previously DLI_FAIL_LOAD_LIB/DLI_FAIL_GET_PROC were off-by-one (4/5
            // instead of the real delayimp.h values 3/4), so real dliFailGetProc (4) events
            // fell through to here and got a raw HMODULE handed back as if it were a FARPROC.
            // Calling that "function" meant jumping into kernel32.dll's PE header bytes as
            // machine code - an almost-certain crash the moment any such import was actually
            // invoked (which is why the app ran fine right up until it needed to use one).
            return LoadLibraryA(b"kernel32.dll\0".as_ptr());
        }
    }

    #[unsafe(no_mangle)]
    pub static mut __pfnDliFailureHook2: PfnDliHook = delay_load_failure_hook;
}