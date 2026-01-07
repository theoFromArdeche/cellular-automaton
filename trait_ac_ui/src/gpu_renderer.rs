use crate::color_scheme::ColorScheme;

use eframe::{egui, glow};
use std::sync::Arc;
use eframe::glow::HasContext;


pub struct GPURenderer {
    gl: Arc<glow::Context>,
    texture: glow::Texture,
    programs: [glow::Program; 4],
    vao: glow::VertexArray,
    vbo: glow::Buffer,
}

impl GPURenderer {
    pub fn new(gl: Arc<glow::Context>) -> Result<Self, String> {
        unsafe {
            let vertex_shader_src = include_str!("shaders/vertex.glsl");
            let fragment_shaders = ColorScheme::SHADERS;
            
            let mut programs = [None, None, None, None];
            for i in 0..fragment_shaders.len() {
                programs[i] = Some(Self::create_program(&gl, vertex_shader_src, fragment_shaders[i])?);
            }
            
            let texture = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            
            let vao = gl.create_vertex_array()?;
            let vbo = gl.create_buffer()?;
            
            Ok(Self {
                gl,
                texture,
                programs: [programs[0].unwrap(), programs[1].unwrap(), programs[2].unwrap(), programs[3].unwrap()],
                vao,
                vbo,
            })
        }
    }
    
    pub unsafe fn create_program(gl: &glow::Context, vs: &str, fs: &str) -> Result<glow::Program, String> {
        let program = unsafe { gl.create_program()? };
        let v = unsafe { gl.create_shader(glow::VERTEX_SHADER)? };
        unsafe { gl.shader_source(v, vs) };
        unsafe { gl.compile_shader(v) } ;
        if unsafe { !gl.get_shader_compile_status(v) } {
            return Err(unsafe { gl.get_shader_info_log(v) });
        }
        
        let f = unsafe { gl.create_shader(glow::FRAGMENT_SHADER)? };
        unsafe { gl.shader_source(f, fs) };
        unsafe { gl.compile_shader(f) };
        if unsafe { !gl.get_shader_compile_status(f) } {
            return Err(unsafe { gl.get_shader_info_log(f) });
        }
        
        unsafe { gl.attach_shader(program, v) };
        unsafe { gl.attach_shader(program, f) };
        unsafe { gl.link_program(program) };
        
        if unsafe {!gl.get_program_link_status(program)} {
            return Err(unsafe { gl.get_program_info_log(program) });
        }
        
        unsafe { gl.delete_shader(v); }
        unsafe { gl.delete_shader(f); }
        Ok(program)
    }
    
    pub fn update_texture(&mut self, width: usize, height: usize, data: &[u8], resize_flag: bool) {
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            
            // Only reallocate if dimensions change
            if resize_flag {
                self.gl.tex_image_2d(
                    glow::TEXTURE_2D, 0, glow::R8 as i32,
                    width as i32, height as i32, 0,
                    glow::RED, glow::UNSIGNED_BYTE, eframe::glow::PixelUnpackData::Slice(None), // Allocate only
                );
            }

            // Fast update
            self.gl.tex_sub_image_2d(
                glow::TEXTURE_2D, 0, 
                0, 0, // x, y offset
                width as i32, height as i32,
                glow::RED, glow::UNSIGNED_BYTE,
                eframe::glow::PixelUnpackData::Slice(Some(data)),
            );
        }
    }
    
    pub fn paint(&self, scheme: usize, rect: egui::Rect, _screen: [f32; 2], 
            scroll_offset: egui::Vec2, content_size: egui::Vec2) {
        unsafe {
            let gl = &self.gl;
            let prog = self.programs[scheme];
            gl.use_program(Some(prog));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.uniform_1_i32(gl.get_uniform_location(prog, "u_texture").as_ref(), 0);
            
            let rect_size = rect.size();
            gl.uniform_2_f32(
                gl.get_uniform_location(prog, "u_rect_size").as_ref(),
                rect_size.x,
                rect_size.y,
            );
            
            let rect_min = rect.min;
            gl.uniform_2_f32(
                gl.get_uniform_location(prog, "u_rect_min").as_ref(),
                rect_min.x,
                rect_min.y,
            );
            
            // Calculate what portion of the texture is visible
            // Clamp to [0, 1] range to avoid repeating
            let tex_min_x = (scroll_offset.x / content_size.x).max(0.0).min(1.0);
            let tex_min_y = (scroll_offset.y / content_size.y).max(0.0).min(1.0);
            let tex_max_x = ((scroll_offset.x + rect_size.x) / content_size.x).max(0.0).min(1.0);
            let tex_max_y = ((scroll_offset.y + rect_size.y) / content_size.y).max(0.0).min(1.0);
            
            // Calculate how much of the viewport actually contains texture
            let visible_content_width = (content_size.x - scroll_offset.x).max(0.0).min(rect_size.x);
            let visible_content_height = (content_size.y - scroll_offset.y).max(0.0).min(rect_size.y);
            
            // Only render where there's actual content
            let render_max_x = rect.min.x + visible_content_width;
            let render_max_y = rect.min.y + visible_content_height;
            
            let verts: [f32; 24] = [
                rect.min.x, rect.min.y, tex_min_x, tex_min_y,
                render_max_x, rect.min.y, tex_max_x, tex_min_y,
                render_max_x, render_max_y, tex_max_x, tex_max_y,
                rect.min.x, rect.min.y, tex_min_x, tex_min_y,
                render_max_x, render_max_y, tex_max_x, tex_max_y,
                rect.min.x, render_max_y, tex_min_x, tex_max_y,
            ];
            
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&verts), glow::STREAM_DRAW);
            
            let pos = gl.get_attrib_location(prog, "a_pos").unwrap();
            let tc = gl.get_attrib_location(prog, "a_tc").unwrap();
            
            gl.enable_vertex_attrib_array(pos);
            gl.vertex_attrib_pointer_f32(pos, 2, glow::FLOAT, false, 16, 0);
            gl.enable_vertex_attrib_array(tc);
            gl.vertex_attrib_pointer_f32(tc, 2, glow::FLOAT, false, 16, 8);
            
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }
}


