use crate::context::{consts, Context};
use crate::core::*;
use crate::math::*;
use crate::ImageEffect;

///
/// Defines which channels (red, green, blue, alpha and depth) to clear when starting to write to a
/// [render target](crate::RenderTarget) or the [screen](crate::Screen) and which values they are set to
/// (the values must be between 0 and 1).
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ClearState {
    pub red: Option<f32>,
    pub green: Option<f32>,
    pub blue: Option<f32>,
    pub alpha: Option<f32>,
    pub depth: Option<f32>,
}

impl ClearState {
    ///
    /// Nothing will be cleared.
    ///
    pub const fn none() -> Self {
        Self {
            red: None,
            green: None,
            blue: None,
            alpha: None,
            depth: None,
        }
    }

    ///
    /// The depth will be cleared to the given value.
    ///
    pub const fn depth(depth: f32) -> Self {
        Self {
            red: None,
            green: None,
            blue: None,
            alpha: None,
            depth: Some(depth),
        }
    }

    ///
    /// The color channels (red, green, blue and alpha) will be cleared to the given values.
    ///
    pub const fn color(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red: Some(red),
            green: Some(green),
            blue: Some(blue),
            alpha: Some(alpha),
            depth: None,
        }
    }

    ///
    /// Both the color channels (red, green, blue and alpha) and depth will be cleared to the given values.
    ///
    pub const fn color_and_depth(red: f32, green: f32, blue: f32, alpha: f32, depth: f32) -> Self {
        Self {
            red: Some(red),
            green: Some(green),
            blue: Some(blue),
            alpha: Some(alpha),
            depth: Some(depth),
        }
    }
}

impl Default for ClearState {
    fn default() -> Self {
        Self::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0)
    }
}

///
/// The screen render target which is essential to get something on the screen (see the [write function](Screen::write)).
///
pub struct Screen {}

impl Screen {
    ///
    /// Call this function and make a render call (for example on some [object](crate::object))
    /// in the **render** closure to render something to the screen.
    ///
    pub fn write<F: FnOnce() -> Result<(), Error>>(
        context: &Context,
        clear_state: &ClearState,
        render: F,
    ) -> Result<(), Error> {
        context.bind_framebuffer(consts::DRAW_FRAMEBUFFER, None);
        clear(context, clear_state);
        render()?;
        Ok(())
    }

    ///
    /// Returns the RGB color values from the screen as a list of bytes (one byte for each color channel).
    /// Only available on desktop.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_color(context: &Context, viewport: Viewport) -> Result<Vec<u8>, Error> {
        // TODO: Possible to change format
        let mut pixels = vec![0u8; viewport.width * viewport.height * 3];
        context.bind_framebuffer(consts::READ_FRAMEBUFFER, None);
        context.read_pixels_with_u8_data(
            viewport.x as u32,
            viewport.y as u32,
            viewport.width as u32,
            viewport.height as u32,
            consts::RGB,
            consts::UNSIGNED_BYTE,
            &mut pixels,
        );
        Ok(pixels)
    }

    ///
    /// Returns the depth values from the screen as a list of 32-bit floats.
    /// Only available on desktop.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_depth(context: &Context, viewport: Viewport) -> Result<Vec<f32>, Error> {
        // TODO: Possible to change format
        let mut pixels = vec![0f32; viewport.width * viewport.height];
        context.bind_framebuffer(consts::READ_FRAMEBUFFER, None);
        context.read_pixels_with_f32_data(
            viewport.x as u32,
            viewport.y as u32,
            viewport.width as u32,
            viewport.height as u32,
            consts::DEPTH_COMPONENT,
            consts::FLOAT,
            &mut pixels,
        );
        Ok(pixels)
    }
}

///
/// Use a render target to render into a texture ([color](crate::ColorTargetTexture2D), [depth](DepthTargetTexture2D) or both).
/// Can be created each time it is needed.
///
pub struct RenderTarget<'a, 'b> {
    context: Context,
    id: crate::context::Framebuffer,
    color_texture: Option<&'a ColorTargetTexture2D>,
    depth_texture: Option<&'b DepthTargetTexture2D>,
}

