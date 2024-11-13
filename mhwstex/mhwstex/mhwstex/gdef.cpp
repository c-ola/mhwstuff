#include "gdef.h"

void GDeflateData::parse_header(std::ifstream &file){
    GDeflateHeader header = {};
    header.sections = std::vector<GDefSection>(1);
    file.read((char*)&header.sections[0], sizeof(GDefSection));

    file.read((char*)&header.compressor_id, sizeof(uint8_t));
    file.read((char*)&header.magic, sizeof(uint8_t));
    file.read((char*)&header.num_tiles, sizeof(uint16_t));

    uint32_t val;
    file.read((char*)&val, sizeof(uint32_t));
    // 01 48 03 00 => 0x00034801
    // 0b0000_0000_0000_0011_0100_1000_0000_0001
    header.tile_size = val & 0b11;
    header.last_tile_size = (val >> 2) & 0b111111111111111111;
    header.reserved = val >> 20;
    this->header = header;
}

void GDeflateHeader::print(){
    for (int i = 0; i < sections.size(); i++){
        printf("Section %d: %d, %d\n", i, sections[i].compressed_size, sections[i].offset);
    }
    printf("Compr ID: %x, Magic: %x, Num Tiles: %d\n", compressor_id, magic, num_tiles);
    printf("Tile Size: %d, Last Tile Size: %d, Reserved: %d\n", tile_size, last_tile_size, reserved);
}