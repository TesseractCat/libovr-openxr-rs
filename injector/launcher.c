#include <windows.h>
#include <wchar.h>

static int remote_load_library(HANDLE process, const wchar_t *path) {
    SIZE_T bytes = (wcslen(path) + 1) * sizeof(*path);
    void *remote_path = VirtualAllocEx(process, NULL, bytes,
                                       MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if (!remote_path || !WriteProcessMemory(process, remote_path, path, bytes, NULL)) return 0;

    HMODULE kernel32 = GetModuleHandleW(L"kernel32.dll");
    FARPROC load_library = GetProcAddress(kernel32, "LoadLibraryW");
    HANDLE thread = CreateRemoteThread(process, NULL, 0,
        (LPTHREAD_START_ROUTINE)load_library, remote_path, 0, NULL);
    if (!thread || WaitForSingleObject(thread, INFINITE) != WAIT_OBJECT_0) return 0;
    DWORD module = 0;
    GetExitCodeThread(thread, &module);
    CloseHandle(thread);
    return module != 0;
}

int wmain(int argc, wchar_t **argv) {
    // The launcher discovers adjacent echovr.exe itself; remaining arguments
    // are forwarded as game arguments (for example map/game-type overrides).

    wchar_t launcher_path[MAX_PATH];
    DWORD length = GetModuleFileNameW(NULL, launcher_path, MAX_PATH);
    if (!length || length == MAX_PATH) return 64;
    wchar_t *filename = wcsrchr(launcher_path, L'\\');
    if (!filename) return 64;
    size_t directory_length = (size_t)(filename + 1 - launcher_path);

    wchar_t bypass_path[MAX_PATH], runtime_path[MAX_PATH], platform_path[MAX_PATH], game_path[MAX_PATH];
    if (_snwprintf(game_path, MAX_PATH, L"%.*sechovr.exe",
                   (int)directory_length, launcher_path) < 0 ||
        _snwprintf(bypass_path, MAX_PATH, L"%.*sovr-loader-bypass.dll",
                   (int)directory_length, launcher_path) < 0 ||
        _snwprintf(runtime_path, MAX_PATH, L"%.*sLibOVRRT64_1.dll",
                   (int)directory_length, launcher_path) < 0 ||
        _snwprintf(platform_path, MAX_PATH, L"%.*sLibOVRPlatform64_1.dll",
                   (int)directory_length, launcher_path) < 0) return 64;

    wchar_t command[32768];
    int command_length = _snwprintf(command, 32768, L"\"%s\"", game_path);
    if (command_length < 0 || command_length >= 32768) return 64;
    for (int index = 1; index < argc; ++index) {
        int written = _snwprintf(command + command_length, 32768 - command_length,
                                 L" \"%s\"", argv[index]);
        if (written < 0 || written >= 32768 - command_length) return 64;
        command_length += written;
    }

    STARTUPINFOW startup = { .cb = sizeof(startup) };
    PROCESS_INFORMATION process = {0};
    if (!CreateProcessW(game_path, command, NULL, NULL, FALSE,
                        CREATE_SUSPENDED, NULL, NULL, &startup, &process)) return 65;

    // Preload the adjacent shim under the exact two LibOVR module names before
    // Echo's registry-based loader starts. Windows/Wine module de-duplication
    // then satisfies later absolute-path loads with these process-local images.
    if (!remote_load_library(process.hProcess, runtime_path) ||
        !remote_load_library(process.hProcess, platform_path)) {
        TerminateProcess(process.hProcess, 67);
        return 67;
    }
    if (!remote_load_library(process.hProcess, bypass_path)) {
        TerminateProcess(process.hProcess, 68);
        return 68;
    }

    ResumeThread(process.hThread);
    CloseHandle(process.hThread);
    WaitForSingleObject(process.hProcess, INFINITE);
    DWORD exit_code = 1;
    GetExitCodeProcess(process.hProcess, &exit_code);
    CloseHandle(process.hProcess);
    return (int)exit_code;
}
