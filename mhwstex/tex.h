#pragma once

#include <cstdint>
#include <vector>
#include "gdeflate.h"

typedef uint16_t uint12_t;
typedef uint16_t uint4_t;

struct MipHeader {
    uint64_t offset;
    uint32_t pitch;
    uint32_t size;
};

struct TexHeader {
    char magic[4];
    uint32_t version;
    uint16_t width;
    uint16_t height;
    uint16_t depth;
    uint12_t texs; //12 bits
    uint4_t mips; //4 bits;
    uint32_t format;
    uint32_t tile;
    uint32_t option_flag;
    uint32_t tex_info;
    uint8_t swizzle_height_depth;
    uint8_t swizzle_width;
    uint16_t null;
    uint16_t _unk1;
    uint16_t _unk2;
    std::vector<std::vector<MipHeader>> mip_headers;

    void print();
};

struct Tex {
    TexHeader header;
    GDeflateData gdeflate_data;

    void parse(const char* file_name);
};
