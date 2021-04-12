use newport_engine::{ Engine, WindowEvent };
use newport_graphics::Graphics;
use newport_gpu as gpu;
use gpu::*;

use newport_os::input::*;

use newport_math::{ Color, Matrix4, Vector2, Vector3, srgb_to_linear };

pub use egui::*;

use std::mem::size_of;
use std::sync::Arc;

static SHADER_SOURCE: &str = "
    #define NULL 0
    ByteAddressBuffer all_buffers[]  : register(t0);
    Texture2D         all_textures[] : register(t1);
    SamplerState      all_samplers[] : register(s2);
    struct Constants {
        float4x4 view;
    };
    [[vk::push_constant]] Constants constants;

    struct Vertex {
        float3 position : POSITION;
        float2 uv       : TEXCOORD;
        float4 color    : COLOR;
        uint texture     : TEXTURE;
    };
    struct Vertex_Out {
        float2 uv       : TEXCOORD;
        float4 color    : COLOR;
        uint texture     : TEXTURE;
        
        float4 position : SV_POSITION;
    };

    Vertex_Out main_vs( Vertex IN ){
        Vertex_Out OUT;
        OUT.uv      = IN.uv;
        OUT.texture = IN.texture;
        OUT.color   = IN.color;

        OUT.position = mul(constants.view, float4(IN.position, 1.0));

        OUT.position.y = -OUT.position.y;

        return OUT;
    }

    struct Pixel_In {
        float2 uv    : TEXCOORD;
        float4 color : COLOR;
        uint texture  : TEXTURE;
    };
    float4 main_ps( Pixel_In IN) : SV_TARGET {
        Texture2D    my_texture = all_textures[IN.texture];
        SamplerState my_sampler = all_samplers[IN.texture];
        
        return IN.color * my_texture.Sample(my_sampler, IN.uv, 0);
    }
";

struct EguiTexture {
    gpu:  gpu::Texture,
    egui: Arc<egui::Texture>,
}

pub struct Egui {
    context:  CtxRef,
    input:    Option<RawInput>,
    
    pipeline: Pipeline,
    texture:  Option<EguiTexture>,
}

#[allow(dead_code)]
struct DrawConstants {
    view: Matrix4,
}

impl Egui {
    pub fn new() -> Self {
        let engine = Engine::as_ref();

        let graphics = engine.module::<Graphics>().unwrap();
        let device = graphics.device();

        let vertex_main = "main_vs".to_string();
        let pixel_main  = "main_ps".to_string();
        
        let vertex_bin = shaders::compile("vertex.hlsl", SHADER_SOURCE, &vertex_main, ShaderVariant::Vertex).unwrap();
        let pixel_bin  = shaders::compile("pixel.hlsl", SHADER_SOURCE, &pixel_main, ShaderVariant::Pixel).unwrap();

        let vertex_shader = device.create_shader(&vertex_bin[..], ShaderVariant::Vertex, vertex_main).unwrap();
        let pixel_shader  = device.create_shader(&pixel_bin[..], ShaderVariant::Pixel, pixel_main).unwrap();

        let pipeline_desc = PipelineBuilder::new_graphics(graphics.backbuffer_render_pass())
            .shaders(vec![vertex_shader, pixel_shader])
            .vertex::<GuiVertex>()
            .enable_blend()
            .dst_alpha_blend(BlendFactor::OneMinusSrcAlpha)
            .push_constant_size::<DrawConstants>()
            .build();

        let pipeline = device.create_pipeline(pipeline_desc).unwrap();

        Self {
            context: CtxRef::default(),
            input:   None,

            pipeline: pipeline,
            texture:  None,
        }
    }

