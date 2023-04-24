/*
 * libfip.dll
 *
 * DO NOT SUBMIT GENERATED DLLS FOR INCLUSION INTO WINE!
 */

#include <stdint.h>
#include <stdarg.h>

#include "windef.h"
#include "winbase.h"
#include "winternl.h"
#include "wine/debug.h"

#include "directoutput.h"

WINE_DEFAULT_DEBUG_CHANNEL(libfip);

typedef void (WINAPI *WinApi_DirectOutput_EnumerateCallback)(void* hDevice, void* pCtxt);
typedef void (WINAPI *WinApi_DirectOutput_DeviceChange)(void* hDevice, bool bAdded, void* pCtxt);
typedef void (WINAPI *WinApi_DirectOutput_PageChange)(void* hDevice, DWORD dwPage, bool bSetActive, void* pCtxt);
typedef void (WINAPI *WinApi_DirectOutput_SoftButtonChange)(void* hDevice, DWORD dwButtons, void* pCtxt);

struct CallbackData { void* pfnCb; void* pCtxt; };

void Proxy_DirectOutput_EnumerateCallback(void* hDevice, void* pCtxt) {
    struct CallbackData* cb = (struct CallbackData*)pCtxt;
	return (*((WinApi_DirectOutput_EnumerateCallback)cb->pfnCb))(
        hDevice,
        cb->pCtxt
    );
}
void Proxy_DirectOutput_DeviceChange(void* hDevice, bool bAdded, void* pCtxt) {
    struct CallbackData* cb = (struct CallbackData*)pCtxt;
	return (*((WinApi_DirectOutput_DeviceChange)cb->pfnCb))(
        hDevice, bAdded,
        cb->pCtxt
    );
}
void Proxy_DirectOutput_PageChange(void* hDevice, DWORD dwPage, bool bSetActive, void* pCtxt) {
    struct CallbackData* cb = (struct CallbackData*)pCtxt;
	return (*((WinApi_DirectOutput_PageChange)cb->pfnCb))(
        hDevice, dwPage, bSetActive,
        cb->pCtxt
    );
}
void Proxy_DirectOutput_SoftButtonChange(void* hDevice, DWORD dwButtons, void* pCtxt) {
    struct CallbackData* cb = (struct CallbackData*)pCtxt;
	return (*((WinApi_DirectOutput_SoftButtonChange)cb->pfnCb))(
        hDevice, dwButtons,
        cb->pCtxt
    );
}

