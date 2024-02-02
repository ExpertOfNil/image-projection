This repository is for demonstrating some of the difficulties I am having with
texture projection.  The objective is to be able to project a texture onto a
mesh where the knowns are:

1. Camera position in world space
1. Camera intrinsics
1. Camera extrinsics
1. Vertex positions in world space

However, the vertex to texture coordinate mapping is not known.

_Note_: On startup, the diagonal on the plane,
shared by both triangles, runs vertically from the view perpective.

Original Image:
![result](https://github.com/ExpertOfNil/image-projection/blob/master/res/image_projection_test_square.png)

Demonstrations:
1. `master` branch: projection of a texture onto a plane where textrue
coordinates are computed in the vertex shader.  Distortion occurs at the
diagonal shared by the plane's two triangles.  Here is the result
![result](https://github.com/ExpertOfNil/image-projection/blob/master/res/result.png)
2. `projector` branch: same as `master` except the texture is projected from
a static camera other than the view camera.  This branch also includes the cube
which should match the cube in the texture.  The texture fits the cube pretty
well, but you'll notice that the texture seems to be pinched at the corner
closest to the camera, on the plane. Here is the result
![result](https://github.com/ExpertOfNil/image-projection/blob/projector/res/proj_result.png)
