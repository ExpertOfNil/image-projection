This repository is for demonstrating some of the difficulties I am having with
texture projection.  The objective is to be able to project a texture onto a
mesh where the knowns are:

1. Camera position in world space
1. Camera intrinsics
1. Camera extrinsics
1. Vertex positions in world space

However, the vertex to texture coordinate mapping is not known.

Demonstrations:
1. `master` branch: projection of a texture onto a plane where textrue
coordinates are computed in the vertex shader.  Distortion occurs at the
diagonal shared by the plane's two triangles.  Here is the result
![result](https://github.com/ExpertOfNil/image-projection/blob/master/res/result.png).
2. `projector` branch: same as `master` except the texture is projected from
a static camera other than the view camera.
