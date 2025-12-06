use colored::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::env;
use std::fs;
use std::time::Duration;

// STRUCTURES

#[derive(Debug, Clone, Eq, PartialEq)]
struct Node {
    pos: (usize, usize),
    cost: u32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// PARSING

fn parse_dimensions(arg: &str) -> (usize, usize) {
    let parts: Vec<&str> = arg.split('x').collect();
    if parts.len() != 2 {
        panic!("Format invalide. Exemple accepté : 12x8");
    }
    let w = parts[0].parse::<usize>().unwrap();
    let h = parts[1].parse::<usize>().unwrap();
    (w, h)
}

// GENERATION DE MAP

use rand::Rng;

fn generate_map(w: usize, h: usize) -> Vec<Vec<u32>> {
    let mut rng = rand::rng();

    let mut grid = vec![vec![0u32; w]; h];

    for row in grid.iter_mut() {
        for cell in row.iter_mut() {
            *cell = rng.random_range(0..=255);
        }
    }

    grid[0][0] = 0x00;
    grid[h - 1][w - 1] = 0xFF;

    grid
}

fn save_map(grid: &Vec<Vec<u32>>, path: &str) {
    let mut out = String::new();
    for row in grid {
        let line = row
            .iter()
            .map(|v| format!("{:02X}", v))
            .collect::<Vec<String>>()
            .join(" ");
        out.push_str(&line);
        out.push('\n');
    }
    fs::write(path, out).unwrap();
    println!("Map saved to: {}", path);
}

// CHARGEMENT

fn load_map(path: &str) -> Vec<Vec<u32>> {
    let content = fs::read_to_string(path).unwrap();
    let mut grid = vec![];

    for line in content.lines() {
        let row = line
            .split_whitespace()
            .map(|hex| u32::from_str_radix(hex, 16).unwrap())
            .collect::<Vec<u32>>();
        grid.push(row);
    }

    grid
}

// DJIKSTRA

fn neighbors(x: usize, y: usize, w: usize, h: usize) -> Vec<(usize, usize)> {
    let mut v = vec![];
    if x > 0 {
        v.push((x - 1, y));
    }
    if x + 1 < w {
        v.push((x + 1, y));
    }
    if y > 0 {
        v.push((x, y - 1));
    }
    if y + 1 < h {
        v.push((x, y + 1));
    }
    v
}

// Dijkstra générique
fn dijkstra(grid: &[Vec<u32>], max_mode: bool) -> (u32, Vec<(usize, usize)>) {
    let h = grid.len();
    let w = grid[0].len();

    let start = (0, 0);
    let goal = (w - 1, h - 1);

    let mut dist = HashMap::new();
    let mut prev = HashMap::new();

    let mut heap = BinaryHeap::new();

    dist.insert(start, 0u32);
    heap.push(Node {
        pos: start,
        cost: 0,
    });

    while let Some(Node { pos, cost }) = heap.pop() {
        if pos == goal {
            break;
        }

        for (nx, ny) in neighbors(pos.0, pos.1, w, h) {
            let edge_cost = grid[ny][nx];

            let next_cost = if max_mode {
                cost + (255 - edge_cost)
            } else {
                cost + edge_cost
            };

            if dist.get(&(nx, ny)).is_none_or(|&old| next_cost < old) {
                dist.insert((nx, ny), next_cost);
                prev.insert((nx, ny), pos);
                heap.push(Node {
                    pos: (nx, ny),
                    cost: next_cost,
                });
            }
        }
    }

    let mut path = vec![];
    let mut cur = goal;

    while let Some(&p) = prev.get(&cur) {
        path.push(cur);
        cur = p;
    }
    path.push(start);

    path.reverse();

    let mut true_cost = 0;
    for &(x, y) in &path {
        true_cost += grid[y][x];
    }

    (true_cost, path)
}

// VISUALISATION COULEURS

#[allow(dead_code)]
fn color_for_hex(v: u32) -> ColoredString {
    let r = (v as f32 / 255.0 * 255.0) as u8;
    let g = 255 - r;
    format!("{:02X}", v).truecolor(r, g, 128)
}

#[allow(dead_code)]
fn visualize(grid: &[Vec<u32>], min_path: &[(usize, usize)], max_path: &[(usize, usize)]) {
    let mut in_min = std::collections::HashSet::new();
    let mut in_max = std::collections::HashSet::new();

    for &p in min_path {
        in_min.insert(p);
    }
    for &p in max_path {
        in_max.insert(p);
    }

    for (y, row) in grid.iter().enumerate() {
        let mut line = String::new();
        for (x, &v) in row.iter().enumerate() {
            let mut s = format!("{:02X}", v);
            if in_min.contains(&(x, y)) {
                s = s.white().bold().to_string();
            } else if in_max.contains(&(x, y)) {
                s = s.red().bold().to_string();
            } else {
                let c = color_for_hex(v);
                s = c.to_string();
            }
            line.push_str(&format!("{} ", s));
        }
        println!("{}", line);
    }
}

// MODE ANIMATION
#[allow(unused_variables)]
#[allow(dead_code)]
fn animate(grid: &[Vec<u32>]) {
    println!("Animation mode not fully implemented – but here’s a simple demo:");
    for i in 0..5 {
        println!("Step {} exploring...", i);
        std::thread::sleep(Duration::from_millis(200));
    }
}

// MAIN

fn main() {
    let args: Vec<String> = env::args().collect();

    // CASE 1 : GENERATION
    if args.contains(&"--generate".to_string()) {
        let i = args.iter().position(|a| a == "--generate").unwrap();
        let dims = parse_dimensions(&args[i + 1]);
        let grid = generate_map(dims.0, dims.1);

        if args.contains(&"--output".to_string()) {
            let j = args.iter().position(|a| a == "--output").unwrap();
            let out = &args[j + 1];
            save_map(&grid, out);
        } else {
            println!("(pas d'output fourni)");
        }
        return;
    }

    // CASE 2 : ANALYSE MAP
    if args.len() >= 2 && args[1].ends_with(".txt") {
        let grid = load_map(&args[1]);

        println!("Analyzing hexadecimal grid...");
        println!("Grid size: {}x{}", grid[0].len(), grid.len());
        println!("Start: (0,0) = 0x00");
        println!("End:   ({},{}) = 0xFF", grid[0].len() - 1, grid.len() - 1);

        let (min_cost, min_path) = dijkstra(&grid, false);
        let (max_cost, max_path) = dijkstra(&grid, true);

        println!("\nMINIMUM COST PATH:");
        println!("Cost: {} decimal", min_cost);
        println!("Path: {:?}", min_path);

        println!("\nMAXIMUM COST PATH:");
        println!("Cost: {} decimal", max_cost);
        println!("Path: {:?}", max_path);

        return;
    }

    println!("Usage:\n  cargo run -- --generate 12x8 --output map.txt\n  cargo run -- map.txt\n");
}
