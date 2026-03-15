use defmt::{ println };
use heapless::String;
use core::fmt::Write;
use embedded_graphics::{
    Drawable,
    Pixel,
    image::Image,
    mono_font::{ MonoTextStyleBuilder, ascii::FONT_6X10 },
    pixelcolor::BinaryColor,
    prelude::Point,
    text::{ Baseline, Text },
};
use esp_hal::{ Async, i2c::master::I2c, time::{ Instant, Duration } };
use ssd1306::{
    Ssd1306,
    mode::BufferedGraphicsMode,
    prelude::I2CInterface,
    size::DisplaySize128x64,
};

const DISPLAY_WIDTH: i32 = 128;
const DISPLAY_HEIGHT: i32 = 64;
const MAX_ENEMIES: usize = 3;
const MAX_PLAYER_PROJECTILES: usize = 10;
const MAX_ENEMY_PROJECTILES: usize = 30;
const ENEMY_DIRECTION_CHANGE_INTERVAL: Duration = Duration::from_millis(400);

use super::{
    structs::{ Direction, Coord, Projectile, Tag },
    sprites::{ HEART, SHIP, ENEMY, RAW_SHIP, RAW_ENEMY },
};
//
//
pub enum Event {
    BtnPressed(u8),
}

#[derive(PartialEq, Eq)]
pub enum State {
    Loading,
    Idle,
    Playing,
    GameOver,
}

