use egaku2d::*;
use std::collections::HashSet;
use std::{thread, time};
use rand::Rng;

const ARENA_W :usize = 100;
const ARENA_SCALE :usize = 10;

const WAIT_TIME_MS :u64 = 0;

const NUM_WALLS :i32 = 100;
const WALL_NEARBY_PROB :f64 = 0.48;

const NUM_PEOPLE :usize = 300;

const COLOR_EMPTY : [f32;3] = [0.5,0.3,0.4];
const COLOR_WALL : [f32;4] = [0.2,0.1,0.2,1.];

const PATH_RETRIES : i32 = 10;

#[derive(Copy,Clone)]
pub struct Person {
    destination : (usize, usize),
    location : (usize, usize),
    color : [f32;4],
    failed_walks:i32,
}

#[derive(Copy,Clone)]
pub enum Tile {
    Wall,
    Occupied,
    Destination([f32;4]),
    Empty
}

pub struct Board{
     tiles : [[Tile; ARENA_W] ; ARENA_W]
}

impl Board {
    pub fn new() -> Board {
        Board {
            tiles: [[Tile::Empty; ARENA_W]; ARENA_W]
        }
    }
    fn gen_walls(&mut self){
        for _i in 0..NUM_WALLS {
            let (x,y) = self.get_rand_empty();
            self.tiles[x][y] = Tile::Wall;
        }
        for x in 1..ARENA_W-1 {
            for y in 1..ARENA_W-1{
                let mut count =0.0;
                count += match self.tiles[x-1][y]{
                    Tile::Wall => 1. ,
                    _ => 0.
                };
                count += match self.tiles[x+1][y]{
                    Tile::Wall => 1. ,
                    _ => 0.
                };
                count += match self.tiles[x][y-1]{
                    Tile::Wall => 1. ,
                    _ => 0.
                };
                count += match self.tiles[x][y+1]{
                    Tile::Wall => 1. ,
                    _ => 0.
                };
                
                let prob = count * WALL_NEARBY_PROB *100.;
                let num :f64 = rand::thread_rng().gen_range(0,100) as f64;
                if prob>num {
                    self.tiles[x][y] = Tile::Wall;
                }

            }
        }
    }
    fn draw(&self, canvas : &mut SimpleCanvas) {
        for y in 0..ARENA_W{
            for x in 0..ARENA_W{
                let color = match self.tiles[x][y] {
                    Tile::Empty => { continue;
                    },
                    Tile::Wall => {
                        if y>=ARENA_W {continue;}
                        match self.tiles[x][y+1] {
                            Tile::Wall => COLOR_WALL,
                            _ =>  [0.,0.,0.,0.45]
                        }
                    },
                    Tile::Occupied => continue,
                    Tile::Destination(c) => c
                };
                canvas.squares()
                    .add([(x*ARENA_SCALE+ARENA_SCALE/2) as f32,(y*ARENA_SCALE + ARENA_SCALE/2) as f32])
                    .send_and_uniforms(canvas,ARENA_SCALE as f32)
                    .with_color(color)
                    .draw();
            }
        }
    }
    fn get_rand_empty(&self) -> (usize, usize) {
        loop{
            let x = rand::thread_rng().gen_range(0,ARENA_W-1);
            let y = rand::thread_rng().gen_range(0,ARENA_W-1);
            match self.tiles[x][y] {
                Tile::Empty => return (x,y),
                _ => ()
            };
        }
    }
}
impl Person {
    pub fn new_rand(board :&mut Board) -> Person{
        let location = board.get_rand_empty();
        let destination = board.get_rand_empty();
        Person::new(board, location, destination)
    }
    pub fn new(board :&mut Board, location : (usize, usize), destination:(usize, usize)) -> Person {
        board.tiles[location.0][location.1]=Tile::Occupied;
        let color = [rand::thread_rng().gen_range(0.,1.), 
                     rand::thread_rng().gen_range(0.,1.),
                     rand::thread_rng().gen_range(0.,1.), 0.9];
        board.tiles[destination.0][destination.1]=Tile::Destination(color);
        Person{
            location,
            destination,
            color,  
            failed_walks:0
        }
    }
    fn draw(&self, canvas : &mut SimpleCanvas){
            canvas.circles()
                .add([(self.location.0*ARENA_SCALE + ARENA_SCALE/2) as f32,(self.location.1*ARENA_SCALE + ARENA_SCALE/2) as f32])
                .send_and_uniforms(canvas,ARENA_SCALE as f32)
                .with_color(self.color)
                .draw();
    }
    fn new_dest(&mut self, board : &mut Board) {
        board.tiles[self.destination.0][self.destination.1]=Tile::Empty;
        self.destination = board.get_rand_empty();
        board.tiles[self.destination.0][self.destination.1]=Tile::Destination(self.color);
    }
    fn walk(&mut self, board: &mut Board){
        let astar = a_star(self.location, self.destination, board);
        match astar {
            Some(new_loc) => {
                self.failed_walks=0;
                board.tiles[self.location.0][self.location.1] = Tile::Empty;
                self.location = new_loc;
                board.tiles[self.location.0][self.location.1] = Tile::Occupied;
            },
            None =>{
                if self.failed_walks > PATH_RETRIES {
                 self.new_dest(board);
                }
                self.failed_walks += 1;
            }
        }
    }
}

