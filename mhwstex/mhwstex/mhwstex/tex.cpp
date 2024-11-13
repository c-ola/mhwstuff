#include "tex.h"

#include <iostream>
#include <fstream>

void Tex::parse(const char* file_name) {
    std::ifstream file(file_name, std::ios::in | std::ios::binary);
    uint64_t index;

    file.read((char*)&this->header.magic, 4);
    file.read((char*)&this->header.version, 4);
    file.read((char*)&this->header.width, 2);
    file.read((char*)&this->header.height, 2);
    file.read((char*)&this->header.depth, 2);

    // Tex/Mip coubts
    uint16_t val;
    file.read((char*)&val, 2);

    // 01 10 => 0x1001
    // 0b0001_0000_0000_0001
    this->header.texs = (val >> 12);
    this->header.mips = (val & 0xfff);

    file.read((char*)&this->header.format, 4);
    file.read((char*)&this->header.tile, 4);
    file.read((char*)&this->header.option_flag, 4);
    file.read((char*)&this->header.tex_info, 4);
    file.read((char*)&this->header.swizzle_height_depth, 1);
    file.read((char*)&this->header.swizzle_width, 1);
    file.read((char*)&this->header.null, 2);
    file.read((char*)&this->header._unk1, 2);
    file.read((char*)&this->header._unk2, 2);

    for (int i = 0; i < this->header.texs; i++) {
        this->header.mip_headers.push_back(std::vector<MipHeader>(this->header.mips));
        for (int j = 0; j < this->header.mips; j++) {

            file.read((char*)&this->header.mip_headers[i][j], sizeof(MipHeader));
        }
    }
    this->header = this->header;

    GDeflateData gdef_header = {};
    gdef_header.sections = std::vector<GDefSection>(1);
    file.read((char*)&gdef_header.sections[0], sizeof(GDefSection));

    /*file.read((char*)&header.compressor_id, sizeof(uint8_t));
    file.read((char*)&header.magic, sizeof(uint8_t));
    file.read((char*)&header.num_tiles, sizeof(uint16_t));*/

    /*uint32_t val;
    file.read((char*)&val, sizeof(uint32_t));
    // 01 48 03 00 => 0x00034801
    // 0b0000_0000_0000_0011_0100_1000_0000_0001
    header.tile_size = val & 0b11;
    header.last_tile_size = (val >> 2) & 0b111111111111111111;
    header.reserved = val >> 20;
    this->header = header;*/
    file.seekg(gdef_header.sections[0].offset, std::ios_base::seekdir::_S_cur);
    gdef_header.data = (uint8_t*)malloc(gdef_header.sections[0].compressed_size);
    file.read((char*)&gdef_header.data, gdef_header.sections[0].compressed_size);
    this->gdeflate_data = gdef_header;
}

void TexHeader::print() {
    printf("Magic: %s, Ver: %u, Dims: {%hu, %hu, %hu}\n", this->magic, this->version, this->width, this->height, this->depth);
    printf("Texs: %u, Mips: %u\n", this->texs, this->mips);
    for (int i = 0; i < this->texs; i++) {
        printf("Tex %d\n", i);
        for (int j = 0; j < this->mips; j++) {
            MipHeader mh = this->mip_headers[i][j];
            printf("\tMip %d\n", j);
            printf("\t\tOffset: %ld, Pitch: %d, Size: %d\n", mh.offset, mh.pitch, mh.size);

        }
    }

}
