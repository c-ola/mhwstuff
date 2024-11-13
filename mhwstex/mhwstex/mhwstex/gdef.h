#pragma once
#include <cstdint>
#include <vector>

#include <fstream>

struct GDefSection {
    uint32_t compressed_size;
    uint32_t offset;
};

struct GDeflateHeader {
    std::vector<GDefSection> sections;
    uint8_t compressor_id;
    uint8_t magic;
    uint16_t num_tiles;

    // 32 bits in order
    uint8_t tile_size; // 2 bits
    uint32_t last_tile_size; // 18 bits
    uint16_t reserved; // 12 bits
    void print();
};

struct GDeflateData {
    GDeflateHeader header;
    std::vector<uint8_t> data;

    void parse_header(std::ifstream &file);
};