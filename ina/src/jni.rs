// SPDX-FileCopyrightText: © 2024 Logan Magee
//
// SPDX-License-Identifier: Apache-2.0

use std::{fs::File, mem::ManuallyDrop, os::fd::FromRawFd};

use jni::{
    JNIEnv,
    objects::{JClass, JObject},
    sys::jlong,
};

// android/file_descriptor_jni.h, API 31
#[link(name = "nativehelper")]
unsafe extern "C" {
    fn AFileDescriptor_getFd(env: *mut jni::sys::JNIEnv, fd: jni::sys::jobject) -> std::ffi::c_int;
}

unsafe fn file_from_java_fd(env: &JNIEnv, java_fd: JObject) -> File {
    // SAFETY: The caller guarantees that [java_fd] is an open file descriptor
    unsafe {
        let raw_fd = AFileDescriptor_getFd(env.get_raw(), java_fd.as_raw());
        File::from_raw_fd(raw_fd)
    }
}

// SAFETY: There is no other global function with this name
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_app_accrescent_ina_Ina_patchInner(
    env: JNIEnv,
    _class: JClass,
    old_fd: JObject,
    patch_fd: JObject,
    new_fd: JObject,
) -> jlong {
    // SAFETY: The caller guarantees that [old_fd] is an open file descriptor
    let old_file = ManuallyDrop::new(unsafe { file_from_java_fd(&env, old_fd) });
    // SAFETY: The caller guarantees that [patch_fd] is an open file descriptor
    let patch_file = ManuallyDrop::new(unsafe { file_from_java_fd(&env, patch_fd) });
    // SAFETY: The caller guarantees that [new_fd] is an open file descriptor
    let mut new_file = ManuallyDrop::new(unsafe { file_from_java_fd(&env, new_fd) });

    match crate::patch(&*old_file, &*patch_file, &mut *new_file) {
        Ok(num_bytes) => {
            if num_bytes > i64::MAX as u64 {
                -(1 << 32)
            } else {
                num_bytes as jlong
            }
        }
        Err(error) => match error {
            crate::PatchError::Io(io_err) => {
                if let Some(errno) = io_err.raw_os_error()
                    && errno > 0
                {
                    -errno as jlong
                } else {
                    -(2 << 32)
                }
            }
            crate::PatchError::BadMagic(_) => -(3 << 32),
            crate::PatchError::UnsupportedVersion(_) => -(4 << 32),
        },
    }
}

// SAFETY: There is no other global function with this name
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_app_accrescent_ina_Ina_unbrotliInner(
    env: JNIEnv,
    _class: JClass,
    src_fd: JObject,
    dst_fd: JObject,
) -> jlong {
    // SAFETY: The caller guarantees that [src_fd] is an open file descriptor
    let src_file = ManuallyDrop::new(unsafe { file_from_java_fd(&env, src_fd) });
    // SAFETY: The caller guarantees that [dst_fd] is an open file descriptor
    let mut dst_file = ManuallyDrop::new(unsafe { file_from_java_fd(&env, dst_fd) });

    let mut decompressor = brotli_decompressor::Decompressor::new(&*src_file, 64 * 1024);
    match std::io::copy(&mut decompressor, &mut *dst_file) {
        Ok(num_bytes) => {
            if num_bytes > i64::MAX as u64 {
                -(1 << 32)
            } else {
                num_bytes as jlong
            }
        }
        Err(error) => {
            if let Some(errno) = error.raw_os_error()
                && errno > 0
            {
                -errno as jlong
            } else {
                -(2 << 32)
            }
        }
    }
}
