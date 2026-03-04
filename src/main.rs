use rand::RngExt;
use rdev::{Event, EventType, Key, listen};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use terminal_size::terminal_size;
use utilprint::{TextEffects, utilprint};

#[derive(Clone, Copy, PartialEq)]
enum CobraEffect {
    Blow,
    Grow,
    PowerUp,
}

#[derive(Clone)]
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
    Edge,
}

struct ThingOnScreen {
    position: Position,
    value: String,
    effect: Option<CobraEffect>,
    kind: ThingKind,
}

impl ThingOnScreen {
    fn from_kind_at_pos(kind: ThingKind, position: Position) -> Self {
        match kind {
            ThingKind::Food => Self {
                position,
                kind,
                effect: Some(CobraEffect::Grow),
                value: String::from("@Y#25C9"),
            },
            ThingKind::Drug => Self {
                position,
                kind,
                effect: Some(CobraEffect::PowerUp),
                value: String::from("@M#2605"),
            },
            ThingKind::Rock => Self {
                position,
                kind,
                effect: Some(CobraEffect::Blow),
                value: String::from("@W#2620"),
            },
            ThingKind::Cobra => Self {
                position,
                kind,
                effect: None,
                value: String::from("@G#2501"),
            },
            _ => Self {
                position,
                kind,
                effect: None,
                value: String::new(),
            },
        }
    }

    fn get_cobra_pixel(
        prev_pos: Option<&Position>,
        position: Position,
        next_pos: Option<&Position>,
    ) -> Self {
        let value = Cobra::get_value(prev_pos, &position, next_pos);
        Self {
            position,
            kind: ThingKind::Cobra,
            effect: None,
            value,
        }
    }

    fn get_idx(&self) -> (usize, usize) {
        (self.position.y as usize, self.position.x as usize)
    }

    fn get_edge(x: u32, y: u32, height: u32, width: u32) -> Option<Self> {
        let mut value = String::from("@W");
        if x == 0 && y == 0 {
            value += "#2554"
        } else if x == width - 1 && y == height - 1 {
            value += "#255D"
        } else if x == 0 && y == height - 1 {
            value += "#255A"
        } else if y == 0 && x == width - 1 {
            value += "#2557"
        } else if x == 0 || x == width - 1 {
            value += "#2551"
        } else if y == 0 || y == height - 1 {
            value += "#2550"
        }
        if value.contains("#") {
            Some(Self {
                effect: None,
                position: Position { x, y },
                kind: ThingKind::Edge,
                value,
            })
        } else {
            None
        }
    }

