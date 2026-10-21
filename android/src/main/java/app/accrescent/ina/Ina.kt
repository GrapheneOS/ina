// SPDX-FileCopyrightText: © 2024 Logan Magee
//
// SPDX-License-Identifier: Apache-2.0

package app.accrescent.ina

import android.system.ErrnoException
import java.io.FileDescriptor
import java.io.IOException

/**
 * Bindings for ina's native patcher and brotli decompressor.
 */
public class Ina {
    /**
     * Entry points.
     */
    public companion object {
        init {
            System.loadLibrary("ina")
        }

        /**
         * Patches an old file given an Ina patch file. [old], [patch] and [new] file descriptors
         * remain owned by the caller. Returns the number of bytes written to [new].
         */
        @JvmStatic
        @Throws(IOException::class)
        public fun patch(old: FileDescriptor, patch: FileDescriptor, new: FileDescriptor): Long {
            val ret = patchInner(old, patch, new)
            if (ret >= 0) {
                return ret
            }
            throwErrorAsIoException(-ret, "ina_patch")
        }

        @JvmStatic
        private external fun patchInner(old: FileDescriptor, patch: FileDescriptor, new: FileDescriptor): Long

        @Throws(IOException::class)
        private fun throwErrorAsIoException(err: Long, functionName: String): Nothing {
            val errCode = err shr 32
            if (errCode != 0L) {
                throw IOException("$functionName error $errCode")
            }
            val ee = ErrnoException(functionName, err.toInt())
            throw ee.rethrowAsIOException()
        }
    }
}
