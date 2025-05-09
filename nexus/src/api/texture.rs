//! Texture loading.

use crate::{
    util::{path_to_c, str_to_c},
    AddonApi, TextureApi,
};
use std::{
    ffi::{c_char, c_void},
    mem,
    path::Path,
};
use windows::Win32::{Foundation::HMODULE, Graphics::Direct3D11::ID3D11ShaderResourceView};

/// A loaded texture.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(C)]
pub struct Texture {
    /// Width of the texture.
    pub width: u32,

    /// Height of the texture.
    pub height: u32,

    /// Shader resource view of the texture.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub resource: Option<ID3D11ShaderResourceView>,
}

impl Texture {
    /// Returns the associated resource as raw pointer.
    #[inline]
    pub fn resource_ptr(&self) -> *const c_void {
        // ShaderResourceView is a IUnknown, which is is a NonNull<c_void>
        unsafe { mem::transmute_copy::<Option<ID3D11ShaderResourceView>, *const c_void>(&self.resource) }
    }

    /// Returns the associated [`imgui::TextureId`].
    #[inline]
    pub fn id(&self) -> imgui::TextureId {
        self.resource_ptr().into()
    }

    /// Returns the original texture size in [`imgui`] format.
    #[inline]
    pub fn size(&self) -> [f32; 2] {
        [self.width as f32, self.height as f32]
    }

    /// Returns a resized texture size in [`imgui`] format.
    #[inline]
    pub fn size_resized(&self, factor: f32) -> [f32; 2] {
        let [x, y] = self.size();
        [factor * x, factor * y]
    }
}

pub type RawTextureReceiveCallback =
    extern "C-unwind" fn(identifier: *const c_char, texture: *const Texture);

pub type RawTextureGet = unsafe extern "C-unwind" fn(identifier: *const c_char) -> *const Texture;

pub type RawTextureGetOrCreateFromFile = unsafe extern "C-unwind" fn(
    identifier: *const c_char,
    filename: *const c_char,
) -> *const Texture;

pub type RawTextureGetOrCreateFromResource = unsafe extern "C-unwind" fn(
    identifier: *const c_char,
    resource_id: u32,
    module: HMODULE,
) -> *const Texture;

pub type RawTextureGetOrCreateFromUrl = unsafe extern "C-unwind" fn(
    identifier: *const c_char,
    remote: *const c_char,
    endpoint: *const c_char,
) -> *const Texture;

pub type RawTextureGetOrCreateFromMemory = unsafe extern "C-unwind" fn(
    identifier: *const c_char,
    data: *const c_void,
    size: usize,
) -> *const Texture;

pub type RawTextureLoadFromFile = unsafe extern "C-unwind" fn(
    identifier: *const c_char,
    filename: *const c_char,
    callback: RawTextureReceiveCallback,
);

pub type RawTextureLoadFromResource = unsafe extern "C-unwind" fn(
    identifier: *const c_char,
    resource_id: u32,
    module: HMODULE,
    callback: RawTextureReceiveCallback,
);

pub type RawTextureLoadFromUrl = unsafe extern "C-unwind" fn(
    identifier: *const c_char,
    remote: *const c_char,
    endpoint: *const c_char,
    callback: RawTextureReceiveCallback,
);

pub type RawTextureLoadFromMemory = unsafe extern "C-unwind" fn(
    identifier: *const c_char,
    data: *const c_void,
    size: usize,
    callback: RawTextureReceiveCallback,
);

/// Attempts to retrieve a texture by its identifier.
pub fn get_texture(identifier: impl AsRef<str>) -> Option<Texture> {
    let TextureApi { get, .. } = AddonApi::get().texture;
    let identifier = str_to_c(identifier, "failed to convert texture identifier");
    unsafe { get(identifier.as_ptr()).as_ref().cloned() }
}

/// Attempts to retrieve a texture or creates it from the given file path.
pub fn get_texture_or_create_from_file(
    identifier: impl AsRef<str>,
    file: impl AsRef<Path>,
) -> Option<Texture> {
    let TextureApi {
        get_or_create_from_file,
        ..
    } = AddonApi::get().texture;
    let identifier = str_to_c(identifier, "failed to convert texture identifier");
    let file = path_to_c(file, "failed to convert texture file");
    unsafe {
        get_or_create_from_file(identifier.as_ptr(), file.as_ptr())
            .as_ref()
            .cloned()
    }
}

/// Attempts to retrieve a texture or creates it from the given resource.
pub fn get_texture_or_create_from_resource(
    identifier: impl AsRef<str>,
    resource_id: u32,
    module: HMODULE,
) -> Option<Texture> {
    let TextureApi {
        get_or_create_from_resource,
        ..
    } = AddonApi::get().texture;
    let identifier = str_to_c(identifier, "failed to convert texture identifier");
    unsafe {
        get_or_create_from_resource(identifier.as_ptr(), resource_id, module)
            .as_ref()
            .cloned()
    }
}

/// Attempts to retrieve a texture or creates it from the given URL.
pub fn get_texture_or_create_from_url(
    identifier: impl AsRef<str>,
    remote: impl AsRef<str>,
    endpoint: impl AsRef<str>,
) -> Option<Texture> {
    let TextureApi {
        get_or_create_from_url,
        ..
    } = AddonApi::get().texture;
    let identifier = str_to_c(identifier, "failed to convert texture identifier");
    let remote = str_to_c(remote, "failed to convert texture url remote");
    let endpoint = str_to_c(endpoint, "failed to convert texture url endpoint");
    unsafe {
        get_or_create_from_url(identifier.as_ptr(), remote.as_ptr(), endpoint.as_ptr())
            .as_ref()
            .cloned()
    }
}

