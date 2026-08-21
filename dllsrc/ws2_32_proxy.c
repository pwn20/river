// Strictly block Windows from loading default socket headers
#define WIN32_LEAN_AND_MEAN
#define _WINSOCKAPI_
#define _WINSOCK2API_
#include <windows.h>

#ifndef WSAAPI
#define WSAAPI __stdcall
#endif

#ifndef SOCKET_ERROR
#define SOCKET_ERROR (-1)
#endif

// Manually define necessary types so we don't need winsock.h
typedef UINT_PTR SOCKET;
typedef unsigned long u_long;
typedef unsigned short u_short;

static HMODULE hRealWs2 = NULL;

BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved)
{
    (void)hinstDLL;
    (void)lpvReserved;

    if (fdwReason == DLL_PROCESS_ATTACH)
    {
        DisableThreadLibraryCalls(hinstDLL);

        WCHAR sysPath[MAX_PATH];
        if (GetSystemDirectoryW(sysPath, MAX_PATH) > 0)
        {
            lstrcatW(sysPath, L"\\ws2_32.dll");
            hRealWs2 = LoadLibraryW(sysPath);
        }
    }
    return TRUE;
}

#define FORWARD(ret, name, params, args) \
    typedef ret (WSAAPI *PFN_##name) params; \
    static PFN_##name pfn_##name = NULL; \
    __declspec(dllexport) ret WSAAPI name params \
    { \
        if (!pfn_##name && hRealWs2) pfn_##name = (PFN_##name)GetProcAddress(hRealWs2, #name); \
        if (pfn_##name) return pfn_##name args; \
        return (ret)0; \
    }

// Standard Winsock Exports mapped dynamically
FORWARD(int, WSAStartup, (WORD wVer, void *lpWSAData), (wVer, lpWSAData))
FORWARD(int, WSACleanup, (void), ())
FORWARD(int, WSAGetLastError, (void), ())
FORWARD(int, __WSAFDIsSet, (SOCKET fd, void *set), (fd, set))
FORWARD(int, getaddrinfo, (const char *node, const char *service, const void *hints, void **res), (node, service, hints, res))
FORWARD(void, freeaddrinfo, (void *ai), (ai))
FORWARD(int, gethostname, (char *name, int namelen), (name, namelen))
FORWARD(SOCKET, socket, (int af, int type, int protocol), (af, type, protocol))
FORWARD(int, closesocket, (SOCKET s), (s))
FORWARD(int, bind, (SOCKET s, const void *addr, int namelen), (s, addr, namelen))
FORWARD(int, connect, (SOCKET s, const void *name, int namelen), (s, name, namelen))
FORWARD(int, send, (SOCKET s, const char *buf, int len, int flags), (s, buf, len, flags))
FORWARD(int, recv, (SOCKET s, char *buf, int len, int flags), (s, buf, len, flags))
FORWARD(int, select, (int nfds, void *rfds, void *wfds, void *efds, const void *timeout), (nfds, rfds, wfds, efds, timeout))
FORWARD(int, ioctlsocket, (SOCKET s, long cmd, u_long *argp), (s, cmd, argp))
FORWARD(int, setsockopt, (SOCKET s, int level, int optname, const char *optval, int optlen), (s, level, optname, optval, optlen))
FORWARD(int, getsockopt, (SOCKET s, int level, int optname, char *optval, int *optlen), (s, level, optname, optval, optlen))
FORWARD(int, getsockname, (SOCKET s, void *name, int *namelen), (s, name, namelen))
FORWARD(int, getpeername, (SOCKET s, void *name, int *namelen), (s, name, namelen))
FORWARD(u_short, htons, (u_short hostshort), (hostshort))
FORWARD(u_long, htonl, (u_long hostlong), (hostlong))
FORWARD(u_short, ntohs, (u_short netshort), (netshort))
FORWARD(u_long, ntohl, (u_long netlong), (netlong))

typedef int (WSAAPI *PFN_gethostname_raw)(char *name, int namelen);
static PFN_gethostname_raw pfn_gethostname_raw = NULL;

__declspec(dllexport) int WSAAPI GetHostNameW(PWSTR name, int namelen)
{
    if (!name || namelen <= 0)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return SOCKET_ERROR;
    }

    if (!pfn_gethostname_raw && hRealWs2)
    {
        pfn_gethostname_raw = (PFN_gethostname_raw)GetProcAddress(hRealWs2, "gethostname");
    }
    
    if (!pfn_gethostname_raw)
    {
        return SOCKET_ERROR;
    }

    char ansiBuf[256] = {0};
    int res = pfn_gethostname_raw(ansiBuf, sizeof(ansiBuf));
    
    if (res != 0)
    {
        return res;
    }

    int converted = MultiByteToWideChar(CP_ACP, 0, ansiBuf, -1, name, namelen);
    
    if (converted == 0)
    {
        return SOCKET_ERROR;
    }

    return 0;
}