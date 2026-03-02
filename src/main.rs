use rand::RngExt;
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]

enum CobraEffect {
    Blow,
    Grow,
    PowerUp,
    Walk,
}

enum Shape {
    Triangle,
    Circle,
    Square,
}

struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Copy)]
struct Position {
    x: u8,
    y: u8,
}

impl Position {
    fn new(field: &Field) -> Self {
        let mut rng = rand::rng();
        Self {
            x: rng.random_range(0..=field.width),
            y: rng.random_range(0..=field.height),
        }
    }

    fn gen_without_collision(field: &Field, cobra: &Cobra) -> Self {
        let mut new_pos = Self::new(&field);
        loop {
            if cobra.collide(&new_pos) {
                new_pos = Self::new(&field);
                continue;
            }
            let mut has_collided = false;
            for thing in &field.things {
                if thing.collide(&new_pos) {
                    has_collided = true;
                    break;
                }
            }
            if has_collided {
                new_pos = Self::new(&field);
                continue;
            }
            break;
        }
        new_pos
    }
}

enum ThingKind {
    Food,
    Drug,
    Rock,
    Cobra,
}

struct ThingOnScreen {
    position: Position,
    color: Color,
    shape: Shape,
    effect: CobraEffect,
    kind: ThingKind,
}

impl ThingOnScreen {
    
    fn to_str(&self) -> str {
        match self.kind {
            ThingKind::Food =>"U+1F60B",
            ThingKind::Drug => "U+1F48A",
            ThingKind::Rock => "U+1FAA8",
            ThingKind::Cobra => "U+1F7E9",
        }
    }

    fn new_from_kind(kind: ThingKind, position: Position) -> Self {
        match kind {
            ThingKind::Food => Self {
                position,
                kind,
                color: Color::from_rgb(0, 255, 0),
                shape: Shape::Circle,
                effect: CobraEffect::Grow,
            },
            ThingKind::Drug => Self {
                position,
                kind,
                color: Color::from_rgb(0, 255, 255),
                shape: Shape::Triangle,
                effect: CobraEffect::PowerUp,
            },
            ThingKind::Rock => Self {
                position,
                kind,
                color: Color::from_rgb(255, 255, 255),
                shape: Shape::Square,
                effect: CobraEffect::Blow,
            },
            ThingKind::Cobra => Self {
                position,
                kind,
                color: Color::from_rgb(0, 0, 255),
                shape: Shape::Square,
                effect: CobraEffect::Walk,
            },
        }
    }

    fn gen_at_the_field(kind: ThingKind, field: &Field, cobra: &Cobra) -> Self {
        let position = Position::gen_without_collision(field, cobra);
        Self::new_from_kind(kind, position)
    }

    fn collide(&self, pos: &Position) -> bool {
        pos.x == self.position.x && pos.y == self.position.y
    }
}

struct Level {
    number: u8,
}

impl Level {
    fn new(number: u8) -> Self {
        Self { number }
    }

    fn speed(&self, power_up: bool) -> u32 {
        let mut speed = u32::from(self.number) * 1000;
        if power_up {
            speed += speed / 4
        }
        speed
    }
}

enum Direction {
    Up,
    Down,
    Right,
    Left,
}

enum CobraState {
    Alive,
    PoweredUp,
    Dead,
}

struct Cobra {
    body: Vec<ThingOnScreen>,
    head_dir: Direction,
    state: CobraState,
    lifes: u8,
}

impl Cobra {
    fn new(field: &Field, lifes: u8) -> Self {
        let center_pos = Position {
            x: &field.width / 2,
            y: &field.height / 2,
        };
        let mut body: Vec<ThingOnScreen> = Vec::with_capacity(255);
        let body_part = ThingOnScreen::new_from_kind(ThingKind::Cobra, center_pos);
        body.push(body_part);
        Self {
            body,
            head_dir: Direction::Right,
            state: CobraState::Alive,
            lifes,
        }
    }

    fn collide(&self, pos: &Position) -> bool {
        for body_part in &self.body {
            if pos.x == body_part.position.x && pos.y == body_part.position.y {
                return true;
            }
        }
        false
    }