    fn gen_at_the_field(kind: ThingKind, field: &Field, cobra: &Cobra) -> Self {
        let position = Position::gen_without_collision(field, cobra);
        Self::from_kind_at_pos(kind, position)
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
            body,
            head_dir: Direction::Right,
            state: CobraState::Alive,
            lives,
        };
        cobra.reset(field);
        cobra
    }

    fn get_value(
        prev_pos: Option<&Position>,
        pos: &Position,
        next_pos: Option<&Position>,
    ) -> String {
        let mut value = String::from("@G");

        if let Some(pp) = prev_pos
            && let Some(np) = next_pos
        {
            if pp.x != np.x && pp.y == np.y {
                // horizontal
                value += "#2501";
            } else if pp.x == np.x && pp.y != np.y {
                // vertical
                value += "#2503";
            } else if (pp.x == pos.x && pp.y > pos.y && np.x > pos.x && np.y == pos.y)
                || (pp.x < pos.x && pp.y == pos.y && np.x == pos.x && np.y > pos.y)
            {
                // bottom-left corner
                value += "#2517";
            } else if (pp.x == pos.x && pp.y < pos.y && np.x > pos.x && np.y == pos.y)
                || (pp.x > pos.x && pp.y == pos.y && np.x == pos.x && np.y < pos.y)
            {
                // top-right corner
                value += "#250F";
            } else if (pp.x < pos.x && pp.y == pos.y && np.x == pos.x && np.y < pos.y)
                || (pp.x == pos.x && pp.y < pos.y && np.x < pos.x && np.y == pos.y)
            {
                // top-left corner
                value += "#2513";
            } else if (pp.x < pos.x && pp.y == pos.y && np.x == pos.x && np.y > pos.y)
                || (pp.x == pos.x && pp.y > pos.y && np.x < pos.x && np.y == pos.y)
            {
                // bottom-right corner
                value += "#251B";
            }
        } else if let Some(pp) = prev_pos
            && next_pos.is_none()
        {
            // is head
            if pp.y == pos.y {
                // horizontal
                value += "#2501";
            } else {
                // vertical
                value += "#2503";
            }
        } else if let Some(np) = next_pos
            && prev_pos.is_none()
        {
            // is tail
            if np.y == pos.y {
                value += "#2501";
            } else {
                value += "#2503";
            }
        }
        value
    }

    fn die(&mut self, field: &Field) {
        self.lives -= 1;
        self.reset(field);
    }

    fn reset(&mut self, field: &Field) {
        self.body.clear();
        self.head_dir = Direction::Right;
        self.state = CobraState::Alive;
        let center_pos = field.get_center();
        let mut head_pos = center_pos.clone();
        let mut tail_pos = center_pos.clone();
        let body = ThingOnScreen::from_kind_at_pos(ThingKind::Cobra, center_pos);
        head_pos.x += 1;
        let head = ThingOnScreen::get_cobra_pixel(Some(&body.position), head_pos, None);
        tail_pos.x -= 1;
        let tail = ThingOnScreen::get_cobra_pixel(None, tail_pos, Some(&body.position));
        self.body.push(tail);
        self.body.push(body);
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
        let neck_i = self.body.len() - 1;
        let neck = &self.body[neck_i];
        let new_neck_pos = neck.position.clone();
        let mut pos = neck.position.clone();

        // Move tail
        match self.head_dir {
            Direction::Up => pos.y -= 1,
            Direction::Down => pos.y += 1,
            Direction::Right => pos.x += 1,
            Direction::Left => pos.x -= 1,
        }
        // Handles edge collision
        let mut effect: Option<CobraEffect> = None;
        if pos.x == field.width || pos.y == field.height || pos.x == 0 || pos.y == 0 {
            self.state = CobraState::Dead;
            effect = Some(CobraEffect::Blow);
        }

        // Check for thing colision
        for thing in &field.things {
            if thing.collide(&pos) {
                effect = thing.effect;
                break;
            }
        }
        match effect {
            Some(CobraEffect::Blow) => self.state = CobraState::Dead,
            Some(CobraEffect::Grow) => (),
            Some(CobraEffect::PowerUp) => self.state = CobraState::PoweredUp,
            // move cobra
            _ => (),
        }
        let new_head = ThingOnScreen::get_cobra_pixel(Some(&neck.position), pos, None);
        // Create new neck as can change depending on move Direction
        self.body[neck_i] = ThingOnScreen::get_cobra_pixel(
            Some(&self.body[neck_i].position),
            new_neck_pos,
            Some(&new_head.position),
        );
        self.body.push(new_head);
        self.body.remove(0);
        effect
    }
}

struct Field {
    edges: Vec<ThingOnScreen>,
    things: Vec<ThingOnScreen>,
    height: u32,
    width: u32,
}

impl Field {
    fn get_center(&self) -> Position {
        Position {
            x: self.width / 2,
            y: self.height / 2,
        }
    }

    fn get_edges(height: u32, width: u32) -> Vec<ThingOnScreen> {
        let mut edges = Vec::new();
        for i in 0..width {
            for j in 0..height {
                let edge = ThingOnScreen::get_edge(i, j, height, width);
                if let Some(e) = edge {
                    edges.push(e);
                }
            }
        }
        edges
    }

