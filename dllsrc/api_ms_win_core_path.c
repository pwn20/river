#include <windows.h>
#include <shlwapi.h>
#include <strsafe.h>
#include <wchar.h>

BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved)
{
    (void)hinstDLL;
    (void)lpvReserved;

    switch (fdwReason)
    {
        case DLL_PROCESS_ATTACH:
        {
            DisableThreadLibraryCalls(hinstDLL);
            break;
        }
        case DLL_THREAD_ATTACH:
        case DLL_THREAD_DETACH:
        case DLL_PROCESS_DETACH:
        {
            break;
        }
    }
    return TRUE;
}

__declspec(dllexport) HRESULT WINAPI PathCchAddBackslash(PWSTR pszPath, size_t cchPath)
{
    if (!pszPath || cchPath == 0)
    {
        return E_INVALIDARG;
    }
    
    size_t len = wcslen(pszPath);
    if (len == 0 || pszPath[len - 1] == L'\\')
    {
        return S_FALSE; 
    }
    
    if (len + 1 >= cchPath)
    {
        return HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER);
    }
    
    PathAddBackslashW(pszPath);
    return S_OK;
}

__declspec(dllexport) HRESULT WINAPI PathCchAddBackslashEx(PWSTR pszPath, size_t cchPath, PWSTR *ppszEnd, size_t *pcchRemaining)
{
    HRESULT hr = PathCchAddBackslash(pszPath, cchPath);
    if (SUCCEEDED(hr))
    {
        if (ppszEnd)
        {
            *ppszEnd = pszPath + wcslen(pszPath);
        }
        if (pcchRemaining)
        {
            *pcchRemaining = cchPath - wcslen(pszPath);
        }
    }
    return hr;
}

__declspec(dllexport) HRESULT WINAPI PathCchAddExtension(PWSTR pszPath, size_t cchPath, PCWSTR pszExt)
{
    if (!pszPath || !pszExt || cchPath == 0)
    {
        return E_INVALIDARG;
    }
    
    if (wcslen(pszPath) + wcslen(pszExt) >= cchPath)
    {
        return HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER);
    }
    
    PathAddExtensionW(pszPath, pszExt);
    return S_OK;
}

__declspec(dllexport) HRESULT WINAPI PathCchAppend(PWSTR pszPath, size_t cchPath, PCWSTR pszMore)
{
    if (!pszPath || !pszMore || cchPath == 0)
    {
        return E_INVALIDARG;
    }
    
    WCHAR tempBuf[MAX_PATH] = {0};
    StringCchCopyW(tempBuf, MAX_PATH, pszPath);
    
    if (!PathAppendW(tempBuf, pszMore))
    {
        return E_FAIL;
    }
    
    if (wcslen(tempBuf) >= cchPath)
    {
        return HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER);
    }
    
    StringCchCopyW(pszPath, cchPath, tempBuf);
    return S_OK;
}

__declspec(dllexport) HRESULT WINAPI PathCchAppendEx(PWSTR pszPath, size_t cchPath, PCWSTR pszMore, DWORD dwFlags)
{
    (void)dwFlags;
    return PathCchAppend(pszPath, cchPath, pszMore);
}

__declspec(dllexport) HRESULT WINAPI PathCchCanonicalize(PWSTR pszPathOut, size_t cchPathOut, PCWSTR pszPathIn)
{
    if (!pszPathOut || !pszPathIn || cchPathOut == 0)
    {
        return E_INVALIDARG;
    }
    
    WCHAR tempBuf[MAX_PATH] = {0};
    if (!PathCanonicalizeW(tempBuf, pszPathIn))
    {
        return E_FAIL;
    }
    
    if (wcslen(tempBuf) >= cchPathOut)
    {
        return HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER);
    }
    
    StringCchCopyW(pszPathOut, cchPathOut, tempBuf);
    return S_OK;
}

__declspec(dllexport) HRESULT WINAPI PathCchCanonicalizeEx(PWSTR pszPathOut, size_t cchPathOut, PCWSTR pszPathIn, DWORD dwFlags)
{
    (void)dwFlags;
    return PathCchCanonicalize(pszPathOut, cchPathOut, pszPathIn);
}

__declspec(dllexport) HRESULT WINAPI PathCchCombine(PWSTR pszPathOut, size_t cchPathOut, PCWSTR pszPathIn, PCWSTR pszMore)
{
    if (!pszPathOut || cchPathOut == 0)
    {
        return E_INVALIDARG;
    }
    
    WCHAR tempBuf[MAX_PATH] = {0};
    if (!PathCombineW(tempBuf, pszPathIn, pszMore))
    {
        return E_FAIL;
    }
    
    if (wcslen(tempBuf) >= cchPathOut)
    {
        return HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER);
    }
    
    StringCchCopyW(pszPathOut, cchPathOut, tempBuf);
    return S_OK;
}

__declspec(dllexport) HRESULT WINAPI PathCchCombineEx(PWSTR pszPathOut, size_t cchPathOut, PCWSTR pszPathIn, PCWSTR pszMore, DWORD dwFlags)
{
    (void)dwFlags;
    return PathCchCombine(pszPathOut, cchPathOut, pszPathIn, pszMore);
}

__declspec(dllexport) PCWSTR WINAPI PathCchFindExtension(PCWSTR pszPath, size_t cchPath)
{
    (void)cchPath;
    if (!pszPath)
    {
        return NULL;
    }
    return PathFindExtensionW(pszPath);
}

