# Flan's PSX Texture Collection file specification
[Back to main page.](../README.md)

## Texture Collection file (.txc)
|Type|Description|
|----|-----------|
|u32|Number of texture cells in this file.|
|TextureCellDesc[]| Array of texture cell descriptions.|
|u16|Number of 16 color palettes in this file.
|u16[]|Raw 16-bit (R5G5B5A1) palette color values.|
|u8[]|Raw texture indices.

## TextureCellDesc
|Type|Description|
|----|-----------|
|u32| Offset to texture data in binary section.|
|u16| Index into palette array.|
|u8| Texture width in pixels.|
|u8| Texture height in pixels.|




## Texture Collection file (.txc)
The file header is as follows
|Type|Name|Description|
|---------|---------------------------|-------------------------------------|
| char[4] | file_magic                |File identifier magic, always "FTXC"
| u32     | length_binary_section     |Length of the binary section in bytes
| u16     | n_texture_cell            |Number of texture cells in this file.|
| u16     | n_palettes                |Number of palettes in this file.|
| u32     | offset_texture_cell_descs |Offset into the binary section to the start of the array of TextureCellDesc structs. |
| u32     | offset_palettes           |Offset to the color palettes section, relative to the end of this header. Each color is a 16-bit depth color.|
| u32     | offset_name_table         |Offset to an array of offsets into the binary section. The offsets point to null-terminated strings. The names are stored in the same order as the texture cells, so the same index can be used for both arrays. Used for debugging, and this value should be null if the table is not included in the file. |

All offsets are relative to the start of this binary section.

## TextureCellDesc
|Type|Name|Description|
|----|----|-----------|
|u32|offset_within_texture_data|Offset into raw texture data section.|
|u16|palette_index| Palette index.|
|u8|texture_width| Texture width in pixels.|
|u8|texture_height| Texture height in pixels.|