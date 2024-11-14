#include "tex.h"

#include <iostream>
#include <fstream>

TexHeader TexHeader::parse(std::ifstream &file){
    uint64_t index;
    TexHeader header = {};
    file.read((char*)&header.magic, 4);
    file.read((char*)&header.version, 4);
    file.read((char*)&header.width, 2);
    file.read((char*)&header.height, 2);
    file.read((char*)&header.depth, 2);

    // Tex/Mip coubts
    uint16_t val;
    file.read((char*)&val, 2);

    // 01 10 => 0x1001
    // 0b0001_0000_0000_0001
    header.mips = (val >> 12);
    header.texs = (val & 0xfff);

    file.read((char*)&header.format, 4);
    file.read((char*)&header.tile, 4);
    file.read((char*)&header.option_flag, 4);
    file.read((char*)&header.tex_info, 4);
    file.read((char*)&header.swizzle_height_depth, 1);
    file.read((char*)&header.swizzle_width, 1);
    file.read((char*)&header.null, 2);
    file.read((char*)&header._unk1, 2);
    file.read((char*)&header._unk2, 2);

    for (int i = 0; i < header.texs; i++) {
        header.mip_headers.push_back(std::vector<MipHeader>(header.mips));
        for (int j = 0; j < header.mips; j++) {

            file.read((char*)&header.mip_headers[i][j], sizeof(MipHeader));
        }
    }
    return header;
}

void TexHeader::write(std::ofstream &file){
    uint64_t index;
    file.write((char*)&this->magic, 4);
    file.write((char*)&this->version, 4);
    file.write((char*)&this->width, 2);
    file.write((char*)&this->height, 2);
    file.write((char*)&this->depth, 2);

    // Tex/Mip coubts
    uint16_t val = 0;
    // 01 10 => 0x1001
    // 0b0001_0000_0000_0001
    val += this->mips << 12;
    val += this->texs;
    file.write((char*)&val, 2);

    file.write((char*)&this->format, 4);
    file.write((char*)&this->tile, 4);
    file.write((char*)&this->option_flag, 4);
    file.write((char*)&this->tex_info, 4);
    file.write((char*)&this->swizzle_height_depth, 1);
    file.write((char*)&this->swizzle_width, 1);
    file.write((char*)&this->null, 2);
    file.write((char*)&this->_unk1, 2);
    file.write((char*)&this->_unk2, 2);

    for (int i = 0; i < this->texs; i++) {
        for (int j = 0; j < this->mips; j++) {
            file.write((char*)&this->mip_headers[i][j], sizeof(MipHeader));
        }
    }
}


void TexHeader::print() {
    printf("Magic: %s, Ver: %u, Dims: {%hu, %hu, %hu}\n", magic, this->version, this->width, this->height, this->depth);
    printf("Texs: %u, Mips: %u\n", texs, this->mips);
    for (int i = 0; i < texs; i++) {
        printf("Tex %d\n", i);
        for (int j = 0; j < mips; j++) {
            MipHeader mh = mip_headers[i][j];
            printf("\tMip %d\n", j);
            printf("\t\tOffset: %ld, Pitch: %d, Size: %d\n", mh.offset, mh.pitch, mh.size);

        }
    }

}
