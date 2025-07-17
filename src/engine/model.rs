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
    next_piece: PhysicalTetromino,
    stored_piece: PhysicalTetromino,
    iterator: TetrominoIterator<T>,
    switchted_active_piece_since_last_drop: bool,
}

impl<T: Rng + Sized + Send> Tetris<T> {
    pub fn new(rng: T) -> Self {
        let mut iterator = TetrominoIterator::new(rng);
        let mut field = [CellStatus::Empty; TETRIS_FIELD_LENGTH].into();
        let active_piece = Tetris::<T>::place_tetromino_on_field(&mut field, (&mut iterator).next().unwrap());
        let next_piece = Tetris::<T>::tetromino_to_physical((&mut iterator).next().unwrap());
        let stored_piece = Tetris::<T>::tetromino_to_physical((&mut iterator).next().unwrap());

        Self {
            field,
            active_piece,
            next_piece,
            stored_piece,
            iterator,
            switchted_active_piece_since_last_drop: false,
        }
    }

    pub fn try_switch_active_piece(&mut self) -> Result<(), ()> {
        if self.switchted_active_piece_since_last_drop {
            return Err(());
        }

        //clear board of active piece
        for pos in self.active_piece.coords {
            if let Some(cell) = self.field.get_mut(pos.x, pos.y) {
                *cell = CellStatus::Empty;
            }
        }

        let old_active = self.active_piece.tetromino;
        self.active_piece = Tetris::<T>::place_tetromino_on_field(&mut self.field, self.stored_piece.tetromino);
        self.stored_piece = Tetris::<T>::tetromino_to_physical(old_active);
        self.switchted_active_piece_since_last_drop = true;
        Ok(())
    }

    pub fn get_block_list(&self) -> Vec<(CellStatus, u32, u32)> {
        let mut vec = Vec::new();

        for y in 0..TETRIS_FIELD_DEFAULT_HEIGHT {
            for x in 0..TETRIS_FIELD_DEFAULT_WIDTH {
                let elem = self.field.get(x as i32, y as i32 ).unwrap();
                if elem != CellStatus::Empty {
                    vec.push((elem, x, y));
                }
            }
        }

        vec
    }

    pub fn get_next_block_list(&self) -> [(CellStatus, u32, u32); 4] {
        let mut arr: [(CellStatus, u32, u32); 4] = [(CellStatus::Empty, 0, 0); 4];
        
        for (index, pos) in self.next_piece.coords.iter().enumerate() {
            arr[index] = (self.next_piece.color, pos.x as u32, pos.y as u32);
        }

        arr
    }

    pub fn get_stored_block_list(&self) -> [(CellStatus, u32, u32); 4] {
        let mut arr: [(CellStatus, u32, u32); 4] = [(CellStatus::Empty, 0, 0); 4];
        
        for (index, pos) in self.stored_piece.coords.iter().enumerate() {
            arr[index] = (self.stored_piece.color, pos.x as u32, pos.y as u32);
        }

        arr
    }

