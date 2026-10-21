// SPDX-FileCopyrightText: © 2024 Logan Magee
//
// SPDX-License-Identifier: Apache-2.0

use std::{fs::File, mem::ManuallyDrop, os::fd::FromRawFd};

use jni_sys::{JNIEnv, jclass, jlong, jobject};

#[link(name = "nativehelper")]
unsafe extern "C" {
    // available since API 31
    fn AFileDescriptor_getFd(env: *mut JNIEnv, fd: jobject) -> std::ffi::c_int;
}

unsafe fn file_from_java_fd(env: *mut JNIEnv, java_fd: jobject) -> File {
    // SAFETY: The caller guarantees that [env] is a valid JNIEnv pointer for the current thread and
    // that [java_fd] is an open file descriptor
    unsafe {
        let raw_fd = AFileDescriptor_getFd(env, java_fd);
        File::from_raw_fd(raw_fd)
    }
}

// SAFETY: There is no other global function with this name
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_app_accrescent_ina_Ina_patchInner(
    env: *mut JNIEnv,
    _class: jclass,
    old_fd: jobject,
    patch_fd: jobject,
    new_fd: jobject,
) -> jlong {
    // SAFETY: The caller guarantees that [old_fd] is an open file descriptor
    let old_file = ManuallyDrop::new(unsafe { file_from_java_fd(env, old_fd) });
    // SAFETY: The caller guarantees that [patch_fd] is an open file descriptor
    let patch_file = ManuallyDrop::new(unsafe { file_from_java_fd(env, patch_fd) });
    // SAFETY: The caller guarantees that [new_fd] is an open file descriptor
    let mut new_file = ManuallyDrop::new(unsafe { file_from_java_fd(env, new_fd) });

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
    env: *mut JNIEnv,
    _class: jclass,
    src_fd: jobject,
    dst_fd: jobject,
) -> jlong {
    // SAFETY: The caller guarantees that [src_fd] is an open file descriptor
    let src_file = ManuallyDrop::new(unsafe { file_from_java_fd(env, src_fd) });
    // SAFETY: The caller guarantees that [dst_fd] is an open file descriptor
    let mut dst_file = ManuallyDrop::new(unsafe { file_from_java_fd(env, dst_fd) });

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
