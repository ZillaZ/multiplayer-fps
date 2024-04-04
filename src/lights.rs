use crate::*;
use raylib::prelude::*;

pub struct Light {
    light_type: i32,
    enabled: i32,
    position: Vector3,
    target: Vector3,
    color: Color,
    enabled_loc: i32,
    type_loc: i32,
    position_loc: i32,
    target_loc: i32,
    color_loc: i32,
}

impl Light {
    pub fn new(
        light_type: i32,
        position: Vector3,
        target: Vector3,
        color: Color,
        shader: &mut Shader,
    ) -> Self {
        let light = Self {
            enabled: 1,
            light_type,
            position,
            target,
            color,

            enabled_loc: shader.get_shader_location("lights[0].enabled"),
            type_loc: shader.get_shader_location("lights[0].type"),
            position_loc: shader.get_shader_location("lights[0].position"),
            target_loc: shader.get_shader_location("lights[0].target"),
            color_loc: shader.get_shader_location("lights[0].color"),
        };

        light.update_light_values(shader);

        light
    }

    fn update_light_values(&self, shader: &mut Shader) {
        shader.set_shader_value(self.enabled_loc, self.enabled);
        shader.set_shader_value(self.type_loc, self.light_type);

        // Send to shader light position values
        let position = [self.position.x, self.position.y, self.position.z];
        shader.set_shader_value(self.position_loc, position);

        // Send to shader self target position values
        let target = [self.target.x, self.target.y, self.target.z];
        shader.set_shader_value(self.target_loc, target);

        // Send to shader self color values
        let r: f32 = self.color.r as f32;
        let g: f32 = self.color.g as f32;
        let b: f32 = self.color.b as f32;
        let a: f32 = self.color.a as f32;

        let color = [r / 255.0, g / 255.0, b / 255.0, a / 255.0];
        shader.set_shader_value(self.color_loc, color);
    }
}