    pub fn process_input(&mut self, event: &WindowEvent) {
        if self.input.is_none() {
            self.input = Some(RawInput::default());
        }
        let input = self.input.as_mut().unwrap();

        let engine = Engine::as_ref();
        let dpi = engine.window().dpi();

        match event {
            WindowEvent::Char(c) => {
                let mut s = String::new();
                s.push(*c);

                input.events.push(Event::Text(s));
            },
            WindowEvent::Key{ key, pressed } => {
                let key = match *key {
                    KEY_DOWN  => Some(Key::ArrowDown), 
                    KEY_LEFT  => Some(Key::ArrowLeft), 
                    KEY_RIGHT => Some(Key::ArrowRight), 
                    KEY_UP    => Some(Key::ArrowUp), 

                    KEY_ESCAPE      => Some(Key::Escape),
                    KEY_TAB         => Some(Key::Tab),
                    KEY_BACKSPACE   => Some(Key::Backspace),
                    KEY_ENTER       => Some(Key::Enter),
                    KEY_SPACE       => Some(Key::Space),
                    KEY_INSERT      => Some(Key::Insert),
                    KEY_DELETE      => Some(Key::Delete),
                    KEY_HOME        => Some(Key::Home),
                    KEY_END         => Some(Key::End),
                    
                    // TODO: PageUp
                    // TODO: PageDown

                    KEY_0 => Some(Key::Num0),
                    KEY_1 => Some(Key::Num1),
                    KEY_2 => Some(Key::Num2),
                    KEY_3 => Some(Key::Num3),
                    KEY_4 => Some(Key::Num4),
                    KEY_5 => Some(Key::Num5),
                    KEY_6 => Some(Key::Num6),
                    KEY_7 => Some(Key::Num7),
                    KEY_8 => Some(Key::Num8),
                    KEY_9 => Some(Key::Num9),

                    KEY_A => Some(Key::A),
                    KEY_B => Some(Key::B), 
                    KEY_C => Some(Key::C), 
                    KEY_D => Some(Key::D), 
                    KEY_E => Some(Key::E), 
                    KEY_F => Some(Key::F), 
                    KEY_G => Some(Key::G), 
                    KEY_H => Some(Key::H), 
                    KEY_I => Some(Key::I), 
                    KEY_J => Some(Key::J), 
                    KEY_K => Some(Key::K), 
                    KEY_L => Some(Key::L), 
                    KEY_M => Some(Key::M), 
                    KEY_N => Some(Key::N), 
                    KEY_O => Some(Key::O), 
                    KEY_P => Some(Key::P), 
                    KEY_Q => Some(Key::Q), 
                    KEY_R => Some(Key::R), 
                    KEY_S => Some(Key::S), 
                    KEY_T => Some(Key::T), 
                    KEY_U => Some(Key::U), 
                    KEY_V => Some(Key::V), 
                    KEY_W => Some(Key::W), 
                    KEY_X => Some(Key::X), 
                    KEY_Y => Some(Key::Y), 
                    KEY_Z => Some(Key::Z), 
                    _ => None,
                };

                if key.is_none() {
                    return;
                }
                let key = key.unwrap();
                input.events.push(Event::Key{
                    key:        key,
                    pressed:    *pressed,
                    modifiers:  Default::default(),
                });
            },
            WindowEvent::MouseButton{ mouse_button, pressed, position } => {
                let button = match *mouse_button {
                    MOUSE_BUTTON_LEFT   => Some(PointerButton::Primary),
                    MOUSE_BUTTON_MIDDLE => Some(PointerButton::Middle),
                    MOUSE_BUTTON_RIGHT  => Some(PointerButton::Secondary),
                    _ => None,
                };
                if button.is_none() {
                    return;
                }
                let button = button.unwrap();

                input.events.push(Event::PointerButton{
                    pos: pos2(position.0 as f32 / dpi, position.1 as f32 / dpi),
                    button:     button,
                    pressed:    *pressed,
                    modifiers:  Default::default(),
                });
            },
            WindowEvent::MouseMove(x, y) => {
                input.events.push(Event::PointerMoved(pos2(*x as f32 / dpi, *y as f32 / dpi)));
            }
            _ => {}
        }
    }

    pub fn begin_frame(&mut self, dt: f32) {
        let engine = Engine::as_ref();
        let viewport = engine.window().size();

        let dpi = engine.window().dpi();

        let mut input = self.input.take().unwrap_or_default();
        input.screen_rect = Some(Rect::from_min_max(pos2(0.0, 0.0), pos2(viewport.0 as f32 / dpi, viewport.1 as f32 / dpi)));
        input.predicted_dt = dt;
        input.pixels_per_point = Some(dpi);   

        self.context.begin_frame(input);
    }