__declspec(dllexport) BOOL WINAPI PathCchIsRoot(PCWSTR pszPath)
{
    if (!pszPath)
    {
        return FALSE;
    }
    return PathIsRootW(pszPath);
}

__declspec(dllexport) HRESULT WINAPI PathCchRemoveBackslash(PWSTR pszPath, size_t cchPath)
{
    (void)cchPath;
    if (!pszPath)
    {
        return E_INVALIDARG;
    }
    
    size_t len = wcslen(pszPath);
    if (len > 0 && pszPath[len - 1] == L'\\')
    {
        pszPath[len - 1] = L'\0';
        return S_OK;
    }
    return S_FALSE;
}

__declspec(dllexport) HRESULT WINAPI PathCchRemoveBackslashEx(PWSTR pszPath, size_t cchPath, PWSTR *ppszEnd, size_t *pcchRemaining)
{
    HRESULT hr = PathCchRemoveBackslash(pszPath, cchPath);
    if (ppszEnd)
    {
        *ppszEnd = pszPath + wcslen(pszPath);
    }
    if (pcchRemaining)
    {
        *pcchRemaining = cchPath - wcslen(pszPath);
    }
    return hr;
}

__declspec(dllexport) HRESULT WINAPI PathCchRemoveExtension(PWSTR pszPath, size_t cchPath)
{
    (void)cchPath;
    if (!pszPath)
    {
        return E_INVALIDARG;
    }
    PathRemoveExtensionW(pszPath);
    return S_OK;
}

__declspec(dllexport) HRESULT WINAPI PathCchRemoveFileSpec(PWSTR pszPath, size_t cchPath)
{
    (void)cchPath;
    if (!pszPath)
    {
        return E_INVALIDARG;
    }
    PathRemoveFileSpecW(pszPath);
    return S_OK;
}

__declspec(dllexport) HRESULT WINAPI PathCchRenameExtension(PWSTR pszPath, size_t cchPath, PCWSTR pszExt)
{
    if (!pszPath || !pszExt || cchPath == 0)
    {
        return E_INVALIDARG;
    }
    
    WCHAR tempBuf[MAX_PATH] = {0};
    StringCchCopyW(tempBuf, MAX_PATH, pszPath);
    PathRenameExtensionW(tempBuf, pszExt);
    
    if (wcslen(tempBuf) >= cchPath)
    {
        return HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER);
    }
    
    StringCchCopyW(pszPath, cchPath, tempBuf);
    return S_OK;
}

__declspec(dllexport) HRESULT WINAPI PathCchSkipRoot(PCWSTR pszPath, PCWSTR *ppszRootEnd)
{
    if (!pszPath || !ppszRootEnd)
    {
        return E_INVALIDARG;
    }
    PCWSTR pResult = (PCWSTR)PathSkipRootW(pszPath);
    if (pResult && *pResult != L'\0')
    {
        *ppszRootEnd = pResult;
        return S_OK;
    }
    return E_FAIL;
}

__declspec(dllexport) HRESULT WINAPI PathCchStripPrefix(PWSTR pszPath, size_t cchPath)
{
    if (!pszPath || cchPath == 0)
    {
        return E_INVALIDARG;
    }
    
    if (wcsncmp(pszPath, L"\\\\?\\", 4) == 0)
    {
        size_t len = wcslen(pszPath);
        wmemmove(pszPath, pszPath + 4, len - 4 + 1);
        return S_OK;
    }
    return S_FALSE; 
}

__declspec(dllexport) HRESULT WINAPI PathCchStripToRoot(PWSTR pszPath, size_t cchPath)
{
    (void)cchPath;
    if (!pszPath)
    {
        return E_INVALIDARG;
    }
    PathStripToRootW(pszPath);
    return S_OK;
}

__declspec(dllexport) BOOL WINAPI PathIsUNCEx(PCWSTR pszPath, PCWSTR *ppszServer)
{
    if (!pszPath)
    {
        return FALSE;
    }
    if (ppszServer)
    {
        *ppszServer = NULL;
    }
    return PathIsUNCW(pszPath);
}

__declspec(dllexport) HRESULT WINAPI PathAllocCanonicalize(PCWSTR pszPathIn, DWORD dwFlags, PWSTR *ppszPathOut)
{
    (void)dwFlags;
    if (!pszPathIn || !ppszPathOut)
    {
        return E_INVALIDARG;
    }
    
    PWSTR buf = (PWSTR)LocalAlloc(LPTR, MAX_PATH * sizeof(WCHAR));
    if (!buf)
    {
        return E_OUTOFMEMORY;
    }
    
    if (!PathCanonicalizeW(buf, pszPathIn))
    {
        LocalFree(buf);
        return E_FAIL;
    }
    
    *ppszPathOut = buf;
    return S_OK;
}

__declspec(dllexport) HRESULT WINAPI PathAllocCombine(PCWSTR pszPathIn, PCWSTR pszMore, DWORD dwFlags, PWSTR *ppszPathOut)
{
    (void)dwFlags;
    if (!pszPathIn || !ppszPathOut)
    {
        return E_INVALIDARG;
    }
    
    PWSTR buf = (PWSTR)LocalAlloc(LPTR, MAX_PATH * sizeof(WCHAR));
    if (!buf)
    {
        return E_OUTOFMEMORY;
    }
    
    if (!PathCombineW(buf, pszPathIn, pszMore))
    {
        LocalFree(buf);
        return E_FAIL;
    }
    
    *ppszPathOut = buf;
    return S_OK;
}