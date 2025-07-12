use rand::Rng;
use rand::seq::SliceRandom;

const NBR_OF_TETROMINUS: u32 = 7;
pub const TETRIS_FIELD_DEFAULT_WIDTH: u32 = 10;
pub const TETRIS_FIELD_DEFAULT_HEIGHT: u32 = 20;
const TETRIS_FIELD_LENGTH: usize = (TETRIS_FIELD_DEFAULT_WIDTH * TETRIS_FIELD_DEFAULT_HEIGHT) as usize;


#[derive(Clone, Debug)]
pub struct Tetris<T: Rng + Sized + Send> {
    pub field: TetrisField,
    active_piece: PhysicalTetromino,
    next_piece: Tetromino,
    stored_piece: Tetromino,
    iterator: TetrominoIterator<T>,
    switchted_active_piece_since_last_drop: bool,
}

impl<T: Rng + Sized + Send> Tetris<T> {
    pub fn new(rng: T) -> Self {
        let mut iterator = TetrominoIterator::new(rng);
        let mut field = [CellStatus::Empty; TETRIS_FIELD_LENGTH].into();
        let active_piece = Tetris::<T>::place_tetromino_on_field(&mut field, (&mut iterator).next().unwrap());

        Self {
            field,
            active_piece,
            next_piece: (&mut iterator).next().unwrap(),
            stored_piece: (&mut iterator).next().unwrap(),
            iterator,
            switchted_active_piece_since_last_drop: false,
        }
    }

    pub fn try_switch_active_piece(&mut self) -> Result<(), ()> {
        // if self.switchted_active_piece_since_last_drop {
        //     return Err(());
        // }

        // std::mem::swap(&mut self.active_piece, &mut self.stored_piece);
        // Ok(())

        todo!()
    }

    pub fn get_block_list(&self) -> Vec<(CellStatus, u32, u32)> {
        let mut vec = Vec::new();

        for y in 0..TETRIS_FIELD_DEFAULT_HEIGHT {
            for x in 0..TETRIS_FIELD_DEFAULT_WIDTH {
                let elem = self.field.get(x, y).unwrap();
                if elem != CellStatus::Empty {
                    vec.push((elem, x, y));
                }
            }
        }

        vec
    }

    fn place_tetromino_on_field(field: &mut TetrisField, tetromino: Tetromino) -> PhysicalTetromino {
        let half_width = TETRIS_FIELD_DEFAULT_WIDTH / 2 - 1;

        let mut phys_tetromino = match tetromino {
            Tetromino::O => {
                PhysicalTetromino::new(tetromino, CellStatus::Yellow) + Pos2::new(half_width, TETRIS_FIELD_DEFAULT_HEIGHT)
            }
            Tetromino::Line => {
                PhysicalTetromino::new(tetromino, CellStatus::Cyan) + Pos2::new(half_width - 1, TETRIS_FIELD_DEFAULT_HEIGHT)
            }
            Tetromino::T => {
                PhysicalTetromino::new(tetromino, CellStatus::Purple) + Pos2::new(half_width - 1, TETRIS_FIELD_DEFAULT_HEIGHT)
            }
            Tetromino::L => {
                PhysicalTetromino::new(tetromino, CellStatus::Orange) + Pos2::new(half_width - 1, TETRIS_FIELD_DEFAULT_HEIGHT)
            }
            Tetromino::J => {
                PhysicalTetromino::new(tetromino, CellStatus::Blue) + Pos2::new(half_width - 1, TETRIS_FIELD_DEFAULT_HEIGHT)
            }
            Tetromino::S => {
                PhysicalTetromino::new(tetromino, CellStatus::Green) + Pos2::new(half_width - 1, TETRIS_FIELD_DEFAULT_HEIGHT)
            }
            Tetromino::Z => {
                PhysicalTetromino::new(tetromino, CellStatus::Red) + Pos2::new(half_width - 1, TETRIS_FIELD_DEFAULT_HEIGHT)
            }
        };

        if Tetris::<T>::try_drop(field, &mut phys_tetromino).is_ok()
                && tetromino != Tetromino::Line {
            let _ = Tetris::<T>::try_drop(field, &mut phys_tetromino);
        }

        phys_tetromino
    }

