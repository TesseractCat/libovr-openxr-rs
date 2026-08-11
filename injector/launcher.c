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
    if (argc != 2) return 64;

    wchar_t bypass_path[MAX_PATH];
    DWORD length = GetModuleFileNameW(NULL, bypass_path, MAX_PATH);
    if (!length || length == MAX_PATH) return 64;
    wchar_t *filename = wcsrchr(bypass_path, L'\\');
    if (!filename) return 64;
    if (_snwprintf(filename + 1, MAX_PATH - (filename + 1 - bypass_path),
                   L"ovr-loader-bypass.dll") < 0) return 64;

    wchar_t command[32768];
    if (_snwprintf(command, 32768, L"\"%s\"", argv[1]) < 0) return 64;

    STARTUPINFOW startup = { .cb = sizeof(startup) };
    PROCESS_INFORMATION process = {0};
    if (!CreateProcessW(argv[1], command, NULL, NULL, FALSE,
                        CREATE_SUSPENDED, NULL, NULL, &startup, &process)) return 65;

    if (!remote_load_library(process.hProcess, bypass_path)) {
        TerminateProcess(process.hProcess, 67);
        return 67;
    }

    ResumeThread(process.hThread);
    CloseHandle(process.hThread);
    WaitForSingleObject(process.hProcess, INFINITE);
    DWORD exit_code = 1;
    GetExitCodeProcess(process.hProcess, &exit_code);
    CloseHandle(process.hProcess);
    return (int)exit_code;
}
