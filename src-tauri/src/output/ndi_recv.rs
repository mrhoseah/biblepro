//! NDI receiver — finds sources on the LAN and receives their video frames.
//! Gated behind `#[cfg(feature = "ndi")]`.

use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct NdiSourceInfo {
    pub name: String,
}

/// Raw RGBA pixels received from an NDI source.
#[derive(Clone)]
pub struct ReceivedFrame {
    pub data:   Vec<u8>,
    pub width:  u32,
    pub height: u32,
}

// ── FFI (ndi feature only) ────────────────────────────────────────────────────

#[cfg(feature = "ndi")]
mod ffi {
    use std::ffi::{c_char, c_int, c_uint, c_void};

    pub type NDIFindInstance = *mut c_void;
    pub type NDIRecvInstance = *mut c_void;

    /// Wraps NDIRecvInstance so it can cross thread boundaries.
    pub struct RecvPtr(pub NDIRecvInstance);
    unsafe impl Send for RecvPtr {}

    #[repr(C)]
    pub struct NDISource {
        pub p_ndi_name:    *const c_char,
        pub p_url_address: *const c_char,
    }

    #[repr(C)]
    pub struct NDIFindCreate {
        pub show_local_sources: bool,
        pub p_groups:           *const c_char,
        pub p_extra_ips:        *const c_char,
    }

    /// Layout mirrors NDIlib_recv_create_v3_t which embeds NDIlib_source_t inline.
    #[repr(C)]
    pub struct NDIRecvCreate {
        pub p_ndi_name:         *const c_char, // source.p_ndi_name
        pub p_url_address:      *const c_char, // source.p_url_address
        pub color_format:       c_int,          // 2 = RGBX/RGBA
        pub bandwidth:          c_int,          // 100 = highest
        pub allow_video_fields: bool,
        pub p_ndi_recv_name:    *const c_char,
    }

    #[repr(C)]
    pub struct NDIVideoFrameV2 {
        pub xres:                 c_int,
        pub yres:                 c_int,
        pub four_cc:              u32,
        pub frame_rate_n:         c_int,
        pub frame_rate_d:         c_int,
        pub picture_aspect_ratio: f32,
        pub frame_format_type:    c_int,
        pub timecode:             i64,
        pub p_data:               *mut u8,
        pub line_stride_or_size:  c_int,
        pub p_metadata:           *const c_char,
        pub timestamp:            i64,
    }

    pub const FRAME_TYPE_VIDEO: c_int = 1;

    #[link(name = "ndi")]
    extern "C" {
        pub fn NDIlib_find_create_v3(
            p_create_settings: *const NDIFindCreate,
        ) -> NDIFindInstance;
        pub fn NDIlib_find_wait_for_sources(
            p_instance: NDIFindInstance,
            timeout_in_ms: c_uint,
        ) -> bool;
        pub fn NDIlib_find_get_current_sources(
            p_instance: NDIFindInstance,
            p_no_sources: *mut c_uint,
        ) -> *const NDISource;
        pub fn NDIlib_find_destroy(p_instance: NDIFindInstance);

        pub fn NDIlib_recv_create_v3(
            p_create_settings: *const NDIRecvCreate,
        ) -> NDIRecvInstance;
        pub fn NDIlib_recv_capture_v3(
            p_instance: NDIRecvInstance,
            p_video_data: *mut NDIVideoFrameV2,
            p_audio_data: *mut c_void,
            p_metadata_frame: *mut c_void,
            timeout_in_ms: c_uint,
        ) -> c_int;
        pub fn NDIlib_recv_free_video_v2(
            p_instance: NDIRecvInstance,
            p_video_data: *const NDIVideoFrameV2,
        );
        pub fn NDIlib_recv_destroy(p_instance: NDIRecvInstance);
    }
}

// ── list_ndi_sources ──────────────────────────────────────────────────────────

/// Scan the LAN for NDI sources (blocks up to ~2 s). Always safe to call.
#[cfg(feature = "ndi")]
pub fn list_ndi_sources() -> Vec<NdiSourceInfo> {
    use std::ffi::CStr;
    unsafe {
        let find = ffi::NDIlib_find_create_v3(std::ptr::null());
        if find.is_null() { return vec![]; }
        ffi::NDIlib_find_wait_for_sources(find, 2000);
        let mut count: std::ffi::c_uint = 0;
        let ptr = ffi::NDIlib_find_get_current_sources(find, &mut count);
        let mut out = Vec::with_capacity(count as usize);
        if !ptr.is_null() {
            for i in 0..count as usize {
                let src = &*ptr.add(i);
                if !src.p_ndi_name.is_null() {
                    let name = CStr::from_ptr(src.p_ndi_name).to_string_lossy().into_owned();
                    out.push(NdiSourceInfo { name });
                }
            }
        }
        ffi::NDIlib_find_destroy(find);
        out
    }
}

#[cfg(not(feature = "ndi"))]
pub fn list_ndi_sources() -> Vec<NdiSourceInfo> { vec![] }

// ── NdiReceiver ───────────────────────────────────────────────────────────────

/// Handle to an active NDI receive session.
/// Drop to stop the background capture thread.
pub struct NdiReceiver {
    pub(crate) latest: Arc<Mutex<Option<ReceivedFrame>>>,
    stop:              Arc<AtomicBool>,
    _thread:           std::thread::JoinHandle<()>,
}