    pub fn end_frame(&mut self, ) -> (Output, Vec<ClippedMesh>) {
        let (output, shapes) = self.context.end_frame();

        let engine = Engine::as_ref();
        let graphics = engine.module::<Graphics>().unwrap();
        let device = graphics.device();

        let texture = self.context.texture();
        if self.texture.is_none() || self.texture.is_some() && self.texture.as_ref().unwrap().egui.version != texture.version {
            let pixel_buffer = device.create_buffer(
                BufferUsage::TRANSFER_SRC, 
                MemoryType::HostVisible, 
                texture.pixels.len() * 4,
            ).unwrap();

            let mut pixels = Vec::with_capacity(texture.pixels.len());
            for it in texture.pixels.iter() {
                let mut color: u32 = (*it as u32) << 24;
                color |= (*it as u32) << 16;
                color |= (*it as u32) << 8;
                color |= *it as u32;


                pixels.push(color);
            }
            pixel_buffer.copy_to(&pixels[..]);

            let gpu_texture = device.create_texture(
                TextureUsage::TRANSFER_DST | TextureUsage::SAMPLED,
                MemoryType::DeviceLocal, 
                Format::RGBA_U8_SRGB,
                texture.width as u32,
                texture.height as u32,
                1,
                Wrap::Clamp,
                Filter::Nearest,
                Filter::Nearest
            ).unwrap();

            let mut gfx = device.create_graphics_context().unwrap();
            {
                gfx.begin();
                gfx.resource_barrier_texture(&gpu_texture, gpu::Layout::Undefined, gpu::Layout::TransferDst);
                gfx.copy_buffer_to_texture(&gpu_texture, &pixel_buffer);
                gfx.resource_barrier_texture(&gpu_texture, gpu::Layout::TransferDst, gpu::Layout::ShaderReadOnly);
                gfx.end();
            }
    
            let receipt = device.submit_graphics(vec![gfx], &[]);
            receipt.wait();

            self.texture = Some(EguiTexture{
                gpu: gpu_texture,
                egui: texture,
            });
        }

        (output, self.context.tessellate(shapes))
    }

    pub fn draw(&mut self, clipped_meshes: Vec<ClippedMesh>, gfx: &mut GraphicsContext) {
        if clipped_meshes.len() == 0 {
            return;
        }

        let engine = Engine::as_ref();
        let graphics = engine.module::<Graphics>().unwrap();
        let device = graphics.device();

        let texture = self.texture.as_ref().unwrap();

        let dpi = engine.window().dpi();

        // Load all vertex data into one giant buffer
        let mut vertex_buffer_len = 0;
        let mut index_buffer_len = 0;
        for it in clipped_meshes.iter() {
            vertex_buffer_len += it.1.vertices.len();
            index_buffer_len  += it.1.indices.len();
        }

        let mut vertices = Vec::with_capacity(vertex_buffer_len);
        let mut indices = Vec::with_capacity(index_buffer_len);

        let mut indice_start = 0;
        for it in clipped_meshes.iter() {
            for vertex in it.1.vertices.iter() {
                let tex = match it.1.texture_id {
                    TextureId::Egui => texture.gpu.bindless().unwrap(),
                    TextureId::User(num) => num as u32,
                };

                let r = srgb_to_linear(vertex.color.r() as f32 / 255.0);
                let g = srgb_to_linear(vertex.color.g() as f32 / 255.0);
                let b = srgb_to_linear(vertex.color.b() as f32 / 255.0);
                let a = vertex.color.a() as f32 / 255.0;
                let color = Color::new(r, g, b, a);

                vertices.push(GuiVertex{
                    position: Vector2::new(vertex.pos.x, vertex.pos.y),
                    uv:       Vector2::new(vertex.uv.x,  vertex.uv.y),
                    color:    color,
                    tex:      tex,
                });
            }

            for index in it.1.indices.iter() {
                indices.push(index + indice_start);
            }

            indice_start += it.1.vertices.len() as u32;
        }

        let vertex_buffer = device.create_buffer(BufferUsage::VERTEX, MemoryType::HostVisible, vertex_buffer_len * size_of::<GuiVertex>()).unwrap();
        vertex_buffer.copy_to(&vertices[..]);

        let index_buffer = device.create_buffer(BufferUsage::INDEX, MemoryType::HostVisible, index_buffer_len * size_of::<u32>()).unwrap();
        index_buffer.copy_to(&indices[..]);

        let viewport = engine.window().size();
        let viewport = Vector2::new(viewport.0 as f32 / dpi, viewport.1 as f32 / dpi);

        let proj = Matrix4::ortho(viewport.x, viewport.y, 1000.0, 0.1);
        let view = Matrix4::translate(Vector3::new(-viewport.x / 2.0, -viewport.y / 2.0, 0.0));

        gfx.bind_pipeline(&self.pipeline);
        gfx.bind_vertex_buffer(&vertex_buffer);
        gfx.bind_index_buffer(&index_buffer);
        gfx.push_constants(DrawConstants{
            view: proj * view,
        });
        gfx.draw_indexed(indices.len(), 0);
    }

    pub fn ctx(&self) -> &CtxRef {
        &self.context
    }
}

#[allow(dead_code)]
struct GuiVertex {
    position: Vector2,
    uv:       Vector2,
    color:    Color,
    tex:      u32,
}

impl Vertex for GuiVertex {
    fn attributes() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute::Vector2,
            VertexAttribute::Vector2,
            VertexAttribute::Color,
            VertexAttribute::Uint32,
        ]
    }
}