impl<'a, 'b> RenderTarget<'a, 'b> {
    ///
    /// Constructs a new render target that enables rendering into the given
    /// [color](crate::ColorTargetTexture2D) and [depth](DepthTargetTexture2D) textures.
    ///
    pub fn new(
        context: &Context,
        color_texture: &'a ColorTargetTexture2D,
        depth_texture: &'b DepthTargetTexture2D,
    ) -> Result<Self, Error> {
        Ok(Self {
            context: context.clone(),
            id: new_framebuffer(context)?,
            color_texture: Some(color_texture),
            depth_texture: Some(depth_texture),
        })
    }

    ///
    /// Constructs a new render target that enables rendering into the given
    /// [color](crate::ColorTargetTexture2D) texture.
    ///
    pub fn new_color(
        context: &Context,
        color_texture: &'a ColorTargetTexture2D,
    ) -> Result<Self, Error> {
        Ok(Self {
            context: context.clone(),
            id: new_framebuffer(context)?,
            color_texture: Some(color_texture),
            depth_texture: None,
        })
    }

    ///
    /// Constructs a new render target that enables rendering into the given [depth](DepthTargetTexture2D) texture.
    ///
    pub fn new_depth(
        context: &Context,
        depth_texture: &'b DepthTargetTexture2D,
    ) -> Result<Self, Error> {
        Ok(Self {
            context: context.clone(),
            id: new_framebuffer(context)?,
            color_texture: None,
            depth_texture: Some(depth_texture),
        })
    }

    ///
    /// Renders whatever rendered in the **render** closure into the textures defined at construction.
    /// Before writing, the textures are cleared based on the given clear state.
    ///
    pub fn write<F: FnOnce() -> Result<(), Error>>(
        &self,
        clear_state: &ClearState,
        render: F,
    ) -> Result<(), Error> {
        self.bind()?;
        clear(
            &self.context,
            &ClearState {
                red: self.color_texture.and(clear_state.red),
                green: self.color_texture.and(clear_state.green),
                blue: self.color_texture.and(clear_state.blue),
                alpha: self.color_texture.and(clear_state.alpha),
                depth: self.depth_texture.and(clear_state.depth),
            },
        );
        render()?;
        if let Some(color_texture) = self.color_texture {
            color_texture.generate_mip_maps();
        }
        Ok(())
    }

