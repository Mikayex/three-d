use crate::camera::*;
use crate::core::*;
use crate::definition::*;
use crate::math::*;
use crate::object::*;

///
/// Three arrows indicating the three main axes; the x-axis (red), the y-axis (green) and the z-axis (blue).
/// Used for easily debugging where objects are placed in the 3D world.
///
pub struct Axes {
    x: Mesh,
    y: Mesh,
    z: Mesh,
}

impl Axes {
    pub fn new(context: &Context, radius: f32, length: f32) -> Result<Self, Error> {
        Ok(Self {
            x: Mesh::new(context, &CPUMesh::arrow(radius, length, 16))?,
            y: Mesh::new(context, &CPUMesh::arrow(radius, length, 16))?,
            z: Mesh::new(context, &CPUMesh::arrow(radius, length, 16))?,
        })
    }

    ///
    /// Render the axes.
    /// Must be called in a render target render function,
    /// for example in the callback function of [Screen::write](crate::Screen::write).
    /// The transformation can be used to position, orientate and scale the axes.
    ///
    pub fn render(
        &self,
        viewport: Viewport,
        transformation: &Mat4,
        camera: &camera::Camera,
    ) -> Result<(), Error> {
        self.x.render_with_color(
            &vec4(1.0, 0.0, 0.0, 1.0),
            RenderStates::default(),
            viewport,
            transformation,
            camera,
        )?;
        self.y.render_with_color(
            &vec4(0.0, 1.0, 0.0, 1.0),
            RenderStates::default(),
            viewport,
            &(transformation * Mat4::from_angle_z(degrees(90.0))),
            camera,
        )?;
        self.z.render_with_color(
            &vec4(0.0, 0.0, 1.0, 1.0),
            RenderStates::default(),
            viewport,
            &(transformation * Mat4::from_angle_y(degrees(-90.0))),
            camera,
        )?;

        Ok(())
    }
}
