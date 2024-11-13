#include "tex.h"

#include <iostream>
#include <fstream>

void Tex::parse(const char* file_name) {
	std::ifstream file(file_name, std::ios::in | std::ios::binary);
	uint64_t index;

	TexHeader tex_header = {};
	file.read((char*)&tex_header.magic, 4);
	file.read((char*)&tex_header.version, 4);
	file.read((char*)&tex_header.width, 2);
	file.read((char*)&tex_header.height, 2);
	file.read((char*)&tex_header.depth, 2);

	// Tex/Mip coubts
	uint16_t val;
	file.read((char*)&val, 2);

	// 01 10 => 0x1001
	// 0b0001_0000_0000_0001
	tex_header.texs = (val >> 12);
	tex_header.mips = (val & 0xfff);

	file.read((char*)&tex_header.format, 4);
	file.read((char*)&tex_header.tile, 4);
	file.read((char*)&tex_header.option_flag, 4);
	file.read((char*)&tex_header.tex_info, 4);
	file.read((char*)&tex_header.swizzle_height_depth, 1);
	file.read((char*)&tex_header.swizzle_width, 1);
	file.read((char*)&tex_header.null, 2);
	file.read((char*)&tex_header._unk1, 2);
	file.read((char*)&tex_header._unk2, 2);

	for (int i = 0; i < tex_header.texs; i++) {
		tex_header.mip_headers.push_back(std::vector<MipHeader>(tex_header.mips));
		for (int j = 0; j < tex_header.mips; j++) {

			file.read((char*)&tex_header.mip_headers[i][j], sizeof(MipHeader));
		}
	}

	this->header = tex_header;
	this->gdeflate_data.parse_header(file);
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