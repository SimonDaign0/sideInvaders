#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Enemy,
    Player,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}
impl Coord {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Projectile {
    pub pos: Coord,
    pub direction: Direction,
    pub tag: Tag,
}
impl Projectile {
    pub fn new(pos: Coord, direction: Direction, tag: Tag) -> Self {
        Self {
            pos,
            direction,
            tag,
        }
    }
    pub fn move_pos(&mut self, direction: Direction) {
        let mut new_pos = Coord::new(self.pos.x, self.pos.y);
        match direction {
            Direction::Left => {
                new_pos.x -= 1;
            }
            Direction::Right => {
                new_pos.x += 1;
            }
            _ => (),
        }
        self.pos = new_pos;
    }
}
