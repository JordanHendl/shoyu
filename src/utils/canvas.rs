use dashi::utils::*;
use dashi::*;
use serde::{Deserialize, Serialize};
use std::fs;
extern crate unzip3;
use self::unzip3::Unzip3;

#[derive(Serialize, Deserialize)]
struct CanvasAttachment {
    name: String,
    format: Format,
    samples: SampleCount,
    load_op: LoadOp,
    store_op: StoreOp,
    stencil_load_op: LoadOp,
    stencil_store_op: StoreOp,
    clear_color: [f32; 4],
}

#[derive(Serialize, Deserialize)]
struct CanvasJSONInfo {
    name: String,
    viewport: Viewport,
    color_attachments: Vec<CanvasAttachment>,
    depth_stencil: Option<CanvasAttachment>,
}

pub enum StaticCanvasProfile {
    SIMPLE, // One color & depth attachment.
}

pub struct Canvas {
    name: String,
    viewport: Viewport,
    color_images: Vec<Handle<Image>>,
    color_views: Vec<Handle<ImageView>>,
    depth: Option<Handle<Image>>,
    render_pass: Handle<RenderPass>,
}

impl Canvas {
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }
    pub fn render_pass(&self) -> Handle<RenderPass> {
        return self.render_pass;
    }

    pub fn name(&self) -> String {
        return self.name.clone();
    }

    pub fn color_attachment(&self, idx: u32) -> Handle<ImageView> {
        self.color_views[idx as usize]
    }

    pub fn from_json(ctx: &mut Context, path: &str) -> Self {
        let json_data = fs::read_to_string(path).expect("Failed to read JSON for Canvas!");
        let info: CanvasJSONInfo =
            serde_json::from_str(&json_data).expect("Failed to read Canvas from JSON!");

        // Fn to convert CanvasAttachment -> tuple (img, view, dashi attachment)
        let mut attach_to_tuple = |a: CanvasAttachment| {
            let img = ctx
                .make_image(&ImageInfo {
                    debug_name: &a.name,
                    dim: [info.viewport.area.w as u32, info.viewport.area.h as u32, 1],
                    format: a.format,
                    mip_levels: 1,
                    initial_data: None,
                })
                .unwrap();

            let view = ctx
                .make_image_view(&ImageViewInfo {
                    debug_name: &a.name,
                    img,
                    layer: 0,
                    mip_level: 0,
                })
                .unwrap();

            let attachment = Attachment {
                view,
                samples: a.samples,
                load_op: a.load_op,
                store_op: a.store_op,
                stencil_load_op: a.stencil_load_op,
                stencil_store_op: a.stencil_store_op,
                clear_color: a.clear_color,
            };

            return (img, view, attachment);
        };

        let colors: Vec<(Handle<Image>, Handle<ImageView>, Attachment)> = info
            .color_attachments
            .into_iter()
            .map(|a| attach_to_tuple(a))
            .collect();

        let (imgs, views, attachs): (Vec<_>, Vec<_>, Vec<_>) = colors.iter().cloned().unzip3();

        let (depth, view, depth_attach) = match info.depth_stencil {
            Some(a) => {
                let (img, view, attachment) = attach_to_tuple(a);
                (Some(img), Some(view), Some(attachment))
            }
            None => (None, None, None),
        };

        let render_pass = ctx
            .make_render_pass(&RenderPassInfo {
                viewport: info.viewport,
                color_attachments: &attachs,
                depth_stencil_attachment: depth_attach.as_ref(),
            })
            .unwrap();

        Self {
            viewport: info.viewport,
            color_images: imgs,
            depth,
            render_pass,
            name: info.name,
            color_views: views,
        }
    }

    pub fn new_static(
        ctx: &mut Context,
        width: u32,
        height: u32,
        profile: StaticCanvasProfile,
    ) -> Self {
        match profile {
            StaticCanvasProfile::SIMPLE => {
                let fb = ctx
                    .make_image(&ImageInfo {
                        debug_name: "color_attachment",
                        dim: [width, height, 1],
                        format: Format::RGBA8,
                        mip_levels: 1,
                        initial_data: None,
                    })
                    .unwrap();

                let fb_view = ctx
                    .make_image_view(&ImageViewInfo {
                        debug_name: "color_attachment",
                        img: fb,
                        ..Default::default()
                    })
                    .unwrap();

                let depth = ctx
                    .make_image(&ImageInfo {
                        debug_name: "Depth Image",
                        dim: [width, height, 1],
                        format: Format::D24S8,
                        mip_levels: 1,
                        initial_data: None,
                    })
                    .expect("Unable to make loaded GPU image!");

                let depth_view = ctx
                    .make_image_view(&ImageViewInfo {
                        debug_name: "Depth View",
                        img: depth,
                        layer: 0,
                        mip_level: 0,
                    })
                    .expect("Unable to make loaded GPU image view!");

                return Self {
                    name: "STATIC".to_string(),
                    viewport: Viewport {
                        area: FRect2D {
                            w: width as f32,
                            h: height as f32,
                            ..Default::default()
                        },
                        scissor: Rect2D {
                            w: width,
                            h: height,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    color_images: vec![fb],
                    depth: Some(depth),
                    render_pass: ctx
                        .make_render_pass(&RenderPassInfo {
                            viewport: Viewport {
                                area: FRect2D {
                                    w: width as f32,
                                    h: height as f32,
                                    ..Default::default()
                                },
                                scissor: Rect2D {
                                    w: width,
                                    h: height,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            color_attachments: &[Attachment {
                                view: fb_view,
                                samples: SampleCount::S1,
                                load_op: LoadOp::Clear,
                                store_op: StoreOp::Store,
                                stencil_load_op: LoadOp::DontCare,
                                stencil_store_op: StoreOp::DontCare,
                                clear_color: [0.0, 0.0, 0.0, 1.0],
                            }],
                            depth_stencil_attachment: Some(&Attachment {
                                view: depth_view,
                                samples: SampleCount::S1,
                                load_op: LoadOp::Clear,
                                store_op: StoreOp::Store,
                                stencil_load_op: LoadOp::DontCare,
                                stencil_store_op: StoreOp::DontCare,
                                clear_color: [1.0, 1.0, 1.0, 1.0],
                            }),
                        })
                        .unwrap(),
                    color_views: vec![fb_view],
                };
            }
        }
    }
}
