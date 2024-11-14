//#include <dstorage.h>
//#include <wrl.h>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <string>
#include <vector>
#include <stdexcept>
//#include <codecvt>
//#include <locale>
//#include <winrt/base.h>

//using Microsoft::WRL::ComPtr;
//using winrt::check_hresult;
//using winrt::com_ptr;

#include "tex.h"
#include "gdeflate.h"

#include "libpng16/png.h"
#include <filesystem>


bool writePNG(const char* filename, uint8_t* buffer, int width, int height);

void decompress_texture(const char* file_name) {
    std::ifstream file(file_name, std::ios::in | std::ios::binary);
    TexHeader th = TexHeader::parse(file);
    th.print();
    //texture.gdeflate_data.header.print();
    
    auto sections = std::vector<GDefSection>(th.mips*th.texs);
    GDefSection section = {};
    for (int i = 0; i < th.mips; i++) {
        file.read((char*)&section, sizeof(GDefSection));
        printf("GDeflate: %d, %d\n", section.compressed_size, section.offset);
        sections[i] = section;
    }
    for (int i = 0; i < 1; i++) {
        auto section = sections[0];
        auto mip = th.mip_headers[0][i];
        size_t out_size = mip.size;
        size_t in_size = section.compressed_size;
        printf("GDeflate: %d, %d\n", section.compressed_size, section.offset);

        printf("in_size: %ld, out_size: %ld\n", in_size, out_size);

        uint8_t* in_buf = (uint8_t*)malloc(in_size);
        size_t base = mip.offset + section.offset + th.texs * th.mips * 8;
        printf("%lx\n", base);
        file.seekg(base);
        file.read((char*)in_buf, section.compressed_size);

        uint8_t* out_buf = (uint8_t*)malloc(out_size + 16);
        if(!GDeflate::Decompress(out_buf, out_size, in_buf, in_size, 1)) {
            printf("Failed\n");
        }else {
            printf("Decompressed!\n");
            std::string full_path = std::string(file_name);
            auto pos = full_path.find("natives");
            auto path = "./outputs/" + full_path.substr(pos, full_path.length());
            path = path.substr(0, path.find_last_of('/'));
            printf("%s\n", path.c_str());
            std::filesystem::create_directories(path);
            auto name = full_path.substr(full_path.find_last_of("/"), full_path.length());
            std::string out_name = path + name;

            printf("%s\n", out_name.c_str());

            std::ofstream out_file(out_name, std::ios::out | std::ios::binary);
            th.write(out_file);
            out_buf[out_size] = 0xa;
            out_file.write((char*)out_buf, out_size);
            out_file.close();
            printf("Wrote to file %s\n", out_name.c_str());
            //writePNG("image.png", out_buf, th.width, th.height);
        }

        free(out_buf);
        free(in_buf);
    }

    file.close();
}

bool writePNG(const char* filename, uint8_t* buffer, int width, int height) {
    FILE *fp = fopen(filename, "wb");
    if (!fp) {
        std::cerr << "Could not open file for writing: " << filename << std::endl;
        return false;
    }

    // Initialize the write structure
    png_structp png = png_create_write_struct(PNG_LIBPNG_VER_STRING, nullptr, nullptr, nullptr);
    if (!png) {
        std::cerr << "Could not allocate write struct" << std::endl;
        fclose(fp);
        return false;
    }

    // Initialize the info structure
    png_infop info = png_create_info_struct(png);
    if (!info) {
        std::cerr << "Could not allocate info struct" << std::endl;
        png_destroy_write_struct(&png, nullptr);
        fclose(fp);
        return false;
    }

    // Error handling
    if (setjmp(png_jmpbuf(png))) {
        std::cerr << "Error during png creation" << std::endl;
        png_destroy_write_struct(&png, &info);
        fclose(fp);
        return false;
    }

    png_init_io(png, fp);

    // Write header
    png_set_IHDR(
            png,
            info,
            width,
            height,
            8, // Bit depth
            PNG_COLOR_TYPE_GRAY, // Color type: grayscale
            PNG_INTERLACE_NONE,
            PNG_COMPRESSION_TYPE_DEFAULT,
            PNG_FILTER_TYPE_DEFAULT
            );
    png_write_info(png, info);

    // Write image data
    png_bytep row_pointers[height];
    for (int y = 0; y < height; ++y) {
        row_pointers[y] = buffer + y * width;
    }

    png_write_image(png, row_pointers);

    // End write
    png_write_end(png, nullptr);

    // Cleanup
    png_destroy_write_struct(&png, &info);
    fclose(fp);

    return true;
}

int main(int argc, char* argv[]) {

    if (argc < 2) {
        printf("provide a file\n");
        //return 0;
    }
    const char* file = argv[1];

    decompress_texture(file);

    return 0;
}