    fn place_tetromino_on_field(field: &mut TetrisField, tetromino: Tetromino) -> PhysicalTetromino {
        let half_width = TETRIS_FIELD_DEFAULT_WIDTH / 2 - 1;

        let mut phys_tetromino = Tetris::<T>::tetromino_to_physical(tetromino);
        match phys_tetromino.tetromino {
            Tetromino::O => {
                phys_tetromino = phys_tetromino + Pos2::new(half_width as i32, TETRIS_FIELD_DEFAULT_HEIGHT as i32);
            }
            Tetromino::Line | Tetromino::T | Tetromino::L | Tetromino::J | Tetromino::S | Tetromino::Z => {
                phys_tetromino = phys_tetromino + Pos2::new((half_width - 1) as i32, TETRIS_FIELD_DEFAULT_HEIGHT as i32);
            }
        }

        // let mut phys_tetromino = match tetromino {
        //     Tetromino::O => {
        //         PhysicalTetromino::new(tetromino, CellStatus::Yellow) + Pos2::new(half_width as i32, TETRIS_FIELD_DEFAULT_HEIGHT as i32)
        //     }
        //     Tetromino::Line => {
        //         PhysicalTetromino::new(tetromino, CellStatus::Cyan) + Pos2::new((half_width - 1) as i32, TETRIS_FIELD_DEFAULT_HEIGHT as i32)
        //     }
        //     Tetromino::T => {
        //         PhysicalTetromino::new(tetromino, CellStatus::Purple) + Pos2::new((half_width - 1) as i32, TETRIS_FIELD_DEFAULT_HEIGHT as i32)
        //     }
        //     Tetromino::L => {
        //         PhysicalTetromino::new(tetromino, CellStatus::Orange) + Pos2::new((half_width - 1) as i32, TETRIS_FIELD_DEFAULT_HEIGHT as i32)
        //     }
        //     Tetromino::J => {
        //         PhysicalTetromino::new(tetromino, CellStatus::Blue) + Pos2::new((half_width - 1) as i32, TETRIS_FIELD_DEFAULT_HEIGHT as i32)
        //     }
        //     Tetromino::S => {
        //         PhysicalTetromino::new(tetromino, CellStatus::Green) + Pos2::new((half_width - 1) as i32, TETRIS_FIELD_DEFAULT_HEIGHT as i32)
        //     }
        //     Tetromino::Z => {
        //         PhysicalTetromino::new(tetromino, CellStatus::Red) + Pos2::new((half_width - 1) as i32, TETRIS_FIELD_DEFAULT_HEIGHT as i32)
        //     }
        // };

        if Tetris::<T>::try_drop(field, &mut phys_tetromino).is_ok()
                && tetromino != Tetromino::Line {
            let _ = Tetris::<T>::try_drop(field, &mut phys_tetromino);
        }

        phys_tetromino
    }

