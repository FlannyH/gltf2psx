# Flan's PSX Model file specification
[Back to main page.](../README.md)

## Model File (.msh)
This file contains a model with a certain amount of submeshes. Submeshes could be used for things like level sections, or different variations of models
| Type    | Name               | Description                                                                   |
| ------- | ------------------ | ----------------------------------------------------------------------------- |
| char[4] | file_magic         | File identifier magic, always "FMSH"                                          |
| u32     | n_submeshes        | Number of submeshes in this model.                                            |
| u32     | offset_mesh_desc   | Offset into the binary section to the start of the array of MeshDesc structs. |
| u32     | offset_vertex_data | Offset into the binary section to the start of the raw VertexPSX data.        |

All offsets are relative to the start of this binary section.

## MeshDesc
| Type | Name         | Description                         |
| ---- | ------------ | ----------------------------------- |
| u16  | vertex_start | First vertex index for this model   |
| u16  | n_vertices   | Number of vertices for this model   |
| i16  | x_min        | Axis aligned bounding box minimum X |
| i16  | x_max        | Axis aligned bounding box maximum X |
| i16  | y_min        | Axis aligned bounding box minimum Y |
| i16  | y_max        | Axis aligned bounding box maximum Y |
| i16  | z_min        | Axis aligned bounding box minimum Z |
| i16  | z_max        | Axis aligned bounding box maximum Z |

## VertexPSX
| Type | Name          | Description                                                                    |
| ---- | ------------- | ------------------------------------------------------------------------------ |
| i16  | x             | Position X                                                                     |
| i16  | y             | Position Y                                                                     |
| i16  | z             | Position Z                                                                     |
| u8   | r             | Color R                                                                        |
| u8   | g             | Color G                                                                        |
| u8   | b             | Color B                                                                        |
| u8   | u             | Texture Coordinate U                                                           |
| u8   | v             | Texture Coordinate V                                                           |
| u8   | texture_index | Texture collection cell index. Only the first vertex's index is actually used. |