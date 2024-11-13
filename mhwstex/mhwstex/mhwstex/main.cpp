//#include <dstorage.h>
//#include <wrl.h>
#include <iostream>
#include <vector>
#include <stdexcept>
#include <string>
//#include <codecvt>
//#include <locale>
//#include <winrt/base.h>

//using Microsoft::WRL::ComPtr;
//using winrt::check_hresult;
//using winrt::com_ptr;

#include <windows.h>
#include <d3d12.h>

#include "tex.h"

void create_gdeflate_file(const char* file_name) {
    Tex texture;
    texture.parse(file_name);
    texture.header.print();
    texture.gdeflate_data.header.print();
}

void decompress_file(const wchar_t* file_name) {
}

int main(int argc, char* argv[]) {
       
    if (argc < 2) {
        printf("provide a file\n");
        //return 0;
    }
    const char* file = argv[1];
    file = "C:\\Users\\nikol\\source\\repos\\mhwstex\\mhwstex\\wal_00_iml3.tex.240701001";

    create_gdeflate_file(file);

    const wchar_t* originalFilename = (wchar_t*)argv[1];
    std::wstring gdeflateFilename = std::wstring(originalFilename) + L".gdeflate";

    uint32_t chunkSizeMiB = 16;
    uint32_t chunkSizeBytes = chunkSizeMiB * 1024 * 1024;
    constexpr uint32_t MAX_STAGING_BUFFER_SIZE = 1024;

    decompress_file(gdeflateFilename.c_str());

    return 0;
}