pub struct StateMachine {
    pub state: State,
    pub player: Player,
    pub player_projectiles: [Option<Projectile>; MAX_PLAYER_PROJECTILES],
    pub enemies: [Option<Enemy>; MAX_ENEMIES],
    pub enemy_projectiles: [Option<Projectile>; MAX_ENEMY_PROJECTILES],
    spawning_spots: [Coord; 3],
    pub score: u32,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: State::Loading,
            player: Player::new(),
            player_projectiles: [None; MAX_PLAYER_PROJECTILES],
            enemies: [None; MAX_ENEMIES],
            enemy_projectiles: [None; MAX_ENEMY_PROJECTILES],
            spawning_spots: [
                Coord::new(1, DISPLAY_HEIGHT / 3 - 13),
                Coord::new(1, (DISPLAY_HEIGHT / 3) * 2 - 13),
                Coord::new(1, DISPLAY_HEIGHT - 13),
            ],
            score: 0,
        }
    }

    fn set_state_gameover(
        &mut self,
        display: &mut Ssd1306<
            I2CInterface<I2c<'_, Async>>,
            DisplaySize128x64,
            BufferedGraphicsMode<DisplaySize128x64>
        >
    ) {
        self.state = State::GameOver;
        display.clear_buffer();
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        Text::with_baseline(
            "Game over!",
            Point::new(DISPLAY_WIDTH / 2 - 59 / 2, DISPLAY_HEIGHT / 2 - 6),
            text_style,
            Baseline::Bottom
        )
            .draw(display)
            .expect("Error displaying game over text");
        Text::with_baseline(
            "Press to play again",
            Point::new(DISPLAY_WIDTH / 2 - 112 / 2, DISPLAY_HEIGHT / 2 - 6),
            text_style,
            Baseline::Top
        )
            .draw(display)
            .expect("Error displaying play again text");

        let mut buf: String<10> = String::new(); //heapless stack buffer
        write!(buf, "Score: {}", self.score).unwrap();
        Text::with_baseline(
            &buf,
            Point::new(DISPLAY_WIDTH / 2 - 29, DISPLAY_HEIGHT / 2 + 5),
            text_style,
            Baseline::Top
        )
            .draw(display)
            .expect("Error displaying play again text");
        _ = display.flush();
    }

    pub fn start(
        &mut self,
        display: &mut Ssd1306<
            I2CInterface<I2c<'_, Async>>,
            DisplaySize128x64,
            BufferedGraphicsMode<DisplaySize128x64>
        >
    ) {
        if self.state == State::Loading {
            display.clear_buffer();
            let text_style = MonoTextStyleBuilder::new()
                .font(&FONT_6X10)
                .text_color(BinaryColor::On)
                .build();
            Text::with_baseline(
                "Press to start!",
                Point::new(DISPLAY_WIDTH / 2 - 90 / 2, DISPLAY_HEIGHT / 2 - 6),
                text_style,
                Baseline::Bottom
            )
                .draw(display)
                .expect("Error displaying press to start text");

            draw_player(
                display,
                Coord::new(self.player.shipdata.pos.x, self.player.shipdata.pos.y)
            );
            _ = display.flush();
            self.state = State::Idle;
        }
    }
    pub fn restart(&mut self) {
        self.player.shipdata.hp = 3;
        self.score = 0;
        self.player.shipdata.pos = Coord::new(DISPLAY_WIDTH / 2, DISPLAY_HEIGHT / 2);
        self.enemies = [None; MAX_ENEMIES];
        self.state = State::Playing;
    }

    pub fn event_handler(&mut self, event: Event) {
        match self.state {
            State::Playing => {
                match event {
                    Event::BtnPressed(0) => {
                        self.player.move_pos(Direction::Left, 2);
                    }

                    Event::BtnPressed(1) => {
                        self.player.move_pos(Direction::Right, 2);
                    }
                    Event::BtnPressed(2) => {
                        self.player.move_pos(Direction::Down, 2);
                    }
                    Event::BtnPressed(3) => {
                        self.player.move_pos(Direction::Up, 2);
                    }
                    Event::BtnPressed(4) => {
                        if self.player.last_shot.elapsed() > self.player.reload_cooldown {
                            let shot_pos = if self.player.next_shot == Direction::Left {
                                self.player.next_shot = Direction::Right;
                                Coord::new(
                                    self.player.shipdata.pos.x,
                                    self.player.shipdata.pos.y + self.player.shipdata.height / 2 + 5
                                )
                            } else {
                                self.player.next_shot = Direction::Left;
                                Coord::new(
                                    self.player.shipdata.pos.x,
                                    self.player.shipdata.pos.y + self.player.shipdata.height / 2 - 5
                                )
                            };
                            self.spawn_projectile(
                                Projectile::new(shot_pos, Direction::Left, Tag::Player)
                            );
                            self.player.last_shot = Instant::now();
                        }
                    }
                    _ => (),
                }
            }
            State::GameOver => {
                self.restart();
            }
            State::Idle => {
                self.state = State::Playing;
            }
            _ => (),
        }
    }

    pub fn spawn_enemy(&mut self, pos: Coord) {
        for slot in &mut self.enemies {
            if slot.is_none() {
                *slot = Some(Enemy::new(pos));
                break;
            }
        }
    }

    pub fn update(
        &mut self,
        display: &mut Ssd1306<
            I2CInterface<I2c<'_, Async>>,
            DisplaySize128x64,
            BufferedGraphicsMode<DisplaySize128x64>
        >
    ) {
        match self.state {
            State::Playing => {
                self.update_projectiles();
                self.update_enemies();
                self.update_player(display);
                self.render(display);
            }
            //State::Idle => {}
            _ => (),
        }
    }

    fn spawn_projectile(&mut self, projectile: Projectile) {
        match projectile.tag {
            Tag::Player => {
                for slot in &mut self.player_projectiles {
                    if slot.is_none() {
                        *slot = Some(projectile);
                        break;
                    }
                }
            }
            Tag::Enemy => {
                for slot in &mut self.enemy_projectiles {
                    if slot.is_none() {
                        *slot = Some(projectile);
                        break;
                    }
                }
            }
        }
    }

    fn render(
        &mut self,
        display: &mut Ssd1306<
            I2CInterface<I2c<'_, Async>>,
            DisplaySize128x64,
            BufferedGraphicsMode<DisplaySize128x64>
        >
    ) {
        match self.state {
            State::Playing => {
                display.clear_buffer();
                //player_projectiles
                for slot in self.player_projectiles {
                    if let Some(projectile) = slot {
                        draw_projectile(display, projectile.pos);
                    }
                }
                for slot in self.enemy_projectiles {
                    if let Some(projectile) = slot {
                        draw_projectile(display, projectile.pos);
                    }
                }
                //ENEMIES
                for slot in &self.enemies {
                    if let Some(enemy) = slot {
                        draw_enemy(display, enemy.shipdata.pos);
                    }
                }
                //PLAYER
                draw_player(
                    display,
                    Coord::new(self.player.shipdata.pos.x, self.player.shipdata.pos.y)
                );
                //health
                for i in 0..self.player.shipdata.hp as i32 {
                    let x = i * 8 + i;
                    draw_heart(display, Coord::new(x, 0));
                }
                //score
                draw_score(&self.score, display);
                // flush
                _ = display.flush();
            }
            _ => (),
        }
    }
    fn update_player(
        &mut self,
        display: &mut Ssd1306<
            I2CInterface<I2c<'_, Async>>,
            DisplaySize128x64,
            BufferedGraphicsMode<DisplaySize128x64>
        >
    ) {
        if self.player.shipdata.hp == 0 {
            self.set_state_gameover(display);
        }
    }
    fn next_round(&mut self) {
        for slot in self.spawning_spots {
            self.spawn_enemy(slot);
        }
    }
    fn update_projectiles(&mut self) {
        //Player player_projectiles
        for player_proj_slot in self.player_projectiles.iter_mut() {
            let mut is_remove_projectile = false;
            if let Some(projectile) = player_proj_slot.as_mut() {
                projectile.move_pos(projectile.direction);
                //collisions
                for slot in &mut self.enemies {
                    if let Some(enemy) = slot {
                        if is_ship_hit(projectile, &enemy.shipdata) {
                            enemy.shipdata.hp -= 1;
                            is_remove_projectile = true;
                        }
                    }
                }
                //OOB?
                if is_out_of_bounds(&projectile.pos, 1, 1) {
                    is_remove_projectile = true;
                }
            }
            if is_remove_projectile {
                *player_proj_slot = None;
            }
        }
        //enemy player_projectiles
        for enemy_proj_slot in self.enemy_projectiles.iter_mut() {
            let mut is_remove_projectile = false;
            if let Some(projectile) = enemy_proj_slot {
                projectile.move_pos(projectile.direction);
                if is_ship_hit(projectile, &self.player.shipdata) {
                    if self.player.shipdata.hp > 0 {
                        self.player.shipdata.hp -= 1;
                        is_remove_projectile = true;
                    }
                }
                //OOB?
                if is_out_of_bounds(&projectile.pos, 1, 1) {
                    is_remove_projectile = true;
                }
            }
            if is_remove_projectile {
                *enemy_proj_slot = None;
            }
        }
    }
    fn update_enemies(&mut self) {
        let mut is_any_enemies = false;

        // temporary projectile queue
        let mut pending: [Option<Projectile>; 8] = [None; 8];
        let mut pending_i = 0;

        for enem_slot in &mut self.enemies {
            if let Some(enemy) = enem_slot {
                is_any_enemies = true;

                if enemy.shipdata.hp <= 0 {
                    self.score += 1;
                    *enem_slot = None;
                    continue;
                }

                enemy.move_pos(enemy.move_direction, 1);

                if enemy.last_direction_change.elapsed() > ENEMY_DIRECTION_CHANGE_INTERVAL {
                    enemy.move_direction = match enemy.move_direction {
                        Direction::Up => Direction::Down,
                        Direction::Down => Direction::Up,
                        d => d,
                    };

                    enemy.last_direction_change = Instant::now();
                }

                if enemy.last_shot.elapsed() > enemy.reload_cooldown {
                    let mod_y = if enemy.next_shot == Direction::Left {
                        enemy.next_shot = Direction::Right;
                        -3
                    } else {
                        enemy.next_shot = Direction::Left;
                        2
                    };

                    let proj_pos = Coord::new(
                        enemy.shipdata.pos.x + enemy.shipdata.width,
                        enemy.shipdata.pos.y + enemy.shipdata.height / 2 + mod_y
                    );

                    if pending_i < pending.len() {
                        pending[pending_i] = Some(
                            Projectile::new(proj_pos, Direction::Right, Tag::Enemy)
                        );
                        pending_i += 1;
                    }

                    enemy.last_shot = Instant::now();
                }
            }
        }

        for proj in pending.into_iter().flatten() {
            self.spawn_projectile(proj);
        }

        if !is_any_enemies {
            self.next_round();
        }
    }
}

