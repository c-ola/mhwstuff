/*
 * SPDX-FileCopyrightText: Copyright (c) 2020, 2021, 2022 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-FileCopyrightText: Copyright (c) Microsoft Corportaion. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
#pragma once
#include <cstdint>
#include <vector>

#include <fstream>

#include <stdint.h>

#include <algorithm>
#include <atomic>
#include <chrono>
#include <cstring>
#include <limits>

#include <assert.h>
#include <stdint.h>

#include <string>



namespace GDeflate
{
    static constexpr uint8_t kGDeflateId = 4;

    static const size_t kDefaultTileSize = 64 * 1024;
    // See README.MD in libdeflate_1_8 for details on Compression Levels
    static const uint32_t MinimumCompressionLevel = 1;
    static const uint32_t MaximumCompressionLevel = 12;

    enum Flags
    {
        COMPRESS_SINGLE_THREAD = 0x200, /*!< Force compression using a single thread. */
    };

    size_t CompressBound(size_t size);

    bool Compress(
        uint8_t* output,
        size_t* outputSize,
        const uint8_t* in,
        size_t inSize,
        uint32_t level,
        uint32_t flags);

    bool Decompress(uint8_t* output, size_t outputSize, const uint8_t* in, size_t inSize, uint32_t numWorkers);

    template<int N, typename T>
    static inline T _align(T a)
    {
        return (a + T(N) - 1) & ~(T(N) - 1);
    }

    template<typename T>
    static inline T _divRoundup(T a, T b)
    {
        return (a + b - 1) / b;
    }

    template<typename T>
    static inline uint32_t _lzCount(T a)
    {
        uint32_t n = 0;

        while (0 == (a & 1) && n < sizeof(T) * 8)
        {
            a >>= 1;
            ++n;
        }

        return n;
    }

    template<typename T>
    static inline T GetBits(uint32_t*& in, uint32_t& offset, uint32_t numBitsToRead)
    {
        constexpr uint32_t kBitsPerBucket = sizeof(*in) * 8;

        T bits = 0;
        uint32_t numBitsConsumed = 0;
        while (numBitsConsumed < numBitsToRead)
        {
            const uint32_t numBits =
                std::min(numBitsToRead - numBitsConsumed, kBitsPerBucket - (offset % kBitsPerBucket));

            const T mask = std::numeric_limits<T>().max() >> (sizeof(T) * 8 - numBits);

            bits |= (T(*in >> (offset % kBitsPerBucket)) & mask) << numBitsConsumed;

            offset += numBits;
            numBitsConsumed += numBits;

            if (0 == offset % kBitsPerBucket)
                in++;
        }

        return bits;
    }

#pragma pack(push, 1)
    struct TileStream
    {
        static constexpr uint32_t kMaxTiles = (1 << 16) - 1;

        uint8_t id;
        uint8_t magic;

        uint16_t numTiles;

        uint32_t tileSizeIdx : 2; // this must be set to 1
        uint32_t lastTileSize : 18;
        uint32_t reserved1 : 12;

        TileStream(size_t uncompressedSize)
        {
            memset(this, 0, sizeof(*this));
            tileSizeIdx = 1;
            SetCodecId(kGDeflateId);
            SetUncompressedSize(uncompressedSize);
        }

        bool IsValid() const
        {
            return id == (magic ^ 0xff);
        }

        size_t GetUncompressedSize() const
        {
            return numTiles * kDefaultTileSize - (lastTileSize == 0 ? 0 : kDefaultTileSize - lastTileSize);
        }

    private:
        void SetCodecId(uint8_t inId)
        {
            id = inId;
            magic = inId ^ 0xff;
        }

        void SetUncompressedSize(size_t size)
        {
            numTiles = static_cast<uint16_t>(size / kDefaultTileSize);
            lastTileSize = static_cast<uint32_t>(size - numTiles * kDefaultTileSize);

            numTiles += lastTileSize != 0 ? 1 : 0;
        }
    };

#pragma pack(pop)

    static_assert(sizeof(TileStream) == 8, "Tile stream header size overrun!");

} // namespace GDeflate

struct GDefSection {
    uint32_t compressed_size;
    uint32_t offset;
};

struct GDeflateHeader {
    uint8_t compressor_id;
    uint8_t magic;
    uint16_t num_tiles;

    // 32 bits in order
    uint8_t tile_size; // 2 bits
    uint32_t last_tile_size; // 18 bits
    uint16_t reserved; // 12 bits
};

struct GDeflateData {
    std::vector<GDefSection> sections;
    uint8_t* data;

    void parse_header(std::ifstream &file);
};

