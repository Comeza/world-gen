#![feature(stmt_expr_attributes)]
use rand::seq::SliceRandom;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum Tile {
    River,
    Wasteland,
    Farmland,
}

#[derive(Default)]
struct Plot {
    tiles: [[Tile; 16]; 16],
}

#[derive(Debug, Clone)]
enum WaveState {
    Collapsed(Tile),
    Superposition(Vec<Tile>),
}

#[derive(Default)]
struct PlotGenerator {
    tiles: [[WaveState; 16]; 16],
}

impl Tile {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tile::River => "░░",
            Tile::Wasteland => "▓▓",
            Tile::Farmland => "██",
        }
    }

    pub fn valid_neighbours(&self) -> Vec<Tile> {
        use Tile::*;
        match self {
            River => [River, Wasteland].into(),
            Wasteland => [River, Wasteland, Farmland].into(),
            Farmland => [Farmland, Wasteland].into(),
        }
    }
}

impl PlotGenerator {
    /// Iterates over every field.
    ///     * If found_entropy < entropy: reset the found fields (add current), and entropy = found_entropy
    ///     * If found_entropy > entropy: ignore
    ///     * if found_entropy = entropy: add to field array
    ///     * if found_entropy = 1      : ignore, is already collapsed
    pub fn find_lowest_entropy(&self) -> Vec<(usize, usize)> {
        let mut lowest = vec![];
        let mut entropy = usize::MAX;

        for y in 0..16 {
            for x in 0..16 {
                match &self.tiles[x][y] {
                    // Collapsed fields have entropy 0
                    WaveState::Collapsed(_) => continue,

                    // replace new lowest
                    WaveState::Superposition(pos) if pos.len() < entropy => {
                        entropy = pos.len();
                        lowest = vec![(x, y)];
                    }

                    // Has the same entropy as the current lowest
                    WaveState::Superposition(pos) if pos.len() == entropy => lowest.push((x, y)),

                    // pos.entropy > entropy
                    WaveState::Superposition(_) => continue,
                }
            }
        }

        lowest
    }

    // After the current field got updated, update other fields accordingly (remove impossible
    // states)
    pub fn update_neighbours(&mut self, (x, y): (usize, usize)) {
        // Method A, updates neighbours in a "+" shape
        let _neighbours: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

        // Method B, updates neighbours in a "#" shape
        let mut neighbours: [(isize, isize); 8] = [(0, 0); 8];

        let mut i = 0;
        for x in -1..=1 {
            for y in -1..=1 {
                if x == 0 && y == 0 {
                    continue;
                }
                neighbours[i] = (x, y);
                i += 1;
            }
        }
        // \Method B

        let possibilities = match &self.tiles[x][y] {
            WaveState::Superposition(s) => s
                .iter()
                .map(Tile::valid_neighbours)
                .flatten()
                .collect::<Vec<_>>(),
            WaveState::Collapsed(c) => c.valid_neighbours(),
        };

        for (dx, dy) in neighbours {
            // skip overflows
            if x as isize + dx < 0 || x as isize + dx >= 16 {
                continue;
            }
            if y as isize + dy < 0 || y as isize + dy >= 16 {
                continue;
            }

            // calculate offset
            let dx = (x as isize + dx) as usize;
            let dy = (y as isize + dy) as usize;

            // Remove impossible states
            if let WaveState::Superposition(poss) = &mut self.tiles[dx][dy] {
                *poss = poss
                    .iter()
                    .map(|x| x.clone())
                    .filter(|t| possibilities.contains(t))
                    .collect();
            }
        }
    }

    /// SEE: [This Video](https://www.youtube.com/watch?v=2SuvO4Gi7uY)
    ///
    /// * Find the fields with the lowest entropy,
    ///     -> if there is just one field, take it
    ///     -> If there is more than one field, take random
    ///
    /// * Collapse field into a field with just one possibility.
    /// * Update the neighbours, and remove possibilities that got "destoryed", in the previous
    /// step
    pub fn collapse(&mut self) {
        while let Some((x, y)) = self
            .find_lowest_entropy()
            .as_slice()
            .choose(&mut rand::thread_rng())
        {
            let (x, y) = (*x, *y);

            if let WaveState::Superposition(states) = &self.tiles[x][y] {
                self.tiles[x][y] = WaveState::Collapsed(
                    *states
                        .as_slice()
                        .choose(&mut rand::thread_rng())
                        .expect("No valid state possible"),
                );
            }

            self.update_neighbours((x, y));
        }
    }

    pub fn into_plot(self) -> Plot {
        let mut plot = Plot::default();
        for y in 0..16 {
            for x in 0..16 {
                plot.tiles[x][y] = match self.tiles[x][y] {
                    WaveState::Collapsed(x) => x,
                    WaveState::Superposition(_) => panic!("Found not collapsed tile"),
                }
            }
        }

        plot
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Wasteland
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for Plot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in 0..16 {
            for y in 0..16 {
                write!(f, "{}", self.tiles[x][y])?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl Default for WaveState {
    fn default() -> Self {
        use Tile::*;
        WaveState::Superposition(vec![River, Wasteland, Farmland])
    }
}

fn main() {
    let mut gen = PlotGenerator::default();
    gen.collapse();

    let plot = gen.into_plot();

    println!("{plot}");
}