    fn new_empty(height: u32, width: u32) -> Self {
        let things: Vec<ThingOnScreen> = Vec::new();
        Self {
            edges: Self::get_edges(height, width),
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
            self.things.push(rock);
        }
        // Add food
        let nfood = usize::from(level);
        for _ in 0..nfood {
            let food = ThingOnScreen::gen_at_the_field(ThingKind::Food, self, cobra);
            self.things.push(food);
        }
        // Add Drug
        let drug = ThingOnScreen::gen_at_the_field(ThingKind::Drug, self, cobra);
        self.things.push(drug);
    }
}

struct GameState {
    score: i32,
    tick: i32,
    current_level: Level,
    field: Field,
    cobra: Cobra,
    set_exit: bool,
    last_key: Arc<Mutex<Option<EventType>>>,
}

impl GameState {
    fn init(height: u32, width: u32, last_key: Arc<Mutex<Option<EventType>>>) -> Self {
        let field = Field::new_empty(height - 4, width - 2);
        let cobra = Cobra::new(&field, 3);
        Self {
            score: 0,
            tick: 0,
            current_level: Level::new(1),
            field,
            cobra,
            set_exit: false,
            last_key,
        }
    }

    fn game_over(&mut self) {
        self.clear_screen();
        utilprint("Game Over! Press R to try again!".lover())
    }

    fn bye(&mut self) {
        self.clear_screen();
        println!("You pressed Q, Good bye!!!");
    }

    fn handle_keys(&mut self) {
        if let Some(key) = &*self.last_key.lock().unwrap() {
            match key {
                EventType::KeyPress(Key::UpArrow) => self.cobra.head_dir = Direction::Up,
                EventType::KeyPress(Key::DownArrow) => self.cobra.head_dir = Direction::Down,
                EventType::KeyPress(Key::RightArrow) => self.cobra.head_dir = Direction::Right,
                EventType::KeyPress(Key::LeftArrow) => self.cobra.head_dir = Direction::Left,
                EventType::KeyPress(Key::KeyQ) => self.set_exit = true,
                _ => (),
            };
        }
    }

    fn kill_cobra(&mut self) {
        if self.cobra.lives == 0 {
            self.game_over();
        } else {
            self.clear_screen();
            println!("You died, restarting level!");
            std::thread::sleep(std::time::Duration::from_secs(2));
            self.cobra.die(&self.field);
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
            let (iy, ix) = thing.get_idx();
            grid[iy][ix] = Some(thing);
        }
        for thing in &self.cobra.body {
            let (iy, ix) = thing.get_idx();
            grid[iy][ix] = Some(thing);
        }
        for thing in &self.field.edges {
            let (iy, ix) = thing.get_idx();
            grid[iy][ix] = Some(thing)
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
        self.handle_keys();
        if self.set_exit {
            self.bye();
            return;
        }
        self.render();
        let effect = self.cobra.move_cobra(&self.field);
        let mut power_up = false;

        if let Some(CobraEffect::Blow) = effect {
            self.kill_cobra();
        } else if let Some(CobraEffect::PowerUp) = effect {
            power_up = true;
        }

        let cobra_speed = self.current_level.speed(power_up) as f32;
        let mut min_wait = 1000.0;
        if cobra_speed > 0.0 {
            min_wait /= cobra_speed;
        }
        let dur = Duration::from_secs_f32(min_wait);
        thread::sleep(dur);
        self.tick += 1
    }
}

static LAST_KEY: once_cell::sync::Lazy<Arc<Mutex<Option<EventType>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

fn main() {
    let (w, h) = terminal_size().unwrap();
    let callback = move |event: Event| {
        if let EventType::KeyPress(_) = event.event_type {
            let last_key = Arc::clone(&LAST_KEY);
            let mut key = last_key.lock().unwrap();
            *key = Some(event.event_type);
        }
    };
    let last_key = Arc::clone(&LAST_KEY);
    let mut state = GameState::init(h.0 as u32, w.0 as u32, last_key);
    state.reset_level();
    let handle = thread::spawn(move || {
        // Code to run in the new thread
        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error)
        }
    });
    loop {
        state.next_tick();
        if state.set_exit {
            println!("Exiting game loop!");
            break;
        }
    }
    handle.join().unwrap();
    println!("Bye!");
    std::process::exit(0);
}
