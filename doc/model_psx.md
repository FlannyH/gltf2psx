# Flan's PSX Model file specification
## Model File (.msh)
This file contains a model with a certain amount of submeshes. Submeshes could be used for things like level sections, or different variations of models
| Type     | Description |
|----------|-------------|
| uint32_t | Number of submeshes in this model. |
| uint32_t[] | Array of submesh offsets. The number of offsets is equal to the number of submeshes. |
| uint16_t[] | Array of submesh vertex counts. The length of this array is equal to the number of submeshes, but padded to make sure the VertexPSX data is aligned to 4 bytes. |
| VertexPSX | Raw vertex data, to be indexed using the submesh offsets and vertex counts.|

## VertexPSX
| Type     | Description |
|----------|-------------|
|int16_t|Position X|
|int16_t|Position Y|
|int16_t|Position Z|
|uint8_t|Color R|
|uint8_t|Color G|
|uint8_t|Color B|
|uint8_t|Texture Coordinate U|
|uint8_t|Texture Coordinate V|
|uint8_t|Index into texture atlas|