#[derive(Clone, Copy)]
pub struct ShipData {
    hp: u8,
    pos: Coord,
    width: i32,
    height: i32,
    tag: Tag,
}
impl ShipData {
    fn new(hp: u8, pos: Coord, width: i32, height: i32, tag: Tag) -> Self {
        Self {
            hp,
            pos,
            width,
            height,
            tag,
        }
    }
}
pub struct Player {
    shipdata: ShipData,
    reload_cooldown: Duration,
    next_shot: Direction, //right or left cannon
    last_shot: Instant, //timestamp
}
impl Player {
    pub fn new() -> Self {
        Self {
            shipdata: ShipData::new(
                3,
                Coord::new(DISPLAY_WIDTH / 2 - 9, DISPLAY_HEIGHT / 2),
                16,
                14,
                Tag::Player
            ),
            reload_cooldown: Duration::from_millis(300),
            next_shot: Direction::Left,
            last_shot: Instant::now(),
        }
    }

    pub fn move_pos(&mut self, direction: Direction, speed: u8) {
        for _ in 0..speed {
            let mut new_pos = Coord::new(self.shipdata.pos.x, self.shipdata.pos.y);
            match direction {
                Direction::Up => {
                    new_pos.y -= 1;
                }
                Direction::Down => {
                    new_pos.y += 1;
                }
                Direction::Left => {
                    new_pos.x -= 1;
                }
                Direction::Right => {
                    new_pos.x += 1;
                }
            }
            if !is_out_of_bounds(&new_pos, self.shipdata.width, self.shipdata.height) {
                self.shipdata.pos = new_pos;
            }
        }
    }
}
#[derive(Clone, Copy)]
pub struct Enemy {
    shipdata: ShipData,
    reload_cooldown: Duration,
    next_shot: Direction, //right or left cannon
    last_shot: Instant, //timestamp
    last_direction_change: Instant, //timestamp
    move_direction: Direction,
}
impl Enemy {
    pub fn new(pos: Coord) -> Self {
        Self {
            shipdata: ShipData::new(2, pos, 11, 8, Tag::Enemy),
            reload_cooldown: Duration::from_millis(1500),
            next_shot: Direction::Left,
            last_shot: Instant::now(),
            last_direction_change: Instant::now(),
            move_direction: Direction::Down,
        }
    }