    ///
    /// Copies the content of the color and depth textures in this render target to the screen.
    ///
    /// # Errors
    /// Will return an error if this render target is not constructed with both a color and depth
    /// texture.
    ///
    pub fn copy_to_screen(&self, viewport: Viewport) -> Result<(), Error> {
        if self.color_texture.is_none() || self.depth_texture.is_none() {
            Err(Error::FailedToCopyFromRenderTarget {message: "Cannot copy depth and color when the render target does not have a color and depth texture.".to_owned()})?;
        }
        Screen::write(&self.context, &ClearState::none(), || {
            let effect = get_copy_effect(&self.context)?;
            effect.use_texture(self.color_texture.unwrap(), "colorMap")?;
            effect.use_texture(self.depth_texture.unwrap(), "depthMap")?;
            effect.apply(
                RenderStates {
                    cull: CullType::Back,
                    depth_test: DepthTestType::Always,
                    ..Default::default()
                },
                viewport,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    ///
    /// Copies the content of the color texture in this render target to the screen.
    ///
    /// # Errors
    /// Will return an error if this render target is not constructed with a color texture.
    ///
    pub fn copy_color_to_screen(&self, viewport: Viewport) -> Result<(), Error> {
        if self.color_texture.is_none() {
            Err(Error::FailedToCopyFromRenderTarget {
                message: "Cannot copy color when the render target does not have a color texture."
                    .to_owned(),
            })?;
        }
        Screen::write(&self.context, &ClearState::none(), || {
            let effect = get_copy_effect(&self.context)?;
            effect.use_texture(self.color_texture.unwrap(), "colorMap")?;
            effect.apply(
                RenderStates {
                    cull: CullType::Back,
                    depth_test: DepthTestType::Always,
                    write_mask: WriteMask::COLOR,
                    ..Default::default()
                },
                viewport,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    ///
    /// Copies the content of the depth texture in this render target to the screen.
    ///
    /// # Errors
    /// Will return an error if this render target is not constructed with a depth texture.
    ///
    pub fn copy_depth_to_screen(&self, viewport: Viewport) -> Result<(), Error> {
        if self.depth_texture.is_none() {
            Err(Error::FailedToCopyFromRenderTarget {
                message: "Cannot copy depth when the render target does not have a depth texture."
                    .to_owned(),
            })?;
        }
        Screen::write(&self.context, &ClearState::none(), || {
            let effect = get_copy_effect(&self.context)?;
            effect.use_texture(self.depth_texture.unwrap(), "depthMap")?;
            effect.apply(
                RenderStates {
                    cull: CullType::Back,
                    depth_test: DepthTestType::Always,
                    write_mask: WriteMask::DEPTH,
                    ..Default::default()
                },
                viewport,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    ///
    /// Copies the content of the color and depth textures in this render target to another render target.
    ///
    /// # Errors
    /// Will return an error if this render target is not constructed with both a color and depth
    /// texture.
    ///
    pub fn copy(&self, other: &Self, viewport: Viewport) -> Result<(), Error> {
        if self.color_texture.is_none()
            || self.depth_texture.is_none()
            || other.color_texture.is_none()
            || other.depth_texture.is_none()
        {
            Err(Error::FailedToCopyFromRenderTarget {message: "Cannot copy depth and color when the render target does not have a color and depth texture.".to_owned()})?;
        }
        other.write(&ClearState::none(), || {
            let effect = get_copy_effect(&self.context)?;
            effect.use_texture(self.color_texture.unwrap(), "colorMap")?;
            effect.use_texture(self.depth_texture.unwrap(), "depthMap")?;
            effect.apply(
                RenderStates {
                    cull: CullType::Back,
                    depth_test: DepthTestType::Always,
                    ..Default::default()
                },
                viewport,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    ///
    /// Copies the content of the color texture in this render target to another render target.
    ///
    /// # Errors
    /// Will return an error if this render target is not constructed with a color
    /// texture.
    ///
    pub fn copy_color(&self, other: &Self, viewport: Viewport) -> Result<(), Error> {
        if self.color_texture.is_none() || other.color_texture.is_none() {
            Err(Error::FailedToCopyFromRenderTarget {
                message: "Cannot copy color when the render target does not have a color texture."
                    .to_owned(),
            })?;
        }
        other.write(&ClearState::none(), || {
            let effect = get_copy_effect(&self.context)?;
            effect.use_texture(self.color_texture.unwrap(), "colorMap")?;
            effect.apply(
                RenderStates {
                    cull: CullType::Back,
                    depth_test: DepthTestType::Always,
                    write_mask: WriteMask::COLOR,
                    ..Default::default()
                },
                viewport,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    ///
    /// Copies the content of the depth texture in this render target to another render target.
    ///
    /// # Errors
    /// Will return an error if this render target is not constructed with a depth texture.
    ///
    pub fn copy_depth(&self, other: &Self, viewport: Viewport) -> Result<(), Error> {
        if self.depth_texture.is_none() || other.depth_texture.is_none() {
            Err(Error::FailedToCopyFromRenderTarget {
                message: "Cannot copy depth when the render target does not have a depth texture."
                    .to_owned(),
            })?;
        }
        other.write(&ClearState::none(), || {
            let effect = get_copy_effect(&self.context)?;
            effect.use_texture(self.depth_texture.unwrap(), "depthMap")?;
            effect.apply(
                RenderStates {
                    cull: CullType::Back,
                    depth_test: DepthTestType::Always,
                    write_mask: WriteMask::DEPTH,
                    ..Default::default()
                },
                viewport,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    fn bind(&self) -> Result<(), Error> {
        self.context
            .bind_framebuffer(consts::DRAW_FRAMEBUFFER, Some(&self.id));
        if let Some(tex) = self.color_texture {
            self.context.draw_buffers(&[consts::COLOR_ATTACHMENT0]);
            tex.bind_as_color_target(0);
        }
        if let Some(tex) = self.depth_texture {
            tex.bind_as_depth_target();
        }
        #[cfg(feature = "debug")]
        check(&self.context)?;
        Ok(())
    }
}

impl Drop for RenderTarget<'_, '_> {
    fn drop(&mut self) {
        self.context.delete_framebuffer(Some(&self.id));
    }
}

///
/// Same as [RenderTarget](crate::RenderTarget) except that the render target contains an
/// array of [color textures](crate::ColorTargetTexture2DArray) and [depth textures](crate::DepthTargetTexture2DArray).
///
pub struct RenderTargetArray<'a, 'b> {
    context: Context,
    id: crate::context::Framebuffer,
    color_texture: Option<&'a ColorTargetTexture2DArray>,
    depth_texture: Option<&'b DepthTargetTexture2DArray>,
}

impl<'a, 'b> RenderTargetArray<'a, 'b> {
    ///
    /// Constructs a new render target array that enables rendering into the given
    /// [color](crate::ColorTargetTexture2DArray) and [depth](DepthTargetTexture2DArray) array textures.
    ///
    pub fn new(
        context: &Context,
        color_texture: &'a ColorTargetTexture2DArray,
        depth_texture: &'b DepthTargetTexture2DArray,
    ) -> Result<Self, Error> {
        Ok(Self {
            context: context.clone(),
            id: new_framebuffer(context)?,
            color_texture: Some(color_texture),
            depth_texture: Some(depth_texture),
        })
    }

    pub fn new_color(
        context: &Context,
        color_texture: &'a ColorTargetTexture2DArray,
    ) -> Result<Self, Error> {
        Ok(Self {
            context: context.clone(),
            id: new_framebuffer(context)?,
            color_texture: Some(color_texture),
            depth_texture: None,
        })
    }

    pub fn new_depth(
        context: &Context,
        depth_texture: &'b DepthTargetTexture2DArray,
    ) -> Result<Self, Error> {
        Ok(Self {
            context: context.clone(),
            id: new_framebuffer(context)?,
            color_texture: None,
            depth_texture: Some(depth_texture),
        })
    }

    ///
    /// Renders whatever rendered in the **render** closure into the textures defined at construction
    /// and defined by the input parameters **color_layers** and **depth_layer**.
    /// Output at location *i* defined in the fragment shader is written to the color texture layer at the *ith* index in **color_layers**.
    /// The depth is written to the depth texture defined by **depth_layer**.
    /// Before writing, the textures are cleared based on the given clear state.
    ///
    pub fn write<F: FnOnce() -> Result<(), Error>>(
        &self,
        clear_state: &ClearState,
        color_layers: &[usize],
        depth_layer: usize,
        render: F,
    ) -> Result<(), Error> {
        self.bind(Some(color_layers), Some(depth_layer))?;
        clear(
            &self.context,
            &ClearState {
                red: self.color_texture.and(clear_state.red),
                green: self.color_texture.and(clear_state.green),
                blue: self.color_texture.and(clear_state.blue),
                alpha: self.color_texture.and(clear_state.alpha),
                depth: self.depth_texture.and(clear_state.depth),
            },
        );
        render()?;
        if let Some(color_texture) = self.color_texture {
            color_texture.generate_mip_maps();
        }
        Ok(())
    }

    pub fn copy_to_screen(
        &self,
        color_layer: usize,
        depth_layer: usize,
        viewport: Viewport,
    ) -> Result<(), Error> {
        if self.color_texture.is_none() || self.depth_texture.is_none() {
            Err(Error::FailedToCopyFromRenderTarget {message: "Cannot copy depth and color when the render target does not have a color and depth texture.".to_owned()})?;
        }
        Screen::write(&self.context, &ClearState::none(), || {
            let effect = get_copy_array_effect(&self.context)?;
            effect.use_texture(self.color_texture.unwrap(), "colorMap")?;
            effect.use_texture(self.depth_texture.unwrap(), "depthMap")?;
            effect.use_uniform_int("colorLayer", &(color_layer as i32))?;
            effect.use_uniform_int("depthLayer", &(depth_layer as i32))?;
            effect.apply(
                RenderStates {
                    cull: CullType::Back,
                    depth_test: DepthTestType::Always,
                    ..Default::default()
                },
                viewport,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn copy_color_to_screen(
        &self,
        color_layer: usize,
        viewport: Viewport,
    ) -> Result<(), Error> {
        if self.color_texture.is_none() {
            Err(Error::FailedToCopyFromRenderTarget {
                message: "Cannot copy color when the render target does not have a color texture."
                    .to_owned(),
            })?;
        }
        let effect = get_copy_array_effect(&self.context)?;
        Screen::write(&self.context, &ClearState::none(), || {
            effect.use_texture(self.color_texture.unwrap(), "colorMap")?;
            effect.use_uniform_int("colorLayer", &(color_layer as i32))?;
            effect.apply(
                RenderStates {
                    cull: CullType::Back,
                    depth_test: DepthTestType::Always,
                    write_mask: WriteMask::COLOR,
                    ..Default::default()
                },
                viewport,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn copy_depth_to_screen(
        &self,
        depth_layer: usize,
        viewport: Viewport,
    ) -> Result<(), Error> {
        if self.depth_texture.is_none() {
            Err(Error::FailedToCopyFromRenderTarget {
                message: "Cannot copy depth when the render target does not have a depth texture."
                    .to_owned(),
            })?;
        }
        Screen::write(&self.context, &ClearState::none(), || {
            let effect = get_copy_array_effect(&self.context)?;
            effect.use_texture(self.depth_texture.unwrap(), "depthMap")?;
            effect.use_uniform_int("depthLayer", &(depth_layer as i32))?;
            effect.apply(
                RenderStates {
                    cull: CullType::Back,
                    depth_test: DepthTestType::Always,
                    write_mask: WriteMask::DEPTH,
                    ..Default::default()
                },
                viewport,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn copy(
        &self,
        color_layer: usize,
        depth_layer: usize,
        other: &RenderTarget,
        viewport: Viewport,
    ) -> Result<(), Error> {
        if self.color_texture.is_none()
            || self.depth_texture.is_none()
            || other.color_texture.is_none()
            || other.depth_texture.is_none()
        {
            Err(Error::FailedToCopyFromRenderTarget {message: "Cannot copy depth and color when the render target does not have a color and depth texture.".to_owned()})?;
        }
        other.write(&ClearState::none(), || {
            let effect = get_copy_array_effect(&self.context)?;
            effect.use_texture(self.color_texture.unwrap(), "colorMap")?;
            effect.use_texture(self.depth_texture.unwrap(), "depthMap")?;
            effect.use_uniform_int("colorLayer", &(color_layer as i32))?;
            effect.use_uniform_int("depthLayer", &(depth_layer as i32))?;
            effect.apply(
                RenderStates {
                    cull: CullType::Back,
                    depth_test: DepthTestType::Always,
                    ..Default::default()
                },
                viewport,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn copy_color(
        &self,
        color_layer: usize,
        other: &RenderTarget,
        viewport: Viewport,
    ) -> Result<(), Error> {
        if self.color_texture.is_none() || other.color_texture.is_none() {
            Err(Error::FailedToCopyFromRenderTarget {
                message: "Cannot copy color when the render target does not have a color texture."
                    .to_owned(),
            })?;
        }
        other.write(&ClearState::none(), || {
            let effect = get_copy_array_effect(&self.context)?;
            effect.use_texture(self.color_texture.unwrap(), "colorMap")?;
            effect.use_uniform_int("colorLayer", &(color_layer as i32))?;
            effect.apply(
                RenderStates {
                    cull: CullType::Back,
                    depth_test: DepthTestType::Always,
                    write_mask: WriteMask::COLOR,
                    ..Default::default()
                },
                viewport,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn copy_depth(
        &self,
        depth_layer: usize,
        other: &RenderTarget,
        viewport: Viewport,
    ) -> Result<(), Error> {
        if self.depth_texture.is_none() || other.depth_texture.is_none() {
            Err(Error::FailedToCopyFromRenderTarget {
                message: "Cannot copy depth when the render target does not have a depth texture."
                    .to_owned(),
            })?;
        }
        other.write(&ClearState::none(), || {
            let effect = get_copy_array_effect(&self.context)?;
            effect.use_texture(self.depth_texture.unwrap(), "depthMap")?;
            effect.use_uniform_int("depthLayer", &(depth_layer as i32))?;
            effect.apply(
                RenderStates {
                    cull: CullType::Back,
                    depth_test: DepthTestType::Always,
                    write_mask: WriteMask::DEPTH,
                    ..Default::default()
                },
                viewport,
            )?;
            Ok(())
        })?;
        Ok(())
    }

    fn bind(
        &self,
        color_layers: Option<&[usize]>,
        depth_layer: Option<usize>,
    ) -> Result<(), Error> {
        self.context
            .bind_framebuffer(consts::DRAW_FRAMEBUFFER, Some(&self.id));
        if let Some(color_texture) = self.color_texture {
            if let Some(color_layers) = color_layers {
                self.context.draw_buffers(
                    &(0..color_layers.len())
                        .map(|i| consts::COLOR_ATTACHMENT0 + i as u32)
                        .collect::<Vec<u32>>(),
                );
                for channel in 0..color_layers.len() {
                    color_texture.bind_as_color_target(color_layers[channel], channel);
                }
            }
        }
        if let Some(depth_texture) = self.depth_texture {
            if let Some(depth_layer) = depth_layer {
                depth_texture.bind_as_depth_target(depth_layer);
            }
        }
        #[cfg(feature = "debug")]
        check(&self.context)?;
        Ok(())
    }
}

impl Drop for RenderTargetArray<'_, '_> {
    fn drop(&mut self) {
        self.context.delete_framebuffer(Some(&self.id));
    }
}

fn new_framebuffer(context: &Context) -> Result<crate::context::Framebuffer, Error> {
    Ok(context
        .create_framebuffer()
        .ok_or_else(|| Error::FailedToCreateFramebuffer {
            message: "Failed to create framebuffer".to_string(),
        })?)
}

#[cfg(feature = "debug")]
fn check(context: &Context) -> Result<(), Error> {
    context
        .check_framebuffer_status()
        .or_else(|message| Err(Error::FailedToCreateFramebuffer { message }))
}

fn clear(context: &Context, clear_state: &ClearState) {
    Program::set_write_mask(
        context,
        WriteMask {
            red: clear_state.red.is_some(),
            green: clear_state.green.is_some(),
            blue: clear_state.blue.is_some(),
            alpha: clear_state.alpha.is_some(),
            depth: clear_state.depth.is_some(),
        },
    );
    let clear_color = clear_state.red.is_some()
        || clear_state.green.is_some()
        || clear_state.blue.is_some()
        || clear_state.alpha.is_some();
    if clear_color {
        context.clear_color(
            clear_state.red.unwrap_or(0.0),
            clear_state.green.unwrap_or(0.0),
            clear_state.blue.unwrap_or(0.0),
            clear_state.alpha.unwrap_or(1.0),
        );
    }
    if let Some(depth) = clear_state.depth {
        context.clear_depth(depth);
    }
    context.clear(if clear_color && clear_state.depth.is_some() {
        consts::COLOR_BUFFER_BIT | consts::DEPTH_BUFFER_BIT
    } else {
        if clear_color {
            consts::COLOR_BUFFER_BIT
        } else {
            consts::DEPTH_BUFFER_BIT
        }
    });
}

fn get_copy_effect(context: &Context) -> Result<&ImageEffect, Error> {
    unsafe {
        static mut COPY_EFFECT: Option<ImageEffect> = None;
        if COPY_EFFECT.is_none() {
            COPY_EFFECT = Some(ImageEffect::new(
                context,
                &"
                uniform sampler2D colorMap;
                uniform sampler2D depthMap;
                in vec2 uv;
                layout (location = 0) out vec4 color;
                void main()
                {
                    color = texture(colorMap, uv);
                    gl_FragDepth = texture(depthMap, uv).r;
                }",
            )?);
        }
        Ok(COPY_EFFECT.as_ref().unwrap())
    }
}

fn get_copy_array_effect(context: &Context) -> Result<&ImageEffect, Error> {
    unsafe {
        static mut COPY_EFFECT: Option<ImageEffect> = None;
        if COPY_EFFECT.is_none() {
            COPY_EFFECT = Some(ImageEffect::new(
                context,
                &"
                uniform sampler2DArray colorMap;
                uniform sampler2DArray depthMap;
                uniform int colorLayer;
                uniform int depthLayer;
                in vec2 uv;
                layout (location = 0) out vec4 color;
                void main()
                {
                    color = texture(colorMap, vec3(uv, colorLayer));
                    gl_FragDepth = texture(depthMap, vec3(uv, depthLayer)).r;
                }",
            )?);
        }
        Ok(COPY_EFFECT.as_ref().unwrap())
    }
}