fn dist(a:(usize, usize), b:(usize, usize))->usize {
    ((a.0 as isize - b.0 as isize).abs() + (a.1 as isize - b.1 as isize).abs()) as usize
}

fn a_star(start:(usize,usize), goal:(usize, usize), board : &Board) -> Option<(usize,usize)> {
    let mut open_set = HashSet::new();
    open_set.insert(start);
    const INFINITY :usize = 10000000;
    let mut came_from = [[(INFINITY,INFINITY);ARENA_W];ARENA_W];

    let mut g_score = [[INFINITY;ARENA_W];ARENA_W];
    g_score[start.0][start.1] = 0;
    
    let mut f_score = [[INFINITY;ARENA_W];ARENA_W];
    f_score[start.0][start.1] = dist(start, goal);
    while !open_set.is_empty() {
        let mut current = (INFINITY,INFINITY);
        let mut c_score = INFINITY;
        for loc in open_set.iter() {
            if f_score[loc.0][loc.1] < c_score {
                current = *loc;
                c_score = f_score[loc.0][loc.1];
            }
        }
        // current is now the location with the lowest fScore
        if (current.0 == goal.0) && (current.1 == goal.1){
            // RETURN
            let mut path = Vec::new();
            while (current.0<ARENA_W) && (current.1<ARENA_W){
                path.push(current);
                current = came_from[current.0][current.1]
            }
            path.pop(); // pop itself off
            return path.pop();
        }
        open_set.remove(&current);  
        // foreach neighbor of current
        let delta_list = [(2,1),(0,1),(1,2),(1,0)];
        for delta in delta_list.iter() {
            let loc = ((current.0 as isize + delta.0 as isize - 1) as usize ,
                        ( current.1 as isize + delta.1 as isize - 1) as usize);
            if loc.0 >= ARENA_W || loc.1 >= ARENA_W {
                continue;
            }
            match board.tiles[loc.0][loc.1] {
                Tile::Empty=>(),
                Tile::Destination(_color)=>{
                    // only add its own destination
                    if !((loc.0==goal.0)&&(loc.1==goal.1)){
                        continue;
                    }
                },
                _ => continue
            }; // We are only good with empty ones
            let tentative_g_score = g_score[current.0][current.1]+1;
            if tentative_g_score < g_score[loc.0][loc.1] {
                came_from[loc.0][loc.1] = current;
                g_score[loc.0][loc.1] = tentative_g_score;
                f_score[loc.0][loc.1] = tentative_g_score + dist(loc,goal);
                if !open_set.contains(&loc){
                    open_set.insert(loc);
                }
                
            }
        }
        
    }
    // No path found
    return None;
}


fn main() {
    let events_loop = glutin::event_loop::EventLoop::new();
    let mut glsys = egaku2d::WindowedSystem::new([ARENA_W*ARENA_SCALE,ARENA_W*ARENA_SCALE],&events_loop,"Dungeon");
    let mut board = Board::new();
    board.gen_walls();
    let mut people : [Person; NUM_PEOPLE] = unsafe { 
        let mut arry : [Person; NUM_PEOPLE] = std::mem::uninitialized();
        for i in &mut arry {
            *i = Person::new_rand(&mut board);
        }
        arry
    };
    loop{
       let mut canvas = glsys.canvas_mut();
        canvas.clear_color(COLOR_EMPTY);
        board.draw(&mut canvas);
        for i in &mut people{
            i.draw(&mut canvas);
            i.walk(&mut board);
        }
        glsys.swap_buffers();
        thread::sleep(time::Duration::from_millis(WAIT_TIME_MS));
    }
}

