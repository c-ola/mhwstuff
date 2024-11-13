//#include <dstorage.h>
//#include <wrl.h>
#include <iostream>
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


bool writePNG(const char* filename, uint8_t* buffer, int width, int height);

void decompress_texture(const char* file_name) {
    Tex texture = {};
    texture.parse(file_name);
    texture.header.print();
    //texture.gdeflate_data.header.print();

    size_t out_size = texture.header.mip_headers[0][0].size;
    uint8_t* output = (uint8_t*)malloc(texture.header.mip_headers[0][0].size);
    size_t in_size = texture.gdeflate_data.sections[0].compressed_size;
    GDeflate::Decompress(output, out_size, texture.gdeflate_data.data, in_size, 1);
    writePNG("image.png", output, texture.header.width, texture.header.height);
    free(output);

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