impl NdiReceiver {
    pub fn latest_frame(&self) -> Option<ReceivedFrame> {
        self.latest.lock().unwrap().clone()
    }
}

impl Drop for NdiReceiver {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

// ── connect_ndi_source ────────────────────────────────────────────────────────

/// Connect to the named NDI source and start a background capture thread.
/// The latest received frame is always available via `NdiReceiver::latest_frame()`.
#[cfg(feature = "ndi")]
pub fn connect_ndi_source(source_name: &str) -> Result<NdiReceiver, String> {
    use std::ffi::{CStr, CString};

    let want = source_name.to_string();

    // ── find the source on the LAN ─────────────────────────────────────────
    let (name_c, url_c) = unsafe {
        let find = ffi::NDIlib_find_create_v3(std::ptr::null());
        if find.is_null() {
            return Err("NDIlib_find_create_v3 failed".into());
        }
        ffi::NDIlib_find_wait_for_sources(find, 3000);
        let mut count: std::ffi::c_uint = 0;
        let ptr = ffi::NDIlib_find_get_current_sources(find, &mut count);
        let mut found_name: Option<CString> = None;
        let mut found_url:  Option<CString> = None;
        if !ptr.is_null() {
            for i in 0..count as usize {
                let src = &*ptr.add(i);
                if src.p_ndi_name.is_null() { continue; }
                let n = CStr::from_ptr(src.p_ndi_name).to_string_lossy();
                if n == want {
                    found_name = Some(CString::new(n.as_ref()).unwrap());
                    if !src.p_url_address.is_null() {
                        let u = CStr::from_ptr(src.p_url_address).to_string_lossy();
                        found_url = Some(CString::new(u.as_ref()).unwrap());
                    }
                    break;
                }
            }
        }
        ffi::NDIlib_find_destroy(find);
        match found_name {
            Some(n) => (n, found_url),
            None => return Err(format!("NDI source '{}' not found on the network", want)),
        }
    };

    // ── create receiver ────────────────────────────────────────────────────
    let recv_label = CString::new("BiblePro Input").unwrap();
    let create = ffi::NDIRecvCreate {
        p_ndi_name:         name_c.as_ptr(),
        p_url_address:      url_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
        color_format:       2,     // RGBX/RGBA
        bandwidth:          100,   // highest
        allow_video_fields: false,
        p_ndi_recv_name:    recv_label.as_ptr(),
    };
    let recv_raw = unsafe { ffi::NDIlib_recv_create_v3(&create) };
    if recv_raw.is_null() {
        return Err("NDIlib_recv_create_v3 failed".into());
    }
    // NDI SDK copies source info internally; CStrings can be freed now.
    drop(name_c); drop(url_c); drop(recv_label);

    // ── spawn capture thread ───────────────────────────────────────────────
    let latest: Arc<Mutex<Option<ReceivedFrame>>> = Arc::new(Mutex::new(None));
    let latest_t = Arc::clone(&latest);
    let stop     = Arc::new(AtomicBool::new(false));
    let stop_t   = Arc::clone(&stop);
    let recv_ptr = ffi::RecvPtr(recv_raw);

    let thread = std::thread::spawn(move || {
        let recv = recv_ptr; // owned by this thread
        loop {
            if stop_t.load(Ordering::Relaxed) { break; }
            let mut vf = ffi::NDIVideoFrameV2 {
                xres: 0, yres: 0, four_cc: 0,
                frame_rate_n: 0, frame_rate_d: 0,
                picture_aspect_ratio: 0.0, frame_format_type: 0,
                timecode: 0, p_data: std::ptr::null_mut(),
                line_stride_or_size: 0, p_metadata: std::ptr::null(),
                timestamp: 0,
            };
            let ft = unsafe {
                ffi::NDIlib_recv_capture_v3(
                    recv.0,
                    &mut vf,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    33, // ~30 fps timeout
                )
            };
            if ft == ffi::FRAME_TYPE_VIDEO
                && !vf.p_data.is_null()
                && vf.xres > 0 && vf.yres > 0
            {
                let w = vf.xres as u32;
                let h = vf.yres as u32;
                let bytes = (w * h * 4) as usize;
                let mut data = unsafe {
                    std::slice::from_raw_parts(vf.p_data, bytes).to_vec()
                };
                unsafe { ffi::NDIlib_recv_free_video_v2(recv.0, &vf) };
                // Screen capture sources use RGBX (alpha undefined/0).
                // Force alpha = 255 so frame_to_png pre-multiplication is a no-op.
                for px in data.chunks_mut(4) { px[3] = 255; }
                *latest_t.lock().unwrap() = Some(ReceivedFrame { data, width: w, height: h });
            }
        }
        unsafe { ffi::NDIlib_recv_destroy(recv.0) };
    });

    Ok(NdiReceiver { latest, stop, _thread: thread })
}

#[cfg(not(feature = "ndi"))]
pub fn connect_ndi_source(_source_name: &str) -> Result<NdiReceiver, String> {
    Err("BiblePro was not compiled with NDI support. Add --features ndi and install the NDI SDK.".into())
}
