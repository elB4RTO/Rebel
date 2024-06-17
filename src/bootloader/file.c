#include "file.h"
#include "lib.h"
#include "debug.h"

// the base in-memory address of the disk, see 'loader.asm'
#define DISK_BASE 0x1000000

// the base in-memory address of the disk partition where the kernel is stored
#define PARTITION_BASE 0x1007E00

// offset of the LBA value
#define LBA_OFFSET 0x1BE

// directory entry flags
#define ENTRY_EMPTY 0x00
#define ENTRY_DELETED 0xE5

// directory entry attributes
#define ATTR_LONG_FILENAME 0x0F

// cluster attributes
#define BAD_CLUSTER 0xFFF7
#define END_OF_CLUSTER_CHAIN 0xFFF8

// the cluster index starts from 2 and cannot be greater than 65535
#define VALIDATE_CLUSTER_INDEX(IDX) ASSERT((cluster_index >= 2 && cluster_index <= 65535), "Invalid cluster index")

/// returns the in-memory address of the BPB
static struct BPB* get_bpb(void)
{
    /*const uint32_t lba = *(uint32_t*)((uint64_t)DISK_BASE + LBA_OFFSET + 8);

    return (struct BPB*)((uint64_t)DISK_BASE + lba * 512);*/
    return (struct BPB*)PARTITION_BASE;
}

/// returns the in-memory address of the FAT table
static uint16_t* get_fat_table(const struct BPB *const bpb)
{
    const uint32_t fat_offset = (uint32_t)bpb->reserved_sector_count * bpb->bytes_per_sector;

    return (uint16_t*)((uint8_t*)bpb + fat_offset);
}

/// returns the value stored in the FAT table for the given `cluster_index`
static uint16_t get_cluster_value(const struct BPB *const bpb, const uint32_t cluster_index)
{
    VALIDATE_CLUSTER_INDEX(cluster_index)

    const uint16_t *const fat_table = get_fat_table(bpb);

    return fat_table[cluster_index];
}

/// returns the size of a cluster in the filesystem
static uint32_t get_cluster_size(const struct BPB *const bpb)
{
    return (uint32_t)bpb->bytes_per_sector * bpb->sectors_per_cluster;
}

/// returns the offset in the data section for the give `cluster_index`
static uint32_t get_data_offset(const struct BPB *const bpb, const uint32_t cluster_index)
{
    VALIDATE_CLUSTER_INDEX(cluster_index)

    const uint32_t cluster_size = get_cluster_size(bpb);
    const uint32_t cluster_offset = (cluster_index - 2) * cluster_size;
    const uint32_t reserved_size = (uint32_t)bpb->reserved_sector_count * bpb->bytes_per_sector;
    const uint32_t fat_size = (uint32_t)bpb->fat_count * bpb->sectors_per_fat * bpb->bytes_per_sector;
    const uint32_t root_dir_size = (uint32_t)bpb->root_entry_count * sizeof(struct DirEntry);
    const uint32_t data_offset = reserved_size + fat_size + root_dir_size;

    return data_offset + cluster_offset;
}

/// returns the in-memory address of the root directory
static struct DirEntry* get_root_directory(const struct BPB *const bpb)
{
    const uint32_t fat_start_sector = bpb->reserved_sector_count;
    const uint32_t fat_sectors = (uint32_t)bpb->fat_count * bpb->sectors_per_fat;
    const uint32_t root_dir_start_sector = fat_start_sector + fat_sectors;
    const uint32_t root_dir_offset = root_dir_start_sector * bpb->bytes_per_sector;

    return (struct DirEntry*)((uint8_t*)bpb + root_dir_offset);
}

/// checks whether the given `entry` matches the given `name` and `ext`
static bool is_file_name_equal(const struct DirEntry *const entry, const char* name, const char* ext)
{
    return memcmp(entry->name, name, 8) == 0 && memcmp(entry->ext, ext, 3) == 0;
}

/// splits the given `path` and stores the filename and the extension in `name` and `ext respectively
static bool split_path(const char* path, char* name, char* ext)
{
    int i;

    for (i = 0; i < 8 && path[i] != '.' && path[i] != '\0'; ++i)
    {
        // TODO: add support for sub-folders
        if (path[i] == '/')
        {
            return false;
        }

        name[i] = path[i];
    }

    if (path[i] == '.')
    {
        ++i;

        for (int j = 0; j < 3 && path[i] != '\0'; ++i, ++j)
        {
            if (path[i] == '/')
            {
                return false;
            }

            ext[j] = path[i];
        }
    }

    // TODO: add support for long file names
    if (path[i] != '\0')
    {
        return false;
    }

    return true;
}

/// returns the entry for the given file in the filesystem, if it exists
static struct DirEntry* search_file(const char* path)
{
    char name[8] = {"        "};
    char ext[3] =  {"   "};

    if (split_path(path, name, ext))
    {
        const struct BPB *const bpb = get_bpb();
        struct DirEntry* dir_entry = get_root_directory(bpb);

        const uint32_t max_i = bpb->root_entry_count;

        for (uint32_t i = 0; i < max_i; ++i)
        {
            struct DirEntry* entry = &dir_entry[i];
            if (entry->name[0] == ENTRY_EMPTY || entry->name[0] == ENTRY_DELETED)
            {
                continue;
            }
            else if (entry->attributes == ATTR_LONG_FILENAME) // currently no support for the long-file-name feature
            {
                continue;
            }
            else if (is_file_name_equal(entry, name, ext))
            {
                return entry;
            }
        }
    }

    return 0; // file not found
}

/// copies the data into `buffer`, starting from the given `cluster_index`, for a size defined by `size`
/// returns the size of data copied
static uint32_t read_raw_data(uint32_t cluster_index, uint8_t* buffer, const uint32_t size)
{
    VALIDATE_CLUSTER_INDEX(cluster_index)

    const struct BPB *const bpb = get_bpb();
    const uint32_t cluster_size = get_cluster_size(bpb);

    uint8_t* data = (uint8_t*)((uint64_t)bpb + get_data_offset(bpb, cluster_index));
    uint32_t read_size = 0;

    while (read_size < size)
    {
        const uint32_t cluster_value = get_cluster_value(bpb, cluster_index);

        if (cluster_value == BAD_CLUSTER)
        {
            ASSERT(0, "Bad cluster");
        }
        else if (cluster_value >= END_OF_CLUSTER_CHAIN)
        {
            const uint32_t left_to_read = size - read_size;
            memmove(buffer, data, left_to_read);
            read_size += left_to_read;
            break;
        }

        memmove(buffer, data, cluster_size);

        data += cluster_size;
        buffer += cluster_size;
        read_size += cluster_size;
        ++cluster_index;
    }

    return read_size;
}

bool load_file(char* path, const uint64_t addr)
{
    const struct DirEntry *const entry = search_file(path);

    if (entry)
    {
        const uint32_t file_size = entry->file_size;
        const uint32_t cluster_index = entry->cluster_index;

        if (read_raw_data(cluster_index, (uint8_t*)addr, file_size) == file_size)
        {
            return true;
        }
    }

    return false;
}

void init_fs(void)
{
    uint8_t* p = (uint8_t*)get_bpb();

    if (p[0x1FE] != 0x55 || p[0x1FF] != 0xAA)
    {
        ASSERT(0, "Invalid filesystem signature");
    }
}