HRESULT WINAPI ProxyDirectOutput_Initialize(LPCWSTR wszPluginName) {
	return DirectOutput_Initialize(wszPluginName);
}
HRESULT WINAPI ProxyDirectOutput_Deinitialize() {
	return DirectOutput_Deinitialize();
}
HRESULT WINAPI ProxyDirectOutput_RegisterDeviceCallback(void* pfnCb, void* pCtxt) {
    struct CallbackData cb = {pfnCb, pCtxt};
    return DirectOutput_RegisterDeviceCallback(Proxy_DirectOutput_DeviceChange, &cb);
}
HRESULT WINAPI ProxyDirectOutput_Enumerate(void* pfnCb, void* pCtxt) {
    struct CallbackData cb = {pfnCb, pCtxt};
    return DirectOutput_Enumerate(Proxy_DirectOutput_EnumerateCallback, &cb);
}
HRESULT WINAPI ProxyDirectOutput_RegisterPageCallback(void* hDevice, void* pfnCb, void* pCtxt) {
    struct CallbackData cb = {pfnCb, pCtxt};
    return DirectOutput_RegisterPageCallback(hDevice, Proxy_DirectOutput_PageChange, &cb);
}
HRESULT WINAPI ProxyDirectOutput_RegisterSoftButtonCallback(void* hDevice, void* pfnCb, void* pCtxt) {
    struct CallbackData cb = {pfnCb, pCtxt};
    return DirectOutput_RegisterSoftButtonCallback(hDevice, Proxy_DirectOutput_SoftButtonChange, &cb);
}
HRESULT WINAPI ProxyDirectOutput_GetDeviceType(void* hDevice, void* pGuid) {
    return DirectOutput_GetDeviceType(hDevice, pGuid);
}
HRESULT WINAPI ProxyDirectOutput_GetDeviceInstance(void* hDevice, void* pGuid) {
    return DirectOutput_GetDeviceInstance(hDevice, pGuid);
}
HRESULT WINAPI ProxyDirectOutput_SetProfile(void* hDevice, DWORD cchProfile, LPCWSTR wszProfile) {
    return DirectOutput_SetProfile(hDevice, cchProfile, wszProfile);
}
HRESULT WINAPI ProxyDirectOutput_AddPage(void* hDevice, DWORD dwPage, LPCWSTR wszDebugName, DWORD dwFlags) {
    return DirectOutput_AddPage(hDevice, dwPage, wszDebugName, dwFlags);
}
HRESULT WINAPI ProxyDirectOutput_RemovePage(void* hDevice, DWORD dwPage) {
    return DirectOutput_RemovePage(hDevice, dwPage);
}
HRESULT WINAPI ProxyDirectOutput_SetLed(void* hDevice, DWORD dwPage, DWORD dwIndex, DWORD dwValue) {
    return DirectOutput_SetLed(hDevice, dwPage, dwIndex, dwValue);
}
HRESULT WINAPI ProxyDirectOutput_SetString(void* hDevice, DWORD dwPage, DWORD dwIndex, DWORD cchValue, LPCWSTR wszValue) {
    return DirectOutput_SetString(hDevice, dwPage, dwIndex, cchValue, wszValue);
}
HRESULT WINAPI ProxyDirectOutput_SetImage(void* hDevice, DWORD dwPage, DWORD dwIndex, DWORD cbValue, const void* pvValue) {
    return DirectOutput_SetImage(hDevice, dwPage, dwIndex, cbValue, pvValue);
}
HRESULT WINAPI ProxyDirectOutput_SetImageFromFile(void* hDevice, DWORD dwPage, DWORD dwIndex, DWORD cchFilename, LPCWSTR wszFilename) {
    return DirectOutput_SetImageFromFile(hDevice, dwPage, dwIndex, cchFilename, wszFilename);
}
HRESULT WINAPI ProxyDirectOutput_StartServer(void* hDevice, DWORD cchFilename, LPCWSTR wszFilename, void* pdwServerId, void* psStatus) {
    return DirectOutput_StartServer(hDevice, cchFilename, wszFilename, pdwServerId, psStatus);
}
HRESULT WINAPI ProxyDirectOutput_CloseServer(void* hDevice, DWORD dwServerId, void* psStatus) {
    return DirectOutput_CloseServer(hDevice, dwServerId, psStatus);
}
HRESULT WINAPI ProxyDirectOutput_SendServerMsg(void* hDevice, DWORD dwServerId, DWORD dwRequest, DWORD dwPage, DWORD cbIn, const void* pvIn, DWORD cbOut, void* pvOut, void* psStatus) {
    return DirectOutput_SendServerMsg(hDevice, dwServerId, dwRequest, dwPage, cbIn, pvIn, cbOut, pvOut, psStatus);
}
HRESULT WINAPI ProxyDirectOutput_SendServerFile(void* hDevice, DWORD dwServerId, DWORD dwRequest, DWORD dwPage, DWORD cbInHdr, const void* pvInHdr, DWORD cchFile, LPCWSTR wszFile, DWORD cbOut, void* pvOut, void* psStatus) {
    return DirectOutput_SendServerFile(hDevice, dwServerId, dwRequest, dwPage, cbInHdr, pvInHdr, cchFile, wszFile, cbOut, pvOut, psStatus);
}
HRESULT WINAPI ProxyDirectOutput_SaveFile(void* hDevice, DWORD dwPage, DWORD dwFile, DWORD cchFilename, LPCWSTR wszFilename, void* psStatus) {
    return DirectOutput_SaveFile(hDevice, dwPage, dwFile, cchFilename, wszFilename, psStatus);
}
HRESULT WINAPI ProxyDirectOutput_DisplayFile(void* hDevice, DWORD dwPage, DWORD dwIndex, DWORD dwFile, void* psStatus) {
    return DirectOutput_DisplayFile(hDevice, dwPage, dwIndex, dwFile, psStatus);
}
HRESULT WINAPI ProxyDirectOutput_DeleteFile(void* hDevice, DWORD dwPage, DWORD dwFile, void* psStatus) {
    return DirectOutput_DeleteFile(hDevice, dwPage, dwFile, psStatus);
}
HRESULT WINAPI ProxyDirectOutput_GetSerialNumber(void* hDevice, LPWSTR pszSerialNumber, DWORD dwSize) {
    return DirectOutput_GetSerialNumber(hDevice, pszSerialNumber, dwSize);
}
