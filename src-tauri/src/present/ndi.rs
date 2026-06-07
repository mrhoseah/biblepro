//! NDI sender — wraps the NewTek NDI SDK via raw FFI.
//!
//! Compile with `--features ndi` once the NDI SDK is installed:
//!   Linux:   /usr/lib/libndi.so  (from NDI SDK installer)
//!   Windows: Processing.NDI.Lib.x64.dll  (in PATH)
//!   macOS:   /usr/local/lib/libndi.dylib
//!
//! Without the feature flag the module still compiles; NDI commands
//! return a "not compiled with NDI support" error.

#[cfg(feature = "ndi")]
mod ffi {
    use std::ffi::{c_char, c_int, c_void};

    pub type NDISendInstance = *mut c_void;

    #[repr(C)]
    pub struct NDISendCreate {
        pub p_ndi_name: *const c_char,
        pub p_groups: *const c_char,
        pub clock_video: bool,
        pub clock_audio: bool,
    }

    #[repr(C)]
    pub struct NDIVideoFrameV2 {
        pub xres: c_int,
        pub yres: c_int,
        pub four_cc: u32, // BGRA = 0x41524742
        pub frame_rate_n: c_int,
        pub frame_rate_d: c_int,
        pub picture_aspect_ratio: f32,
        pub frame_format_type: c_int,
        pub timecode: i64,
        pub p_data: *const u8,
        pub line_stride_or_size: c_int,
        pub p_metadata: *const c_char,
        pub timestamp: i64,
    }

    // FourCC for RGBA (NDI calls it RGBX/RGBA)
    pub const FOURCC_RGBA: u32 = 0x41424752; // 'RGBA'
    pub const FRAME_FORMAT_PROGRESSIVE: c_int = 1;

    #[link(name = "ndi")]
    extern "C" {
        pub fn NDIlib_initialize() -> bool;
        pub fn NDIlib_destroy();
        pub fn NDIlib_send_create(p_create_settings: *const NDISendCreate) -> NDISendInstance;
        pub fn NDIlib_send_destroy(p_instance: NDISendInstance);
        pub fn NDIlib_send_send_video_v2(
            p_instance: NDISendInstance,
            p_video_data: *const NDIVideoFrameV2,
        );
    }
}

#[cfg(feature = "ndi")]
use std::ffi::CString;
#[cfg(feature = "ndi")]
use std::sync::{Arc, Mutex};

use super::config::PresentConfig;
use super::renderer::{render_frame, Frame};

// ── public API (always available) ─────────────────────────────────────────────

pub struct NdiSender {
    #[cfg(feature = "ndi")]
    inner: Arc<Mutex<NdiInner>>,
    #[cfg(not(feature = "ndi"))]
    _phantom: (),
}

#[cfg(feature = "ndi")]
struct NdiInner {
    instance: ffi::NDISendInstance,
}

#[cfg(feature = "ndi")]
unsafe impl Send for NdiInner {}
#[cfg(feature = "ndi")]
unsafe impl Sync for NdiInner {}

impl NdiSender {
    #[cfg(feature = "ndi")]
    pub fn start(ndi_name: &str) -> Result<Self, String> {
        let ok = unsafe { ffi::NDIlib_initialize() };
        if !ok {
            return Err("NDIlib_initialize() failed. Is the NDI runtime installed?".to_string());
        }
        let name_c = CString::new(ndi_name).map_err(|e| e.to_string())?;
        let create = ffi::NDISendCreate {
            p_ndi_name: name_c.as_ptr(),
            p_groups: std::ptr::null(),
            clock_video: true,
            clock_audio: false,
        };
        let instance = unsafe { ffi::NDIlib_send_create(&create) };
        if instance.is_null() {
            return Err("NDIlib_send_create() returned null".to_string());
        }
        Ok(Self {
            inner: Arc::new(Mutex::new(NdiInner { instance })),
        })
    }

    #[cfg(not(feature = "ndi"))]
    pub fn start(_ndi_name: &str) -> Result<Self, String> {
        Err("BiblePro was not compiled with NDI support. \
             Add --features ndi and install the NDI SDK."
            .to_string())
    }

    #[cfg(feature = "ndi")]
    pub fn send(&self, frame: &Frame) -> Result<(), String> {
        let inner = self.inner.lock().unwrap();
        // NDI expects BGRA; our renderer produces RGBA — swap R and B.
        let mut bgra: Vec<u8> = Vec::with_capacity(frame.data.len());
        for px in frame.data.chunks(4) {
            bgra.push(px[2]); // B
            bgra.push(px[1]); // G
            bgra.push(px[0]); // R
            bgra.push(px[3]); // A
        }
        let vf = ffi::NDIVideoFrameV2 {
            xres: frame.width as i32,
            yres: frame.height as i32,
            four_cc: ffi::FOURCC_RGBA, // NDI SDK accepts BGRA under this
            frame_rate_n: 30_000,
            frame_rate_d: 1_001,
            picture_aspect_ratio: frame.width as f32 / frame.height as f32,
            frame_format_type: ffi::FRAME_FORMAT_PROGRESSIVE,
            timecode: i64::MIN, // auto
            p_data: bgra.as_ptr(),
            line_stride_or_size: (frame.width * 4) as i32,
            p_metadata: std::ptr::null(),
            timestamp: i64::MIN,
        };
        unsafe { ffi::NDIlib_send_send_video_v2(inner.instance, &vf) };
        Ok(())
    }

    #[cfg(not(feature = "ndi"))]
    pub fn send(&self, _frame: &Frame) -> Result<(), String> {
        Err("NDI not compiled in".to_string())
    }
}

#[cfg(feature = "ndi")]
impl Drop for NdiInner {
    fn drop(&mut self) {
        unsafe {
            ffi::NDIlib_send_destroy(self.instance);
            ffi::NDIlib_destroy();
        }
    }
}

// ── state held in Tauri ───────────────────────────────────────────────────────

use std::sync::Mutex as StdMutex;

pub struct NdiState(pub StdMutex<Option<NdiSender>>);

impl NdiState {
    pub fn new() -> Self {
        Self(StdMutex::new(None))
    }
}

// ── convenience: render + send in one call ────────────────────────────────────

pub fn render_and_send(
    state: &NdiState,
    verse_text: &str,
    reference: &str,
    cfg: &PresentConfig,
) -> Result<Frame, String> {
    let frame = render_frame(verse_text, reference, cfg)?;
    let guard = state.0.lock().unwrap();
    if let Some(sender) = guard.as_ref() {
        sender.send(&frame).ok(); // non-fatal: preview still works without NDI
    }
    Ok(frame)
}