    fn move_cobra(&mut self, things: &Vec<ThingOnScreen>) -> Option<CobraEffect> {
        let mut head = self.body[self.body.len() - 1].position.clone();
        // Move tail
        match self.head_dir {
            Direction::Up => head.y += 1,
            Direction::Down => head.y -= 1,
            Direction::Right => head.x += 1,
            Direction::Left => head.x -= 1,
        }
        // Check for colision
        let mut effect: Option<CobraEffect> = None;
        for thing in things {
            if thing.collide(&head) {
                effect = Some(thing.effect.clone());
                match thing.effect {
                    CobraEffect::Blow => self.state = CobraState::Dead,
                    CobraEffect::Grow => self
                        .body
                        .push(ThingOnScreen::new_from_kind(ThingKind::Cobra, head)),
                    CobraEffect::PowerUp => self.state = CobraState::PoweredUp,
                    _ => (),
                }
            }
        }
        if effect != Some(CobraEffect::Blow) {
            self.body
                .push(ThingOnScreen::new_from_kind(ThingKind::Cobra, head));
            self.body.remove(0);
        }
        effect
    }
}

struct Field {
    things: Vec<ThingOnScreen>,
    height: u8,
    width: u8,
}

impl Field {
    fn new_empty(height: u8, width: u8) -> Self {
        let things: Vec<ThingOnScreen> = Vec::new();
        Self {
            things,
            height,
            width,
        }
    }

    fn gen_things(&mut self, level: u8, cobra: &Cobra) {
        let nthings: u8 = 4 + (2 ^ level);
        // Add rocks
        for _ in 0..(nthings - level) {
            let rock = ThingOnScreen::gen_at_the_field(ThingKind::Rock, self, cobra);
            self.things.push(rock)
        }
        // Add food
        let nfood = usize::from(level);
        for _ in 0..nfood {
            let food = ThingOnScreen::gen_at_the_field(ThingKind::Food, self, cobra);
            self.things.push(food)
        }
    }
}

struct Pixel {
    value: Option[str]
}

impl Pixel {
    }

#[derive(Debug)]
struct GameState {
    score: i32,
    tick: i32,
    current_level: Level,
    field: Field,
    cobra: Cobra,
}

impl GameState {
    fn init() -> Self {
        let field = Field::new_empty(80, 60);
        let cobra = Cobra::new(&field, 3);
        Self {
            score: 0,
            tick: 0,
            current_level: Level::new(1),
            field,
            cobra,
        }
    }

    fn game_over(&self) {}

    fn loose_life(&self) {}

    fn reset_level(&mut self) {
        self.field
            .gen_things(self.current_level.number, &self.cobra);
    }

    fn power_up(&self) {}

    fn get_field(&mut self) -> Vec<Vec<Pixel>> {
        let width = usize::from(self.field.width);
        let height = usize::from(self.field.height);
        let mut grid: Vec<Vec<Pixel>> = Vec::with_capacity(height);
        for _ in 0..height {
            let mut row: Vec<Pixel> = Vec::with_capacity(width);
            for _ in 0..width {
                row.push(Pixel::None);
            }
            grid.push(row);
        }
        for thing in &self.field.things {
            let ix = usize::from(thing.position.x);
            let iy = usize::from(thing.position.y);
            grid[iy][ix] = Pixel::Some(thing);
        }
        for body_part in &self.cobra.body {
            let ix = usize::from(body_part.position.x);
            let iy = usize::from(body_part.position.y);
            grid[iy][ix] = Pixel::Some(body_part);
        }
        grid
    }

    fn render(&mut self) {
        println!("Game State: {!dbg}" % &self);
        let field = self.get_field();
        for l in field {
            let mut line = String::with_capacity(line.len());
            for p in l {
                if p.is_none() {
                    line += " "
                }
                let p_str = "{}" % p.shape();
                line += p.shape();
            }
            println!("{line}");
        }
    }

    fn next_tick(&mut self) {
        self.cobra.move_cobra(&self.field.things);

        if let CobraState::Dead = self.cobra.state {
            if self.cobra.lifes == 0 {
                self.game_over();
            } else {
                self.cobra.lifes -= 1;
                self.cobra.state = CobraState::Alive;
                self.reset_level();
            }
        } else {
            self.render();
        }
        let mut power_up = false;
        if let CobraState::PoweredUp = self.cobra.state {
            power_up = true;
        }
        let cobra_speed = self.current_level.speed(power_up);
        let mut min_wait: u32 = 2000;
        if cobra_speed > 0 {
            min_wait /= cobra_speed;
        }
        let dur = Duration::from_secs(u64::from(min_wait));
        thread::sleep(dur);
    }
}

fn main() {
    let mut state = GameState::init();
    state.reset_level();
    loop {
        state.next_tick();
    }
}
