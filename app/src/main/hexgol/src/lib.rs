#![allow(nonstandard_style)]

#[cfg(target_arch = "x86_64")]
mod ffi {
    pub mod ffi_x86_64;
    pub use ffi_x86_64::*;
}

#[cfg(target_arch = "aarch64")]
mod ffi {
    pub mod ffi_aarch64;
    pub use ffi_aarch64::*;
}

use ffi::*;

mod game;
use game::*;
mod renderer;
use renderer::*;

use std::ffi::c_void;
use std::ptr::addr_of_mut;

struct Renderer {
    hex_instanced: InstancedMesh,
    gfx: Graphics,
}

use raw_window_handle::*;
unsafe impl HasRawWindowHandle for android_app {
    fn raw_window_handle(&self) -> RawWindowHandle {
        unsafe {
            let mut handle = AndroidNdkWindowHandle::empty();
            handle.a_native_window = std::mem::transmute(self.window);
            RawWindowHandle::AndroidNdk(handle)
        }
    }
}
unsafe impl HasRawDisplayHandle for android_app {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::Android(AndroidDisplayHandle::empty())
    }
}

unsafe extern "C" fn command(app: *mut android_app, cmd: i32) {
    match cmd as u32 {
        NativeAppGlueAppCmd_APP_CMD_INIT_WINDOW => {
            let gfx = pollster::block_on(Graphics::new(
                [
                    anativewindow_getwidth((*app).window) as u32,
                    anativewindow_getheight((*app).window) as u32,
                ],
                &*app,
            ));

            let hex = MeshBuilder::new_hexagon([0.0, 0.0], 1.0).build(gfx.context());
            let hex_instanced = InstancedMesh::new(hex, gfx.context(), &[]);
            let renderer = Box::new(Renderer { gfx, hex_instanced });

            (*app).userData = std::mem::transmute(Box::leak(renderer));
        }
        NativeAppGlueAppCmd_APP_CMD_TERM_WINDOW => {
            if !(*app).userData.is_null() {
                let renderer: *mut Renderer = std::mem::transmute((*app).userData);
                let _renderer = Box::from_raw(renderer);

                (*app).userData = std::ptr::null_mut();
            }
        }
        NativeAppGlueAppCmd_APP_CMD_WINDOW_RESIZED => {
            if !(*app).userData.is_null() {
                let renderer: *mut Renderer = std::mem::transmute((*app).userData);

                (*renderer).gfx.resize([
                    anativewindow_getwidth((*app).window) as u32,
                    anativewindow_getheight((*app).window) as u32,
                ]);
            }
        }
        _ => {}
    }
}

unsafe fn alooper_pollall(
    timeout: i32,
    out_fd: *mut i32,
    out_event: *mut i32,
    out_data: *mut *mut c_void,
) -> i32 {
    ndk_sys::ALooper_pollAll(timeout, out_fd, out_event, out_data)
}

unsafe fn anativewindow_getwidth(window: *mut ndk_sys::ANativeWindow) -> i32 {
    ndk_sys::ANativeWindow_getWidth(window)
}
unsafe fn anativewindow_getheight(window: *mut ndk_sys::ANativeWindow) -> i32 {
    ndk_sys::ANativeWindow_getHeight(window)
}

const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

#[no_mangle]
pub unsafe extern "C" fn android_main(app: *mut android_app) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("Rust")
            .with_min_level(log::Level::Info),
    );

    let mut game = HexGOL::new(35);
    game.randomize();

    let tps = std::time::Duration::from_secs_f32(1.0 / 15.0);
    let mut timer = std::time::Instant::now();

    (*app).onAppCmd = Some(command);

    let mut events: i32 = std::mem::uninitialized();
    let mut poll_source: *mut android_poll_source = std::mem::uninitialized();

    loop {
        if alooper_pollall(
            0,
            std::ptr::null_mut(),
            addr_of_mut!(events),
            std::mem::transmute(addr_of_mut!(poll_source)),
        ) >= 0
        {
            if !poll_source.is_null() {
                (*poll_source).process.unwrap()(app, poll_source);
            }
        }

        if (*app).destroyRequested > 0 {
            break;
        }

        if timer.elapsed() > tps {
            timer = std::time::Instant::now();
            game.update();

            if !(*app).userData.is_null() {
                let renderer: *mut Renderer = std::mem::transmute((*app).userData);

                (*renderer).gfx.update();

                let mut instances = vec![];
                for (hex, cell) in game.iter() {
                    if *cell {
                        instances.push(Instance::new(
                            HexFract::from(*hex).transform(1.0),
                            [1.0, 1.0],
                            WHITE,
                        ));
                    }
                }
                (*renderer)
                    .hex_instanced
                    .update((*renderer).gfx.context(), &instances);

                let mut render_pass = (*renderer).gfx.start_frame();
                (*renderer).hex_instanced.draw(&mut render_pass);
                drop(render_pass);
                (*renderer).gfx.end_frame();
            }
        }
    }
}

use jni::sys::*;

// Rust doesn't give us a clean way to directly export symbols from C/C++
// so we rename the C/C++ symbols and re-export these JNI entrypoints from
// Rust...
//
// https://github.com/rust-lang/rfcs/issues/2771
extern "C" {
    pub fn Java_com_google_androidgamesdk_GameActivity_loadNativeCode_C(
        env: *mut JNIEnv,
        javaGameActivity: jobject,
        path: jstring,
        funcName: jstring,
        internalDataDir: jstring,
        obbDir: jstring,
        externalDataDir: jstring,
        jAssetMgr: jobject,
        savedState: jbyteArray,
    ) -> jlong;

    pub fn GameActivity_onCreate_C(
        activity: *mut GameActivity,
        savedState: *mut ::std::os::raw::c_void,
        savedStateSize: libc::size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Java_com_google_androidgamesdk_GameActivity_loadNativeCode(
    env: *mut JNIEnv,
    java_game_activity: jobject,
    path: jstring,
    func_name: jstring,
    internal_data_dir: jstring,
    obb_dir: jstring,
    external_data_dir: jstring,
    jasset_mgr: jobject,
    saved_state: jbyteArray,
) -> jlong {
    Java_com_google_androidgamesdk_GameActivity_loadNativeCode_C(
        env,
        java_game_activity,
        path,
        func_name,
        internal_data_dir,
        obb_dir,
        external_data_dir,
        jasset_mgr,
        saved_state,
    )
}

#[no_mangle]
pub unsafe extern "C" fn GameActivity_onCreate(
    activity: *mut GameActivity,
    saved_state: *mut std::os::raw::c_void,
    saved_state_size: libc::size_t,
) {
    GameActivity_onCreate_C(activity, saved_state, saved_state_size);
}
