package ai.cypher.backend.cli

import com.intellij.openapi.util.SystemInfo
import com.intellij.util.system.CpuArch

internal object CypherCliPlatform {
    fun current(): String {
        val os = when {
            SystemInfo.isMac -> "darwin"
            SystemInfo.isLinux -> "linux"
            SystemInfo.isWindows -> "windows"
            else -> throw IllegalStateException("Unsupported OS: ${System.getProperty("os.name")}")
        }
        val arch = when (CpuArch.CURRENT) {
            CpuArch.ARM64 -> "arm64"
            CpuArch.X86_64 -> "x64"
            else -> throw IllegalStateException("Unsupported architecture: ${CpuArch.CURRENT}")
        }
        return "$os-$arch"
    }

    fun exe(): String = if (SystemInfo.isWindows) "cypher.exe" else "cypher"

    fun archive(platform: String = current()): String =
        if (platform.startsWith("linux-")) "tar.gz" else "zip"
}