/// Attempts to retrieve a texture or creates it from the given memory.
pub fn get_texture_or_create_from_memory(
    identifier: impl AsRef<str>,
    memory: impl AsRef<[u8]>,
) -> Option<Texture> {
    let TextureApi {
        get_or_create_from_memory,
        ..
    } = AddonApi::get().texture;
    let identifier = str_to_c(identifier, "failed to convert texture identifier");
    let memory = memory.as_ref();
    unsafe {
        get_or_create_from_memory(identifier.as_ptr(), memory.as_ptr().cast(), memory.len())
            .as_ref()
            .cloned()
    }
}

/// Loads a texture from the given file path.
///
/// You can create a [`RawTextureReceiveCallback`] using the [`texture_receive`] macro.
pub fn load_texture_from_file(
    identifier: impl AsRef<str>,
    file: impl AsRef<Path>,
    callback: Option<RawTextureReceiveCallback>,
) {
    let TextureApi { load_from_file, .. } = AddonApi::get().texture;
    let identifier = str_to_c(identifier, "failed to convert texture identifier");
    let file = path_to_c(file, "foo");
    unsafe {
        load_from_file(
            identifier.as_ptr(),
            file.as_ptr(),
            callback.unwrap_or(dummy_receive_texture),
        )
    }
}

/// Loads a texture from the given resource.
///
/// You can create a [`RawTextureReceiveCallback`] using the [`texture_receive`] macro.
pub fn load_texture_from_resource(
    identifier: impl AsRef<str>,
    resource_id: u32,
    module: HMODULE,
    callback: Option<RawTextureReceiveCallback>,
) {
    let TextureApi {
        load_from_resource, ..
    } = AddonApi::get().texture;
    let identifier = str_to_c(identifier, "failed to convert texture identifier");
    unsafe {
        load_from_resource(
            identifier.as_ptr(),
            resource_id,
            module,
            callback.unwrap_or(dummy_receive_texture),
        )
    }
}

/// Loads a texture from the given URL.
///
/// You can create a [`RawTextureReceiveCallback`] using the [`texture_receive`] macro.
///
/// # Usage
/// ```no_run
/// # use nexus::texture::*;
/// # extern "C-unwind" fn receive_texture(_identifier: *const std::ffi::c_char, _texture: *const Texture) {}
/// load_texture_from_url(
///     "TEX_DUNGEON_ICON",
///     "https://render.guildwars2.com",
///     "/file/943538394A94A491C8632FBEF6203C2013443555/102478.png",
///     Some(receive_texture),
/// )
/// ```
pub fn load_texture_from_url(
    identifier: impl AsRef<str>,
    remote: impl AsRef<str>,
    endpoint: impl AsRef<str>,
    callback: Option<RawTextureReceiveCallback>,
) {
    let TextureApi { load_from_url, .. } = AddonApi::get().texture;
    let identifier = str_to_c(identifier, "failed to convert texture identifier");
    let remote = str_to_c(remote, "failed to convert texture url remote");
    let endpoint = str_to_c(endpoint, "failed to convert texture url endpoint");
    unsafe {
        load_from_url(
            identifier.as_ptr(),
            remote.as_ptr(),
            endpoint.as_ptr(),
            callback.unwrap_or(dummy_receive_texture),
        )
    }
}

/// Loads a texture from the given memory.
/// ///
/// You can create a [`RawTextureReceiveCallback`] using the [`texture_receive`] macro.
pub fn load_texture_from_memory(
    identifier: impl AsRef<str>,
    data: impl AsRef<[u8]>,
    callback: Option<RawTextureReceiveCallback>,
) {
    let TextureApi {
        load_from_memory, ..
    } = AddonApi::get().texture;
    let identifier = str_to_c(identifier, "failed to convert texture identifier");
    let data = data.as_ref();
    unsafe {
        load_from_memory(
            identifier.as_ptr(),
            data.as_ptr().cast(),
            data.len(),
            callback.unwrap_or(dummy_receive_texture),
        )
    }
}

extern "C-unwind" fn dummy_receive_texture(_identifier: *const c_char, _texture: *const Texture) {}

/// Macro to wrap a texture receive callback.
///
/// Generates a [`RawTextureReceiveCallback`] wrapper around the passed callback.
///
/// # Usage
/// ```no_run
/// # use nexus::texture::*;
/// use nexus::log::{log, LogLevel};
/// let texture_receive: RawTextureReceiveCallback = texture_receive!(|id, _texture| {
///     log(LogLevel::Info, "My Addon", format!("texture {id} loaded"));
/// });
/// load_texture_from_file("MY_TEXTURE", r"C:\path\to\texture.png", Some(texture_receive));
/// ```
#[macro_export]
macro_rules! texture_receive {
    ( $callback:expr $(,)? ) => {{
        const __CALLBACK: fn(&::std::primitive::str, Option<&$crate::texture::Texture>) = $callback;

        extern "C-unwind" fn __keybind_callback_wrapper(
            identifier: *const ::std::ffi::c_char,
            texture: *const $crate::texture::Texture,
        ) {
            let identifier = unsafe { $crate::__macro::str_from_c(identifier) }
                .expect("invalid identifier in texture callback");
            let texture = unsafe { texture.as_ref() };
            __CALLBACK(identifier, texture)
        }

        __keybind_callback_wrapper
    }};
}

pub use texture_receive;