    pub fn move_pos(&mut self, direction: Direction, speed: u8) {
        for _ in 0..speed {
            let mut new_pos = Coord::new(self.shipdata.pos.x, self.shipdata.pos.y);
            match direction {
                Direction::Up => {
                    new_pos.y -= 1;
                }
                Direction::Down => {
                    new_pos.y += 1;
                }
                Direction::Left => {
                    new_pos.x -= 1;
                }
                Direction::Right => {
                    new_pos.x += 1;
                }
            }
            if !is_out_of_bounds(&new_pos, self.shipdata.width, self.shipdata.height) {
                self.shipdata.pos = new_pos;
            }
        }
    }
}

fn is_ship_hit(projectile: &mut Projectile, ship: &ShipData) -> bool {
    let local_x = projectile.pos.x - ship.pos.x;
    let local_y = projectile.pos.y - ship.pos.y;
    if !(local_x >= 0 && local_x < ship.width - 1 && local_y >= 0 && local_y < ship.height - 1) {
        // if projectile isnt inbound
        return false;
    }

    let row_bytes: usize = (ship.width as usize) / 8;

    let byte_index = (local_y as usize) * row_bytes + (local_x as usize) / 8;

    let bit = 7 - (local_x % 8);
    match ship.tag {
        Tag::Player => {
            let pixel_on = ((RAW_SHIP[byte_index] >> bit) & 1) == 1;
            return pixel_on;
        }
        Tag::Enemy => {
            let pixel_on = ((RAW_ENEMY[byte_index] >> bit) & 1) == 1;
            return pixel_on;
        }
    }
}

fn draw_heart(
    display: &mut Ssd1306<
        I2CInterface<I2c<'_, Async>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>
    >,
    coord: Coord
) {
    let heart = Image::new(&HEART, Point::new(coord.x, coord.y));
    heart.draw(display).unwrap();
}

fn draw_enemy(
    display: &mut Ssd1306<
        I2CInterface<I2c<'_, Async>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>
    >,
    coord: Coord
) {
    let enemy = Image::new(&ENEMY, Point::new(coord.x, coord.y));
    enemy.draw(display).unwrap();
}

fn draw_score(
    score: &u32,
    display: &mut Ssd1306<
        I2CInterface<I2c<'_, Async>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>
    >
) {
    let mut buf: String<10> = String::new(); //heapless stack buffer
    write!(buf, "Score: {}", score).unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    Text::with_baseline(&buf, Point::new(DISPLAY_WIDTH - 70, 1), text_style, Baseline::Top)
        .draw(display)
        .expect("Error displaying score text");
}

fn draw_player(
    display: &mut Ssd1306<
        I2CInterface<I2c<'_, Async>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>
    >,
    coord: Coord
) {
    let ship = Image::new(&SHIP, Point::new(coord.x, coord.y));
    ship.draw(display).unwrap();
}

fn draw_projectile(
    display: &mut Ssd1306<
        I2CInterface<I2c<'_, Async>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>
    >,
    pos: Coord
) {
    let projectile = Pixel(Point::new(pos.x, pos.y), BinaryColor::On);
    projectile.draw(display).unwrap();
}

fn is_out_of_bounds(pos: &Coord, width: i32, height: i32) -> bool {
    pos.y < 0 ||
        pos.y + height - 1 >= DISPLAY_HEIGHT ||
        pos.x < 0 ||
        pos.x + width - 1 >= DISPLAY_WIDTH
}
