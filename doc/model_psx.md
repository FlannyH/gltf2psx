# Flan's PSX Model file specification
[Back to main page.](../README.md)

## Model File (.msh)
This file contains a model with a certain amount of submeshes. Submeshes could be used for things like level sections, or different variations of models
| Type | Name               | Description                        |
|------|--------------------|------------------------------------|
| char[4] | file_magic                |File identifier magic, always "FMSH"
| u32  | n_submeshes        | Number of submeshes in this model. |
| u32  | offset_mesh_desc   | Offset into the binary section to the start of the array of MeshDesc structs.|
| u32  | offset_vertex_data | Offset into the binary section to the start of the raw VertexPSX data.        |

All offsets are relative to the start of this binary section.

## MeshDesc
| Type | Description                       |
|------|-----------------------------------|
| u16  | First vertex index for this model |
| u16  | Number of vertices for this model |

## VertexPSX
| Type | Description             |
|------|-------------------------|
| i16  |Position X               |
| i16  |Position Y               |
| i16  |Position Z               |
| u8   |Color R                  |
| u8   |Color G                  |
| u8   |Color B                  |
| u8   |Texture Coordinate U     |
| u8   |Texture Coordinate V     |
| u8   |Texture collection cell index. Only the first vertex's index is actually used. |