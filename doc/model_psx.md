# Flan's PSX Model file specification
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
|u32| Offset in raw texture indices array.|
|u8| Texture width in pixels.|
|u8| Texture height in pixels.|
|u8| Index into palette array.|
|u8| Reserved.|