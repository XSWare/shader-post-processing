use crate::vertex::Vertex;

pub struct Rectangle {
    pub top: f32,
    pub left: f32,
    pub height: f32,
    pub width: f32,
}

impl Rectangle {
    pub fn create_vertices(&self) -> Vec<Vertex> {
        let mut verticis = vec![];
        verticis.push(Vertex {
            position: [self.left, self.top],
            tex_coords: [0.0, 0.0],
        });

        verticis.push(Vertex {
            position: [self.left, self.top - self.height],
            tex_coords: [0.0, 1.0],
        });

        verticis.push(Vertex {
            position: [self.left + self.width, self.top - self.height],
            tex_coords: [1.0, 1.0],
        });

        verticis.push(Vertex {
            position: [self.left + self.width, self.top],
            tex_coords: [1.0, 0.0],
        });

        verticis
    }

    pub fn get_indices() -> &'static [u16] {
        const INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

        return &INDICES;
    }
}