    fn tetromino_to_physical(tetromino: Tetromino) -> PhysicalTetromino {
        match tetromino {
            Tetromino::O => {
                PhysicalTetromino::new(tetromino, CellStatus::Yellow)
            }
            Tetromino::Line => {
                PhysicalTetromino::new(tetromino, CellStatus::Cyan)
            }
            Tetromino::T => {
                PhysicalTetromino::new(tetromino, CellStatus::Purple)
            }
            Tetromino::L => {
                PhysicalTetromino::new(tetromino, CellStatus::Orange)
            }
            Tetromino::J => {
                PhysicalTetromino::new(tetromino, CellStatus::Blue)
            }
            Tetromino::S => {
                PhysicalTetromino::new(tetromino, CellStatus::Green)
            }
            Tetromino::Z => {
                PhysicalTetromino::new(tetromino, CellStatus::Red)
            }
        }
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
                true
            }
            Err(_) => {
                self.switchted_active_piece_since_last_drop = false;
                self.check_for_lines_and_clear();
                self.next_piece();
                false
            }
        }
    }

    fn next_piece(&mut self) {
        self.active_piece = Tetris::<T>::place_tetromino_on_field(&mut self.field, self.next_piece.tetromino);
        self.next_piece = Tetris::<T>::tetromino_to_physical((&mut self.iterator).next().unwrap());
    }

    pub fn try_left(&mut self) -> Result<(), ()> {
        Tetris::<T>::try_move(&mut self.field, &mut self.active_piece, Direction::Left)
    }

    pub fn try_right(&mut self) -> Result<(), ()> {
        Tetris::<T>::try_move(&mut self.field, &mut self.active_piece, Direction::Right)
    }

    fn check_move(field: &TetrisField, tetromino: &PhysicalTetromino, direction: Direction) -> Result<(), ()> {
        let mut field_copy = field.clone();
        let mut tetromino_copy = tetromino.clone();

        //remove previous cells
        for pos in tetromino_copy.coords {
            if let Some(cell) = field_copy.get_mut(pos.x, pos.y) {
                *cell = CellStatus::Empty;
            }
        }

        //move active tetromino
        match direction {
            Direction::Down => {
                tetromino_copy = (tetromino_copy - Pos2::new(0, 1))?;
            }
            Direction::Left => {
                tetromino_copy = (tetromino_copy - Pos2::new(1, 0))?;
            }
            Direction::Right => {
                tetromino_copy = tetromino_copy + Pos2::new(1, 0);
                for pos in tetromino_copy.coords {
                    if pos.x >= TETRIS_FIELD_DEFAULT_WIDTH as i32 {
                        return Err(());
                    }
                }
            }
            Direction::Up => {
                tetromino_copy = tetromino_copy + Pos2::new(0, 1);
            }
        }

        //assert each new position is empty and legal
        for pos in tetromino_copy.coords {
            if let Some(cell) = field_copy.get(pos.x, pos.y) {
                if cell != CellStatus::Empty {
                    return Err(())
                }
            }
        }

        Ok(())
    }

    fn try_move(field: &mut TetrisField, tetromino: &mut PhysicalTetromino, direction: Direction) -> Result<(), ()> {
        //check if move is doable
        Tetris::<T>::check_move(field, tetromino, direction)?;

        //clear previous positions
        for pos in tetromino.coords {
            if let Some(cell) = field.get_mut(pos.x, pos.y) {
                *cell = CellStatus::Empty;
            }
        }

        //move tetromino coordinates
        match direction {
            Direction::Down => {
                *tetromino = (*tetromino - Pos2::new(0, 1))?;
            }
            Direction::Left => {
                *tetromino = (*tetromino - Pos2::new(1, 0))?;
            }
            Direction::Right => {
                *tetromino = *tetromino + Pos2::new(1, 0);
                for pos in tetromino.coords {
                    if pos.x >= TETRIS_FIELD_DEFAULT_WIDTH as i32  {
                        return Err(());
                    }
                }
            }
            Direction::Up => {
                *tetromino = *tetromino + Pos2::new(0, 1);
            }
        }

        for pos in tetromino.coords {
            if let Some(cell) = field.get_mut(pos.x, pos.y) {
                *cell = tetromino.color;
            }
        }

        Ok(())
    }
    
    pub fn drop_completely_down(&mut self) {
        loop {
            if !self.drop() {
                break;
            }
        }
    }

    fn check_for_lines_and_clear(&mut self) {
        while let Some(line_index) = self.check_line_clearing() {
            self.clear_line_and_drop_all_above(line_index);
        }
    }

    fn check_line_clearing(&self) -> Option<u32> {
        'outer: for y in 0..TETRIS_FIELD_DEFAULT_HEIGHT {
            for x in 0..TETRIS_FIELD_DEFAULT_WIDTH {
                if self.field.get(x as i32, y as i32).unwrap() == CellStatus::Empty {
                    continue 'outer;
                }
            }

            return Some(y);
        }

        None
    }

    fn clear_line_and_drop_all_above(&mut self, line: u32) {
        let field = &mut self.field.field;
        for line in (line as usize)..(TETRIS_FIELD_DEFAULT_HEIGHT as usize - 1) {
            let start_index = line * TETRIS_FIELD_DEFAULT_WIDTH as usize;
            let mid_index = start_index + TETRIS_FIELD_DEFAULT_WIDTH as usize;
            let end_index = mid_index + TETRIS_FIELD_DEFAULT_WIDTH as usize;

            field.copy_within(mid_index..end_index, start_index);
        }

        let last_line_index = (TETRIS_FIELD_DEFAULT_WIDTH * (TETRIS_FIELD_DEFAULT_HEIGHT - 1)) as usize;
        field[last_line_index..].copy_from_slice(&[CellStatus::Empty; TETRIS_FIELD_DEFAULT_WIDTH as usize]);
    }

    pub fn spin_clock_90(&mut self) {
        let _ = self.try_spin(SpinDirection::Clockwise);
    }

    pub fn spin_counter_90(&mut self) {
        let _ = self.try_spin(SpinDirection::CounterClockwise);
    }

    fn try_spin(&mut self, spin_direction: SpinDirection) -> Result<(), ()> {
        if self.check_spin(spin_direction).is_err() {
            let tetromino_original = self.active_piece;
            let field_original = self.field;
            let tetromino_copy = tetromino_original.clone();
            let field_copy = field_original.clone();
            self.active_piece = tetromino_copy;
            self.field = field_copy;

            Tetris::<T>::try_move(&mut self.field, &mut self.active_piece, Direction::Up)?;

            if self.check_spin(spin_direction).is_err() {
                self.active_piece = tetromino_original;
                self.field = field_original;
                return Err(())
            }
        }

        //remove previous blocks
        for pos in self.active_piece.coords {
            if let Some(cell) = self.field.get_mut(pos.x, pos.y) {
                *cell = CellStatus::Empty;
            }
        }

        //spin
        self.active_piece.spin(spin_direction);

        //add new blocks 
        for pos in self.active_piece.coords {
            if let Some(cell) = self.field.get_mut(pos.x, pos.y) {
                *cell = self.active_piece.color;
            }
        }

        Ok(())
    }

    fn check_spin(&self, spin_direction: SpinDirection) -> Result<(), ()> {
        let mut field_copy = self.field.clone();
        let mut tetromino_copy = self.active_piece.clone();

        //clear previous position
        for pos in tetromino_copy.coords {
            if let Some(cell) = field_copy.get_mut(pos.x, pos.y) {
                *cell = CellStatus::Empty;
            }
        }

        //spin the active piece
        tetromino_copy.spin(spin_direction);

        //check the new positions
        for pos in tetromino_copy.coords {
            if let Some(cell) = field_copy.get(pos.x, pos.y) {
                if pos.x >= TETRIS_FIELD_DEFAULT_WIDTH as i32 || !(cell == CellStatus::Empty || cell == tetromino_copy.color) {
                    return Err(());
                }
            } else {
                return Err(());
            }
        }

        Ok(())
    }
}

