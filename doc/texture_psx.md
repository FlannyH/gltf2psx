# Flan's PSX Model file specification
[Back to main page.](../README.md)
## Model File (.msh)
This file contains a model with a certain amount of submeshes. Submeshes could be used for things like level sections, or different variations of models
| Type        | Description                        |
|-------------|------------------------------------|
| u32         | Number of submeshes in this model. |
| MeshDesc[]  | Array of mesh descriptions.        |
| VertexPSX[] | Raw vertex data                    |

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
| u8   |Texture atlas cell index. Only the first vertex's index is actually used. |