    fn try_drop(field: &mut TetrisField, tetromino: &mut PhysicalTetromino) -> Result<(), ()> {
        //remove active tetromino from temporary copy
        let mut field_copy = field.clone();
        for pos in tetromino.coords {
            if let Some(cell) = field_copy.get_mut(pos.x, pos.y) {
                *cell = CellStatus::Empty;
            }
        }

        //check if temporary copy has place for active tetromino one below
        let tetromino_copy = (tetromino.clone() - Pos2::new(0, 1))?;
        let mut is_new_place_free = true;
        for pos in tetromino_copy.coords {
            if let Some(cell) = field_copy.get(pos.x, pos.y) {
                if cell != CellStatus::Empty {
                    is_new_place_free = false;
                    break;
                }
            }
        }

        //drop if possible, else return error
        if is_new_place_free {
            //clear previous positions
            for pos in tetromino.coords {
                if let Some(cell) = field.get_mut(pos.x, pos.y) {
                    *cell = CellStatus::Empty;
                }
            }
            //move tetromino
            *tetromino = (*tetromino - Pos2::new(0, 1)).unwrap();
            //fill new positions
            for pos in tetromino.coords {
                if let Some(cell) = field.get_mut(pos.x, pos.y) {
                    *cell = tetromino.color;
                }
            }
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn drop(&mut self) -> bool {
        let drop_result = Tetris::<T>::try_drop(&mut self.field, &mut self.active_piece);
        match drop_result {
            Ok(_) => {
                //successfull drop, nothing else to be done
                return true;
            }
            Err(_) => {
                self.next_piece();
                return false;
            }
        }
    }

    fn next_piece(&mut self) {
        self.active_piece = Tetris::<T>::place_tetromino_on_field(&mut self.field, self.next_piece);
        self.next_piece = (&mut self.iterator).next().unwrap();
    }

    pub fn try_left(&mut self) -> Result<(), ()> {
        todo!()
    }
}

impl Default for Tetris<rand::rngs::OsRng> {
    fn default() -> Self {
        let rng = rand::rngs::OsRng::default();
        Tetris::new(rng)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TetrisField {
    field: [CellStatus; TETRIS_FIELD_LENGTH],
}

impl TetrisField {
    fn get(&self, x: u32, y: u32) -> Option<CellStatus> {
        if x >= TETRIS_FIELD_DEFAULT_WIDTH || y >= TETRIS_FIELD_DEFAULT_HEIGHT {
            None
        } else {
            Some(self.field[(y * TETRIS_FIELD_DEFAULT_WIDTH + x) as usize])
        }
    }

    fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut CellStatus> {
        if x >= TETRIS_FIELD_DEFAULT_WIDTH || y >= TETRIS_FIELD_DEFAULT_HEIGHT {
            None
        } else {
            Some(&mut self.field[(y * TETRIS_FIELD_DEFAULT_WIDTH + x) as usize])
        }
    }
}

impl From<[CellStatus; TETRIS_FIELD_LENGTH]> for TetrisField {
    fn from(value: [CellStatus; TETRIS_FIELD_LENGTH]) -> Self {
        Self {
            field: value,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CellStatus {
    Empty,
    Cyan,
    Yellow, 
    Purple,
    Green, 
    Red, 
    Blue, 
    Orange,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Tetromino {
    Line,
    O,
    L, 
    J, 
    Z, 
    S, 
    T,
}

impl Tetromino {
    pub fn all_tetromino_array() -> [Self; NBR_OF_TETROMINUS as usize] {
        [
            Self::Line, 
            Self::O, 
            Self::L, 
            Self::J, 
            Self::Z, 
            Self::S, 
            Self::T,
        ]
    }
}

#[derive(Clone, Debug)]
struct TetrominoIterator<T: Rng + Sized> {
    pieces: Vec<Tetromino>,
    rng: T,
}

impl<T: Rng + Sized> TetrominoIterator<T> {
    pub fn new(mut rng: T) -> Self {
        let pieces = Vec::from(Self::get_new_seven(&mut rng));

        Self {
            pieces,
            rng,
        }
    }

    fn get_new_seven(rng: &mut T) -> [Tetromino; NBR_OF_TETROMINUS as usize] {
        let mut pieces = Tetromino::all_tetromino_array();
        pieces.shuffle(rng);
        pieces
    }
}

impl<T: Rng + Sized> Iterator for &mut TetrominoIterator<T> {
    type Item = Tetromino;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pieces.is_empty() {
            let new = TetrominoIterator::get_new_seven(&mut self.rng);
            self.pieces.extend_from_slice(&new);
        }

        let len = self.pieces.len();
        Some(self.pieces.remove(len - 1))
    }
}

#[derive(Clone, Copy, Debug)]
struct Pos2 {
    pub x: u32, 
    pub y: u32
}

impl Pos2 {
    fn new(x: u32, y: u32) -> Self {
        Self {
            x, 
            y, 
        }
    }
}

impl std::ops::Add<Pos2> for Pos2 {
    type Output = Pos2;

    fn add(self, rhs: Pos2) -> Self::Output {

        Pos2{
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Pos2> for Pos2 {
    type Output = Result<Pos2, ()>;

    fn sub(mut self, rhs: Pos2) -> Self::Output {
        if self.x >= rhs.x && self.y >= rhs.y {
            self.x -= rhs.x;
            self.y -= rhs.y;
            Ok(self)
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PhysicalTetromino {
    coords: [Pos2; 4],
    tetromino: Tetromino,
    color: CellStatus,
}

impl PhysicalTetromino {
    /// Generates a new Physical Tetromino from a given [Tetromino]. The initial position of a physical 
    /// Tetromino will be (0, 0) for the left-most, lowest cube - iff the shape is konvex at the bottom left. 
    fn new(tetromino: Tetromino, color: CellStatus) -> Self {
        let coords = match tetromino {
            Tetromino::Line => {
                [
                    Pos2::new(0, 0),
                    Pos2::new(1, 0),
                    Pos2::new(2, 0),
                    Pos2::new(3, 0),
                ]
            }
            Tetromino::O => {
                [
                    Pos2::new(0, 0),
                    Pos2::new(0, 1),
                    Pos2::new(1, 0),
                    Pos2::new(1, 1),
                ]
            }
            Tetromino::J => {
                [
                    Pos2::new(0, 0),
                    Pos2::new(0, 1),
                    Pos2::new(1, 0),
                    Pos2::new(2, 0),
                ]
            }
            Tetromino::L => {
                [
                    Pos2::new(0, 0),
                    Pos2::new(1, 0),
                    Pos2::new(2, 0),
                    Pos2::new(2, 1),
                ]
            }
            Tetromino::S => {
                [
                    Pos2::new(0, 0),
                    Pos2::new(1, 0),
                    Pos2::new(1, 1),
                    Pos2::new(2, 1),
                ]
            }
            Tetromino::Z => {
                [
                    Pos2::new(0, 1),
                    Pos2::new(1, 1),
                    Pos2::new(1, 0),
                    Pos2::new(2, 0),
                ]
            }
            Tetromino::T => {
                [
                    Pos2::new(0, 0),
                    Pos2::new(1, 0),
                    Pos2::new(2, 0),
                    Pos2::new(1, 1),
                ]
            }
        };

        Self {
            coords, 
            tetromino,
            color,
        }
    }
}

impl std::ops::Add<Pos2> for PhysicalTetromino {
    type Output = PhysicalTetromino;

    fn add(mut self, rhs: Pos2) -> Self::Output {
        for pos in &mut self.coords {
            *pos = *pos + rhs;
        }

        self
    }
}

impl std::ops::Sub<Pos2> for PhysicalTetromino {
    type Output = Result<PhysicalTetromino, ()>;

    fn sub(mut self, rhs: Pos2) -> Self::Output {
        for pos in &mut self.coords {
            *pos = (*pos - rhs)?;
        }
        Ok(self)
    }
}
