use rand::RngExt;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use terminal_size::terminal_size;
use utilprint::{TextEffects, utilprint};

#[derive(Clone, Copy, PartialEq)]
enum CobraEffect {
    Blow,
    Grow,
    PowerUp,
    Walk,
}

#[derive(Clone, Copy)]
struct Position {
    x: u32,
    y: u32,
}

impl Position {
    fn new(field: &Field) -> Self {
        let mut rng = rand::rng();
        Self {
            x: rng.random_range(0..=(field.width - 1)),
            y: rng.random_range(0..=(field.height - 1)),
        }
    }

    fn gen_without_collision(field: &Field, cobra: &Cobra) -> Self {
        let mut new_pos = Self::new(field);
        loop {
            if cobra.collide(&new_pos) {
                new_pos = Self::new(field);
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
                new_pos = Self::new(field);
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
    value: String,
    effect: CobraEffect,
    kind: ThingKind,
}

impl ThingOnScreen {
    fn new_from_kind(kind: ThingKind, position: Position) -> Self {
        match kind {
            ThingKind::Food => Self {
                position,
                kind,
                effect: CobraEffect::Grow,
                value: String::from("@Y#25C9"),
            },
            ThingKind::Drug => Self {
                position,
                kind,
                effect: CobraEffect::PowerUp,
                value: String::from("@M#2615"),
            },
            ThingKind::Rock => Self {
                position,
                kind,
                effect: CobraEffect::Blow,
                value: String::from("@W#2620"),
            },
            ThingKind::Cobra => Self {
                position,
                kind,
                effect: CobraEffect::Walk,
                value: String::from("@G#2501"),
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
        let mut speed = u32::from(self.number) * 2000;
        if power_up {
            speed += speed / 4
        }
        speed
    }
}

#[derive(Debug)]
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
    lives: u8,
}

impl Cobra {
    fn new(field: &Field, lives: u8) -> Self {
        let body: Vec<ThingOnScreen> = Vec::new();
        let mut cobra = Self {
            body: body,
            head_dir: Direction::Right,
            state: CobraState::Alive,
            lives,
        };
        cobra.reset(field);
        cobra
    }
    
    fn grow(&self) {

    }

    fn die(&mut self, field: &Field) {
        self.lives -= 1;
        self.state = CobraState::Alive;
        self.reset(field);
    }

    fn reset(&mut self, field: &Field) {
        self.body.clear();
        let center_pos = field.get_center();
        let head = ThingOnScreen::new_from_kind(ThingKind::Cobra, center_pos);
        let mut tail_pos = head.position.clone();
        tail_pos.x -= 1;
        let tail = ThingOnScreen::new_from_kind(ThingKind::Cobra, tail_pos);
        self.body.push(tail);
        self.body.push(head);
    }

    fn collide(&self, pos: &Position) -> bool {
        for body_part in &self.body {
            if pos.x == body_part.position.x && pos.y == body_part.position.y {
                return true;
            }
        }
        false
    }

    fn move_cobra(&mut self, field: &Field) -> Option<CobraEffect> {
        let mut head = self.body[self.body.len() - 1].position.clone();

        // Move tail
        match self.head_dir {
            Direction::Up => head.y += 1,
            Direction::Down => head.y -= 1,
            Direction::Right => head.x += 1,
            Direction::Left => head.x -= 1,
        }
        // Handles edge collision
        let mut effect: Option<CobraEffect> = None;
        if head.x < 0 || head.x >= field.width || head.y < 0 || head.y >= field.height {
            self.state = CobraState::Dead;
            effect = Some(CobraEffect::Blow);

        }
        
        // Check for thing colision
        for thing in &field.things {
            if thing.collide(&head) {
                effect = Some(thing.effect);
                break;
            }
        }
        let new_head = ThingOnScreen::new_from_kind(ThingKind::Cobra, head);
        self.body.push(new_head);
        match effect {
            Some(CobraEffect::Blow) => self.state = CobraState::Dead,
            Some(CobraEffect::Grow) => (),
            Some(CobraEffect::PowerUp) => self.state = CobraState::PoweredUp,
            // move cobra
            _ => _ = self.body.remove(0),
        }
        effect

    }
}

struct Field {
    things: Vec<ThingOnScreen>,
    height: u32,
    width: u32,
}

impl Field {
    fn get_center(&self) -> Position {
        Position{
            x: self.width/2,
            y: self.height/2
        }
    }

    fn new_empty(height: u32, width: u32) -> Self {
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

struct GameState {
    score: i32,
    tick: i32,
    current_level: Level,
    field: Field,
    cobra: Cobra,
}

impl GameState {
    fn init(height: u32, width: u32) -> Self {
        let field = Field::new_empty(height - 4, width - 2);
        let cobra = Cobra::new(&field, 3);
        Self {
            score: 0,
            tick: 0,
            current_level: Level::new(1),
            field,
            cobra,
        }
    }

    fn game_over(&mut self) {
        self.clear_screen();
        utilprint("Game Over! Press R to try again!".lover())
    }

    fn kill_cobra(&mut self) {
        if self.cobra.lives == 0 {
            self.game_over();
        } else {
            self.cobra.lives -= 1;
            self.clear_screen();
            println!("You died, restarting level!");
            std::thread::sleep(std::time::Duration::from_secs(2));
            self.reset_level();
        }
    }

    fn reset_level(&mut self) {
        self.cobra.reset(&self.field);
        self.field.things.clear();
        self.field
            .gen_things(self.current_level.number, &self.cobra);
        self.next_tick();
    }

    fn get_field(&mut self) -> Vec<Vec<Option<&ThingOnScreen>>> {
        let mut grid: Vec<Vec<Option<&ThingOnScreen>>> =
            Vec::with_capacity(self.field.height as usize);
        for _ in 0..self.field.height {
            let mut row: Vec<Option<&ThingOnScreen>> =
                Vec::with_capacity(self.field.width as usize);
            for _ in 0..self.field.width {
                row.push(None);
            }
            grid.push(row);
        }
        for thing in &self.field.things {
            let ix = thing.position.x as usize;
            let iy = thing.position.y as usize;
            grid[iy][ix] = Some(thing);
        }
        for body_part in &self.cobra.body {
            let ix = body_part.position.x as usize;
            let iy = body_part.position.y as usize;
            grid[iy][ix] = Some(body_part);
        }
        grid
    }

    fn clear_screen(&mut self) {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
    }

    fn render(&mut self) {
        self.clear_screen();
        println!(
            "Field: {}x{}  Score: {}   lives: {}     Tick: {}   Head Dir: {:?}",
            &self.field.height,
            &self.field.width,
            &self.score,
            &self.cobra.lives,
            &self.tick,
            &self.cobra.head_dir
        );

        let field = self.get_field();
        for l in field {
            let mut line = String::with_capacity(l.len());
            for p in l {
                if let Some(pixel) = p {
                    line += &pixel.value[..]
                } else {
                    line += " "
                }
            }
            utilprint(line);
        }
    }

    fn next_tick(&mut self) {
        self.render();
        let effect = self.cobra.move_cobra(&self.field);
        let mut power_up = false;

        if let Some(CobraEffect::Blow) = effect {
            self.kill_cobra();
        } else if let Some(CobraEffect::PowerUp) = effect {
            power_up = true;
        }

        let cobra_speed = self.current_level.speed(power_up);
        let mut min_wait: u32 = 1000;
        if cobra_speed > 0 {
            min_wait /= cobra_speed;
        }
        let dur = Duration::from_secs(u64::from(min_wait));
        thread::sleep(dur);
        self.tick += 1
    }
}

fn main() {
    let (w, h) = terminal_size().unwrap();
    let mut state = GameState::init(h.0 as u32, w.0 as u32);
    state.reset_level();
    loop {
        state.next_tick();
    }
}