impl Default for Tetris<rand::rngs::OsRng> {
    fn default() -> Self {
        let rng = rand::rngs::OsRng;
        Tetris::new(rng)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TetrisField {
    field: [CellStatus; TETRIS_FIELD_LENGTH],
}

impl TetrisField {
    fn get(&self, x: i32, y: i32) -> Option<CellStatus> {
        if !(0..TETRIS_FIELD_DEFAULT_WIDTH as i32).contains(&x) || !(0..TETRIS_FIELD_DEFAULT_HEIGHT as i32).contains(&y) {
            None
        } else {
            Some(self.field[(y * TETRIS_FIELD_DEFAULT_WIDTH as i32 + x) as usize])
        }
    }

    fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut CellStatus> {
        if !(0..TETRIS_FIELD_DEFAULT_WIDTH as i32).contains(&x) || !(0..TETRIS_FIELD_DEFAULT_HEIGHT as i32).contains(&y) {
            None
        } else {
            Some(&mut self.field[(y * TETRIS_FIELD_DEFAULT_WIDTH as i32 + x) as usize])
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
    pub x: i32, 
    pub y: i32
}

impl Pos2 {
    fn new(x: i32, y: i32) -> Self {
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

impl From<Pos2f> for Pos2 {
    fn from(value: Pos2f) -> Self {
        Pos2 {
            x: value.x as i32,
            y: value.y as i32,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Pos2f {
    x: f32,
    y: f32,
}

impl Pos2f {
    const ZERO: Pos2f = Pos2f{x: 0.0, y: 0.0};

    fn new(x: f32, y: f32) -> Self {
        Self {
            x, 
            y,
        }
    }

    fn rotate_clock_90(&mut self) {
        let x = self.x;
        let y = self.y;
        
        self.x = y;
        self.y = -x;
    }

    fn rotate_counter_90(&mut self) {
        let x = self.x;
        let y = self.y;

        self.x = -y;
        self.y = x;
    }
}

impl From<Pos2> for Pos2f {
    fn from(value: Pos2) -> Self {
        Self {
            x: value.x as f32,
            y: value.y as f32,
        }
    }
}

impl std::ops::Add<Pos2f> for Pos2f {
    type Output = Pos2f;

    fn add(self, rhs: Pos2f) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Pos2f> for Pos2f {
    type Output = Pos2f;

    fn sub(self, rhs: Pos2f) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PhysicalTetromino {
    coords: [Pos2; 4],
    rotation_center: Pos2f,
    tetromino: Tetromino,
    color: CellStatus,
}

impl PhysicalTetromino {
    /// Generates a new Physical Tetromino from a given [Tetromino]. The initial position of a physical 
    /// Tetromino will be (0, 0) for the left-most, lowest cube - iff the shape is konvex at the bottom left. 
    fn new(tetromino: Tetromino, color: CellStatus) -> Self {
        let (coords, rotation_center) = match tetromino {
            Tetromino::Line => {
                (
                    [
                        Pos2::new(0, 0),
                        Pos2::new(1, 0),
                        Pos2::new(2, 0),
                        Pos2::new(3, 0),
                    ],
                    Pos2f::new(1.5, 0.0),
                )
            }
            Tetromino::O => {
                (   
                    [
                        Pos2::new(0, 0),
                        Pos2::new(0, 1),
                        Pos2::new(1, 0),
                        Pos2::new(1, 1),
                    ],
                    Pos2f::new(0.5, 0.5),
                )
            }
            Tetromino::J => {
                ( 
                    [
                        Pos2::new(0, 0),
                        Pos2::new(0, 1),
                        Pos2::new(1, 0),
                        Pos2::new(2, 0),
                    ],
                    Pos2f::new(1.0, 0.5),
                )
            }
            Tetromino::L => {
                (
                    [
                        Pos2::new(0, 0),
                        Pos2::new(1, 0),
                        Pos2::new(2, 0),
                        Pos2::new(2, 1),
                    ],
                    Pos2f::new(1.0, 0.5),
                )
            }
            Tetromino::S => {
                (   
                    [
                        Pos2::new(0, 0),
                        Pos2::new(1, 0),
                        Pos2::new(1, 1),
                        Pos2::new(2, 1),
                    ], 
                    Pos2f::new(1.0, 0.5),
                )
            }
            Tetromino::Z => {
                (
                    [
                        Pos2::new(0, 1),
                        Pos2::new(1, 1),
                        Pos2::new(1, 0),
                        Pos2::new(2, 0),
                    ],
                    Pos2f::new(1.0, 0.5),
                )
            }
            Tetromino::T => {
                (
                    [
                        Pos2::new(0, 0),
                        Pos2::new(1, 0),
                        Pos2::new(2, 0),
                        Pos2::new(1, 1),
                    ],
                    Pos2f::new(1.0, 0.0),
                )
            }
        };

        Self {
            coords, 
            rotation_center,
            tetromino,
            color,
        }
    }

    fn spin(&mut self, spin_direction: SpinDirection) {
        let mut float_positions = [Pos2f::ZERO; 4];
        for (index, pos) in self.coords.iter().enumerate() {
            float_positions[index] = Pos2f::from(*pos) - self.rotation_center;
        }

        for pos in &mut self.coords {
            let mut float_pos = Pos2f::from(*pos);

            //untranslate
            float_pos = float_pos - self.rotation_center;

            //rotate
            match spin_direction {
                SpinDirection::Clockwise => {
                    float_pos.rotate_clock_90();
                }
                SpinDirection::CounterClockwise => {
                    float_pos.rotate_counter_90();
                }
            }

            //retranslate
            float_pos = float_pos + self.rotation_center;

            *pos = Pos2::from(float_pos);
        }
    }
}

impl std::ops::Add<Pos2> for PhysicalTetromino {
    type Output = PhysicalTetromino;

    fn add(mut self, rhs: Pos2) -> Self::Output {
        for pos in &mut self.coords {
            *pos = *pos + rhs;
        }

        self.rotation_center = self.rotation_center + rhs.into();

        self
    }
}

impl std::ops::Sub<Pos2> for PhysicalTetromino {
    type Output = Result<PhysicalTetromino, ()>;

    fn sub(mut self, rhs: Pos2) -> Self::Output {
        for pos in &mut self.coords {
            *pos = (*pos - rhs)?;
        }

        self.rotation_center = self.rotation_center - rhs.into();

        Ok(self)
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Left, 
    Right, 
    Down,
    Up,
}

#[derive(Clone, Copy, Debug)]
enum SpinDirection {
    Clockwise, 
    CounterClockwise,
}
