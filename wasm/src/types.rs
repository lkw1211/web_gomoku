use strum_macros::{EnumCount as EnumCountMacro, EnumIter};
use strum::IntoEnumIterator;
use std::ops::{Add, Sub, Mul, Div, Index, IndexMut};
use std::mem;

#[path="pattern.rs"]
mod pattern;
use pattern::*;

pub const MOVE_NONE: Move = 0;
pub const BOARD_SIZE: i32 = 15;
pub const DEPTH_MAX: Depth = 120;
pub const DEPTH_NUM: i32 = DEPTH_MAX as i32 + 1;
pub const SCORE_NONE: Score = 32002;
pub const INFINITY_SCORE: Score = 32001;
pub const WIN_SCORE: Score = 32000;
pub const B4_SCORE: Score = 364;
pub const F3_SCORE: Score = 364;
pub const B3_SCORE: Score = 99;
pub const F2_SCORE: Score = 99;
pub const B2_SCORE: Score = 29;
pub const F1_SCORE: Score = 31;
pub const B1_SCORE: Score = 8;
pub const B4_SEE_SCORE: Score = 2048;
pub const F3_SEE_SCORE: Score = 2048;
pub const B3_SEE_SCORE: Score = 256;
pub const F2_SEE_SCORE: Score = 256;
pub const B2_SEE_SCORE: Score = 16;
pub const F1_SEE_SCORE: Score = 16;
pub const B1_SEE_SCORE: Score = 1;
pub const BOARD_BOUNDARY: i32 = 4;
pub const BOARD_SIDE_BIT: i32 = 5;
pub const BOARD_SIDE: i32 = 15;
pub const BOARD_SIDE_CAPACITY: i32 = 1 << BOARD_SIDE_BIT;
pub const MOVE_SIZE: i32 = BOARD_SIDE.pow(2);
pub const MOVE_CAPACITY: i32 = BOARD_SIDE_CAPACITY.pow(2);
pub const STACK_SIZE: i32 = BOARD_SIDE.pow(2) + 1;
pub const WIN_SCORE_THRESHOLD: Score = WIN_SCORE - STACK_SIZE as i16;
pub const BITBOARD_SIZE: i32 = MOVE_CAPACITY / 64 + 1;
pub const VECTOR_SIZE: i32 = BOARD_SIDE * 6 - 2;
pub const BONUS_F3D: Score = 1024;
pub const BONUS_REFUTATION: Score = 1536;
pub const SEE_THRESHOLD: Score = 12;
pub const MATERIAL_NONE: u32 = 15;
pub const DEPTH_ITERATIVE_MAX: Depth = 100;
pub const DEPTH_NONE: Depth = 127;
pub const DEPTH_ITERATIVE_MIN: Depth = 1;

pub type ZobristKey = u64;
pub type Move = i16;
pub type Score = i16;
pub type Depth = i8;

pub static mut TT: TranspositionTable = TranspositionTable::new();
pub static mut BOARD: Option<Board> = None;
pub static mut PLY: Depth = 0;
pub static mut PLY_MAX: Depth = 0;
pub static mut SEARCH_STACK: SearchStack = [SearchStackElement::new(); STACK_SIZE as usize];
pub static mut ROOT_BESTS: Vec<RootExtMove> = Vec::new();
pub static mut COUNTER_MOVES: CounterMoveHistory = [MOVE_NONE; MOVE_CAPACITY as usize];

fn mul_hi64(a: u64, b: u64) -> u64 {
    (((a as u128) * (b as u128)) >> 64) as u64
}

fn get_bits(a: u32, begin: u32, end: u32) -> u32 {
    (a >> begin) & (((1 as u32) << (end - begin)) - 1)
}

fn set_bit(a: &mut u32, ind: i32) {
    *a |= (1 as u32) << ind;
}

fn reset_bit(a: &mut u32, ind: i32) {
    *a &= !((1 as u32) << ind);
}

fn is_ok_move(m: Move) -> bool {
    return 0 <= _rank_of(m) && _rank_of(m) < BOARD_SIDE && 0 <= _file_of(m) && _file_of(m) < BOARD_SIDE;
}

fn is_ok_score(s: Score) -> bool {
    return -INFINITY_SCORE < s && s < INFINITY_SCORE;
}

fn oppo_direction(d: Direction) -> Direction {
    match d {
        Direction::DFile => Direction::DRank,
        Direction::DRank => Direction::DFile,
        Direction::DMDiag => Direction::DADiag,
        Direction::DADiag => Direction::DMDiag,
    }
}

const SCORE_HELPER: [Score; 10] = [
    0,
    0,
    0,
    B4_SCORE,
    F3_SCORE,
    B3_SCORE,
    F2_SCORE,
    B2_SCORE,
    F1_SCORE,
    B1_SCORE,
];

const SEE_HELPER: [Score; 10] = [
    0,
    0,
    0,
    B4_SEE_SCORE,
    F3_SEE_SCORE,
    B3_SEE_SCORE,
    F2_SEE_SCORE,
    B2_SEE_SCORE,
    F1_SEE_SCORE,
    B1_SEE_SCORE,
];

#[derive(Default, Clone, Copy)]
pub struct TTEntry {
    key: u32,
    move16: Move,
    score: Score,
    bound: u8,
    depth: Depth,
}

fn index_of_helper(m: Move, d: Direction) -> i32 {
    match d {
        Direction::DRank => _rank_of(m),
        Direction::DFile => _file_of(m) + BOARD_SIDE,
        Direction::DMDiag => _mdiag_of(m) + BOARD_SIDE * 2,
        Direction::DADiag => _adiag_of(m) + BOARD_SIDE * 4 - 1,
    }
}

fn index_on_helper(m: Move, d: Direction) -> i32 {
    match d {
        Direction::DRank => _file_of(m),
        Direction::DFile => _rank_of(m),
        Direction::DMDiag => _mdiag_index_on(m),
        Direction::DADiag => _adiag_index_on(m),
    }
}

impl TTEntry {
    const fn new() -> TTEntry {
        Self {
            key: 0,
            move16: MOVE_NONE,
            score: SCORE_NONE,
            bound: 0,
            depth: 0,
        }
    }

    pub fn move16(&self) -> Move {
        self.move16
    }

    pub fn score(&self) -> Score {
        self.score
    }

    fn is_pv(&self) -> bool {
        (self.bound & 0x4) != 0
    }

    pub fn bound(&self) -> Bound {
        match self.bound & 0x3 {
            3 => Bound::BoundExact,
            2 => Bound::BoundLower,
            1 => Bound::BoundUpper,
            0 => Bound::BoundNone,
            _ => unreachable!()
        }
    }

    pub fn depth(&self) -> Depth {
        self.depth
    }

    fn save(&mut self, k: ZobristKey, m: Move, s: Score, b: Bound, pv: bool, d: Depth) {
        // Preserve any existing move for the same position
        if m != MOVE_NONE || k as u32 != self.key {
            self.move16 = m;
        }

        // Overwrite less valuable entries
        if b == Bound::BoundExact || k as u32 != self.key || d > self.depth - 4 {
            self.key     = k as u32;
            self.score   = s as Score;
            unsafe {
                self.bound   = TT.generation8 | (pv as u8) << 2| match b {
                    Bound::BoundExact => 3,
                    Bound::BoundLower => 2,
                    Bound::BoundUpper => 1,
                    Bound::BoundNone => 0
                };
            }
            self.depth   = d;
        }
    }
}

#[derive(Default, Clone, Copy)]
struct Cluster {
    entry: [TTEntry; TranspositionTable::CLUSTER_SIZE],
    padding: [char; 2]
}

impl Cluster {
    const fn new() -> Cluster {
        Self {
            entry: [TTEntry::new(); TranspositionTable::CLUSTER_SIZE],
            padding: [' '; 2],
        }
    }
}

pub struct TranspositionTable {
    cluster_count: i32,
    table: [Cluster; 256 * 1024 * 32],
    generation8: u8,
}

impl TranspositionTable {
    const CLUSTER_SIZE: usize = 3;
    const GENERATION_BITS: u32  = 3;
    const GENERATION_DELTA: u8 = (1 << 3);
    const GENERATION_CYCLE: i32 = 255 + (1 << 3);
    const GENERATION_MASK: i32  = (0xFF << 3) & 0xFF;
    const CLUSTER_SIZEOF: i32 = mem::size_of::<Cluster>() as i32;

    const fn new() -> TranspositionTable {
        Self {
            cluster_count: 256 * 1024 * 32,
            table: [Cluster::new(); 256 * 1024 * 32],
            generation8: 0
        }
    }

    fn new_search(&mut self) {
        self.generation8 += Self::GENERATION_DELTA;
    }

    fn probe(&mut self, key: &ZobristKey, found: &mut bool) -> &mut TTEntry {
        let key32: u32 = *key as u32;
        for i in 0..Self::CLUSTER_SIZE {
            if self.table[mul_hi64(*key, self.cluster_count as u64) as usize].entry[i].key == key32 || !self.table[mul_hi64(*key, self.cluster_count as u64) as usize].entry[i].depth != 0 {
                self.table[mul_hi64(*key, self.cluster_count as u64) as usize].entry[i].bound = self.generation8 | (self.table[mul_hi64(*key, self.cluster_count as u64) as usize].entry[i].bound & (Self::GENERATION_DELTA - 1)); // Refresh

                *found = self.table[mul_hi64(*key, self.cluster_count as u64) as usize].entry[i].depth != 0;

                return &mut self.table[mul_hi64(*key, self.cluster_count as u64) as usize].entry[i];
            }
        }
        // Find an entry to be replaced according to the replacement strategy
        let mut replace = 0;
        for i in 1..Self::CLUSTER_SIZE {
            if self.table[mul_hi64(*key, self.cluster_count as u64) as usize].entry[replace].depth as i32 - ((Self::GENERATION_CYCLE + self.generation8 as i32 - self.table[mul_hi64(*key, self.cluster_count as u64) as usize].entry[replace].bound as i32) & Self::GENERATION_MASK) > self.table[mul_hi64(*key, self.cluster_count as u64) as usize].entry[i].depth as i32 - ((Self::GENERATION_CYCLE + self.generation8 as i32 - self.table[mul_hi64(*key, self.cluster_count as u64) as usize].entry[i].bound as i32) & Self::GENERATION_MASK) {
                replace = i;
            }
        }
        *found = false;
        return &mut self.table[mul_hi64(*key, self.cluster_count as u64) as usize].entry[replace];
    }

    fn first_entry(self, key: ZobristKey) -> TTEntry {
        self.table[mul_hi64(key, self.cluster_count as u64) as usize].entry[0]
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum Bound {
    BoundNone = 0,
    BoundUpper = 1,
    BoundLower = 2,
    BoundExact = 1 | 2
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum NodeType {
    NonPV,
    PV
}

#[derive(EnumCountMacro, EnumIter, Clone, Copy, PartialEq)]
pub enum Material {
    C6,
    C5,
    F4,
    B4,
    F3,
    B3,
    F2,
    B2,
    F1,
    B1,
}

#[derive(Clone, Copy)]
pub struct MaterialCount {
    c6: i32,
    c5: i32,
    f4: i32,
    b4: i32,
    f3: i32,
    b3: i32,
    f2: i32,
    b2: i32,
    f1: i32,
    b1: i32,
}

pub type Pv = [Move; STACK_SIZE as usize];

#[derive(Clone, Copy)]
pub struct SearchStackElement {
    pub pv: Option<Pv>,
    pub killers: [Move; 2],
}

impl SearchStackElement {
    const fn new() -> SearchStackElement {
        Self {
            pv: None,
            killers: [0; 2]
        }
    }

    pub fn update_killers(&mut self, m: Move) {
        if self.killers[0] != m {
            self.killers[1] = self.killers[0];
            self.killers[0] = m;
        }
    }
}

pub type SearchStack = [SearchStackElement; STACK_SIZE as usize];

impl Index<Material> for MaterialCount {
    type Output = i32;

    fn index(&self, index: Material) -> &Self::Output {
        match index {
            // Material::MATERIAL_NUM => &self.material_num,
            // Material::MATERIAL_NONE => &self.MATERIAL_NONE,
            Material::C6 => &self.c6,
            Material::C5 => &self.c5,
            Material::F4 => &self.f4,
            Material::B4 => &self.b4,
            Material::F3 => &self.f3,
            Material::B3 => &self.b3,
            Material::F2 => &self.f2,
            Material::B2 => &self.b2,
            Material::F1 => &self.f1,
            Material::B1 => &self.b1
        }
    }
}

impl IndexMut<Material> for MaterialCount {
    fn index_mut(&mut self, index: Material) -> &mut Self::Output {
        match index {
            // Material::MATERIAL_NUM => &mut self.material_num,
            // Material::MATERIAL_NONE => &mut self.MATERIAL_NONE,
            Material::C6 => &mut self.c6,
            Material::C5 => &mut self.c5,
            Material::F4 => &mut self.f4,
            Material::B4 => &mut self.b4,
            Material::F3 => &mut self.f3,
            Material::B3 => &mut self.b3,
            Material::F2 => &mut self.f2,
            Material::B2 => &mut self.b2,
            Material::F1 => &mut self.f1,
            Material::B1 => &mut self.b1
        }
    }
}

impl Index<u32> for MaterialCount {
    type Output = i32;

    fn index(&self, index: u32) -> &Self::Output {
        match index {
            // Material::MATERIAL_NUM => &self.material_num,
            // Material::MATERIAL_NONE => &self.MATERIAL_NONE,
            0 => &self.c6,
            1 => &self.c5,
            2 => &self.f4,
            3 => &self.b4,
            4 => &self.f3,
            5 => &self.b3,
            6 => &self.f2,
            7 => &self.b2,
            8 => &self.f1,
            9 => &self.b1,
            _ => unreachable!()
        }
    }
}

impl IndexMut<u32> for MaterialCount {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        match index {
            // Material::MATERIAL_NUM => &mut self.material_num,
            // Material::MATERIAL_NONE => &mut self.MATERIAL_NONE,
            0 => &mut self.c6,
            1 => &mut self.c5,
            2 => &mut self.f4,
            3 => &mut self.b4,
            4 => &mut self.f3,
            5 => &mut self.b3,
            6 => &mut self.f2,
            7 => &mut self.b2,
            8 => &mut self.f1,
            9 => &mut self.b1,
            _ => unreachable!()
        }
    }
}

impl MaterialCount {
    fn new() -> MaterialCount {
        Self {
            c6: 0,
            c5: 0,
            f4: 0,
            b4: 0,
            f3: 0,
            b3: 0,
            f2: 0,
            b2: 0,
            f1: 0,
            b1: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct RootExtMove {
    pub score: Score,
    pub depth: Depth,
    pub pv: Pv,
}

impl RootExtMove {
    pub fn new() -> RootExtMove {
        Self {
            score: SCORE_NONE,
            depth: 0,
            pv: [0; STACK_SIZE as usize]
        }
    }

    pub fn set(&mut self, s: Score, d: Depth, p: Option<Pv>) {
        self.score = s;
        self.depth = d;
        unsafe {
            self.pv = p.unwrap_unchecked();
        }
    }
}

pub type CounterMoveHistory = [Move; MOVE_CAPACITY as usize];

#[derive(Clone, Copy)]
pub enum Stage {
    MainTT,
    MainINIT,
    MainPICK,
    MainEND,
    VcfTT,
    VcfINIT,
    VcfPICK,
    VcfEND
}

impl Add<i32> for Stage {
    type Output = Stage;

    fn add(self, i: i32) -> Stage {
        match self as i32 + i {
            0 => Stage::MainTT,
            1 => Stage::MainINIT,
            2 => Stage::MainPICK,
            3 => Stage::MainEND,
            4 => Stage::VcfTT,
            5 => Stage::VcfINIT,
            6 => Stage::VcfPICK,
            7 => Stage::VcfEND,
            _ => unreachable!(),
        }
    }
}

impl Sub<i32> for Stage {
    type Output = Stage;

    fn sub(self, i: i32) -> Stage {
        match self as i32 - i {
            0 => Stage::MainTT,
            1 => Stage::MainINIT,
            2 => Stage::MainPICK,
            3 => Stage::MainEND,
            4 => Stage::VcfTT,
            5 => Stage::VcfINIT,
            6 => Stage::VcfPICK,
            7 => Stage::VcfEND,
            _ => unreachable!(),
        }
    }
}

impl Div<i32> for Stage {
    type Output = Stage;

    fn div(self, i: i32) -> Stage {
        match self as i32 / i {
            0 => Stage::MainTT,
            1 => Stage::MainINIT,
            2 => Stage::MainPICK,
            3 => Stage::MainEND,
            4 => Stage::VcfTT,
            5 => Stage::VcfINIT,
            6 => Stage::VcfPICK,
            7 => Stage::VcfEND,
            _ => unreachable!(),
        }
    }
}

impl Mul<i32> for Stage {
    type Output = Stage;

    fn mul(self, i: i32) -> Stage {
        match self as i32 * i {
            0 => Stage::MainTT,
            1 => Stage::MainINIT,
            2 => Stage::MainPICK,
            3 => Stage::MainEND,
            4 => Stage::VcfTT,
            5 => Stage::VcfINIT,
            6 => Stage::VcfPICK,
            7 => Stage::VcfEND,
            _ => unreachable!(),
        }
    }
}


#[derive(Default, Clone, Copy)]
pub struct BitBoard {
    bitboard: [u64; BITBOARD_SIZE as usize]
}

impl BitBoard {
    fn contains(&self, m: Move) -> bool {
        (self.bitboard[(m >> 6) as usize] & ((1 as u64) << (m & 63))) != 0
    }

    fn insert(&mut self, m: Move) {
        self.bitboard[(m >> 6) as usize] |= (1 as u64) << (m & 63);
    }

    fn remove(&mut self, m: Move) {
        self.bitboard[(m >> 6) as usize] &= !((1 as u64) << (m & 63));
    }

    fn reset(&mut self) {
        for _bitboard in self.bitboard.iter_mut() {
            *_bitboard = 0;
        }
    }
}

#[derive(Clone, Copy)]
pub struct MoveList<T: Copy> {
    off_the_end: usize,
    _movelist: [T; (MOVE_SIZE + 1) as usize],
    bitboard: BitBoard
}

impl<T: Copy> MoveList<T> {
    fn begin(&self, add: Option<usize>) -> T {
        match add {
            Some(a) => self._movelist[0 + a],
            None => self._movelist[0]
        }
    }

    fn end(&self, sub: Option<usize>) -> T {
        match sub {
            Some(s) => self._movelist[self.off_the_end - s],
            None => self._movelist[self.off_the_end]
        }
    }

    fn size(&self) -> i32 {
        return self.off_the_end as i32;
    }

    fn reset(&mut self) {
        self.off_the_end = 0;
        self.bitboard.reset();
    }

    fn contains(&self, m: Move) -> bool {
        return self.bitboard.contains(m);
    }

    fn swap(&mut self, it1: usize, it2: usize) {
        let tmp: T = self._movelist[it1];
        self._movelist[it1] = self._movelist[it2];
        self._movelist[it2] = tmp;
    }
}

impl MoveList<Move> {
    fn new() -> MoveList<Move> {
        Self {
            off_the_end: 0,
            _movelist: [MOVE_NONE; (MOVE_SIZE + 1) as usize],
            bitboard: Default::default()
        }
    }

    fn insert(&mut self, m: Move) {
        if !self.contains(m) {
            self._movelist[self.off_the_end] = m;
            self.off_the_end += 1;
            self.bitboard.insert(m);
        }
    }

    fn remove(&mut self, m: Move) {
        if self.contains(m) {
            unsafe {
                self._movelist[self._movelist.iter().position(|&x| x == m).unwrap_unchecked()] = self._movelist[self.off_the_end - 1];
            }
            self.off_the_end -= 1;
            self.bitboard.remove(m);
        }
    }
}

impl MoveList<ExtMove> {
    fn new() -> MoveList<ExtMove> {
        Self {
            off_the_end: 0,
            _movelist: [ExtMove {m: MOVE_NONE, s: SCORE_NONE}; (MOVE_SIZE + 1) as usize],
            bitboard: Default::default()
        }
    }

    fn insert(&mut self, m: Move, s: Option<Score>) {
        match s {
            Some(score) => {
                if !self.contains(m) {
                    self._movelist[self.off_the_end] = ExtMove {m, s: score};
                    self.off_the_end += 1;
                    self.bitboard.insert(m);
                }
            },
            None => {
                if !self.contains(m) {
                    self._movelist[self.off_the_end] = ExtMove {m, s: SCORE_NONE};
                    self.off_the_end += 1;
                    self.bitboard.insert(m);
                }
            }
        }
    }

    fn remove(&mut self, m: Move) {
        if self.contains(m) {
            unsafe {
                self._movelist[self._movelist.iter().position(|&x| x.m == m).unwrap_unchecked()] = self._movelist[self.off_the_end - 1];
            }
            self.off_the_end -= 1;
            self.bitboard.remove(m);
        }
    }
    
    fn max(&self, start: usize, end: usize) -> usize {
        let mut max_i: usize = 0;
        let mut max_val: Score = SCORE_NONE;

        for (i, em) in self._movelist.iter().enumerate() {
            if i >= start && i < end {
                if em.s != SCORE_NONE {
                    if max_val == SCORE_NONE || max_val < em.s {
                        max_i = i;
                        max_val = em.s;
                    }
                }
            }
        }

        max_i
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct ExtMove {
    pub m: Move,
    pub s: Score,
}

pub struct MoveGen {
    movelist: MoveList<ExtMove>,
    picked: usize,
    stage: Stage,
    tt_move: Move,
    root_node: bool,
    ply: Depth,
    killers: [Move; 2],
    counter_move: Move,
}

pub enum GenType {
    WLD,
    DefendB4,
    DefendF3,
    DEFAULT,
    LARGE,
    MAIN,
    TTMOVE,
    VCFROOT,
    VCFCHILD
}

pub trait MoveGenArgs {
    fn new(self) -> MoveGen;
}

impl MoveGenArgs for () {
    fn new(self) -> MoveGen {
        unsafe {
            let (stg, ttm, rnode, p, karr1, karr2, cm) = (Stage::MainTT, MOVE_NONE, false, DEPTH_NONE, MOVE_NONE, MOVE_NONE, MOVE_NONE);
            let piece_cnt: usize = BOARD.as_ref().unwrap().piece_cnt;
            // Late movelist update
            if !BOARD.as_ref().unwrap().updated_move_list[piece_cnt] {
                let last_move = BOARD.as_ref().unwrap().last_move(1);
                BOARD.as_mut().unwrap().update_movelist(last_move);
                BOARD.as_mut().unwrap().updated_move_list[piece_cnt] = true;
            }
            let tt_move: Move = if ttm != MOVE_NONE && BOARD.as_ref().unwrap().m_list_stack[piece_cnt].contains(ttm) {ttm} else {MOVE_NONE};
    
            MoveGen {
                movelist: MoveList::<ExtMove>::new(),
                picked: 0,
                stage: stg + if tt_move == MOVE_NONE {1} else {0},
                tt_move,
                root_node: rnode,
                ply: p,
                killers: if is_ok_move(karr1) && is_ok_move(karr2) {[karr1, karr2]} else {[MOVE_NONE, MOVE_NONE]},
                counter_move: cm,
            }
        }
    }
}

impl MoveGenArgs for (Stage, Move, bool) {
    fn new(self) -> MoveGen {
        unsafe {
            let (stg, ttm, rnode) = self;
            let (p, karr1, karr2, cm) = (DEPTH_NONE, MOVE_NONE, MOVE_NONE, MOVE_NONE);
            // Late movelist update
            if !BOARD.as_ref().unwrap().updated_move_list[BOARD.as_ref().unwrap().piece_cnt] {
                let last_move = BOARD.as_ref().unwrap().last_move(1);
                BOARD.as_mut().unwrap().update_movelist(last_move);
                BOARD.as_mut().unwrap().updated_move_list[BOARD.as_ref().unwrap().piece_cnt] = true;
            }
            let tt_move: Move = if ttm != MOVE_NONE && BOARD.as_ref().unwrap().m_list_stack[BOARD.as_ref().unwrap().piece_cnt].contains(ttm) {ttm} else {MOVE_NONE};
    
            MoveGen {
                movelist: MoveList::<ExtMove>::new(),
                picked: 0,
                stage: stg + if tt_move == MOVE_NONE {1} else {0},
                tt_move,
                root_node: rnode,
                ply: p,
                killers: if is_ok_move(karr1) && is_ok_move(karr2) {[karr1, karr2]} else {[MOVE_NONE, MOVE_NONE]},
                counter_move: cm,
            }
        }
    }
}

impl MoveGenArgs for (Stage, Move, bool, Depth, Move, Move, Move) {
    fn new(self) -> MoveGen {
        unsafe {
            let (stg, ttm, rnode, p, karr1, karr2, cm) = self;
            let piece_cnt: usize = BOARD.as_ref().unwrap().piece_cnt;
            // Late movelist update
            if !BOARD.as_ref().unwrap().updated_move_list[piece_cnt] {
                let last_move = BOARD.as_ref().unwrap().last_move(1);
                BOARD.as_mut().unwrap().update_movelist(last_move);
                BOARD.as_mut().unwrap().updated_move_list[piece_cnt] = true;
            }
            let tt_move: Move = if ttm != MOVE_NONE && BOARD.as_ref().unwrap().m_list_stack[piece_cnt].contains(ttm) {ttm} else {MOVE_NONE};
    
            MoveGen {
                movelist: MoveList::<ExtMove>::new(),
                picked: 0,
                stage: stg + if tt_move == MOVE_NONE {1} else {0},
                tt_move,
                root_node: rnode,
                ply: p,
                killers: if is_ok_move(karr1) && is_ok_move(karr2) {[karr1, karr2]} else {[MOVE_NONE, MOVE_NONE]},
                counter_move: cm,
            }
        }
    }
}

impl MoveGen {
    pub fn new<T>(args: T) -> MoveGen where T: MoveGenArgs {
        args.new()
    }

    fn begin(&self) -> ExtMove {
        return self.movelist.begin(None);
    }

    fn end(&self) -> ExtMove {
        return self.movelist.end(None);
    }

    fn current(&self) -> ExtMove {
        return self.movelist.begin(Some(self.picked as usize));
    }

    pub fn size(&self) -> i32 {
        return self.movelist.size();
    }

    pub fn generate(&mut self, gt: GenType) -> ExtMove {
        unsafe {
            match gt {
                GenType::WLD => {
                    let mut offset: Score = 0;

                    BOARD.as_ref().unwrap().check_wld(&mut offset);

                    let m_list_stack: MoveList<Move> = BOARD.as_ref().unwrap().m_list_stack[BOARD.as_ref().unwrap().piece_cnt];

                    // If we have F4 or B4, we form C5 to win
                    if BOARD.as_ref().unwrap().query(BOARD.as_ref().unwrap().side_to_move, Material::F4) > 0 || BOARD.as_ref().unwrap().query(BOARD.as_ref().unwrap().side_to_move, Material::B4) > 0 {
                        for i in 0..m_list_stack.size() {
                            let m: Move = m_list_stack.begin(Some(i as usize));
                            if BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, m, Material::C5) > 0 {
                                self.movelist.insert(m, Some(WIN_SCORE - offset as Score));
                                return self.begin();
                            }
                        }
                    }

                    // If opponent has F4, we defend and lose. Cannot call pbd->query in this case
                    if BOARD.as_ref().unwrap().query(BOARD.as_ref().unwrap().oppo_to_move, Material::F4) > 0 {
                        for i in 0..m_list_stack.size() {
                            let m: Move = m_list_stack.begin(Some(i as usize));
                            BOARD.as_mut().unwrap().do_move(m);
                            if BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, m, Material::F4) < 0 {
                                BOARD.as_mut().unwrap().undo_move();
                                self.movelist.insert(m, None);
                                return self.begin();
                            }
                            BOARD.as_mut().unwrap().undo_move();
                        }
                    }

                    // If opponent has several B4, we defend and lose
                    if BOARD.as_ref().unwrap().query(BOARD.as_ref().unwrap().oppo_to_move, Material::B4) >= 2 {
                        self.movelist.insert(BOARD.as_ref().unwrap().defend_b4(), Some(-WIN_SCORE + offset as Score));
                        return self.begin();
                    }

                    // If we have F3 and neither has B4, we form F4 to win
                    if BOARD.as_ref().unwrap().query(BOARD.as_ref().unwrap().side_to_move, Material::F3) > 0 && BOARD.as_ref().unwrap().query(BOARD.as_ref().unwrap().oppo_to_move, Material::B4) == 0 {
                        for i in BOARD.as_ref().unwrap().f3_stack[BOARD.as_ref().unwrap().piece_cnt].iter() {
                            if i.color == BOARD.as_ref().unwrap().side_to_move {
                                self.movelist.insert(i.f4a[0], Some(WIN_SCORE - offset as Score));
                                return self.begin();
                            }
                        }
                    }

                    ExtMove {m: MOVE_NONE, s: SCORE_NONE}
                },
                GenType::DefendB4 => {
                    self.movelist.insert(BOARD.as_ref().unwrap().defend_b4(), None);
                
                    return self.begin();
                },
                GenType::DefendF3 => {
                    for i in BOARD.as_ref().unwrap().f3_stack[BOARD.as_ref().unwrap().piece_cnt].iter() {
                        for m in i.f3d.iter() {
                            if *m != MOVE_NONE {
                                self.movelist.insert(*m, Some(self.score_of(*m) + BONUS_F3D));
                            }
                        }
                    }

                    let m_list_stack: MoveList<i16> = BOARD.as_ref().unwrap().m_list_stack[BOARD.as_ref().unwrap().piece_cnt];
                    for i in 0..m_list_stack.size() {
                        let m: Move = m_list_stack.begin(Some(i as usize));
                        if BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, m, Material::B4) > 0 {
                            self.movelist.insert(m, Some(self.score_of(m)));
                        }
                    }

                    return self.begin();
                },
                GenType::DEFAULT => {
                    let m_list_stack: MoveList<Move> = BOARD.as_ref().unwrap().m_list_stack[BOARD.as_ref().unwrap().piece_cnt];
                    for i in 0..m_list_stack.size() {
                        let m: Move = m_list_stack.begin(Some(i as usize));
                        self.movelist.insert(m, Some(self.score_of(m)));
                    }
                
                    return self.begin();
                },
                GenType::LARGE => {
                    for i in 0..BOARD.as_ref().unwrap().piece_cnt {
                        let m: Move = BOARD.as_ref().unwrap().piece_list[i as usize];
                        for i in N3 {
                            if BOARD.as_ref().unwrap().is_empty(m + i) && !self.movelist.contains(m + i) {
                                self.movelist.insert(m + i, Some(self.score_of(m + i)));
                            }
                        }
                    }
                
                    return self.begin();
                },
                GenType::MAIN => {
                    if BOARD.as_ref().unwrap().query(BOARD.as_ref().unwrap().oppo_to_move, Material::B4) > 0 {
                        self.generate(GenType::DefendB4);
                    } else if BOARD.as_ref().unwrap().query(BOARD.as_ref().unwrap().oppo_to_move, Material::F3) > 0 {
                        self.generate(GenType::DefendF3);
                    } else if self.ply < 2 && BOARD.as_ref().unwrap().piece_cnt < 5 {
                        self.generate(GenType::LARGE);
                    } else {
                        self.generate(GenType::DEFAULT);
                    }
                
                    return self.begin();
                },
                GenType::TTMOVE => {
                    self.movelist.insert(self.tt_move, None);

                    return self.begin();
                },
                GenType::VCFROOT => {
                    if BOARD.as_ref().unwrap().query(BOARD.as_ref().unwrap().oppo_to_move, Material::B4) > 0 {
                        if BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, BOARD.as_ref().unwrap().defend_b4(), Material::F4) + BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, BOARD.as_ref().unwrap().defend_b4(), Material::B4) > 0 {
                            self.movelist.insert(BOARD.as_ref().unwrap().defend_b4(), None);
                        }
                    } else {
                        let m_list_stack: MoveList<Move> = BOARD.as_ref().unwrap().m_list_stack[BOARD.as_ref().unwrap().piece_cnt];
                        
                        for i in 0..m_list_stack.size() {
                            let m: Move = m_list_stack.begin(Some(i as usize));

                            if BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, m, Material::B4) > 0 && (BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, m, Material::B4) >= 2 || BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, m, Material::F3) > 0 || BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, m, Material::B3) > 0 || BOARD.as_ref().unwrap().query_vcf(BOARD.as_ref().unwrap().side_to_move, m) > 0) {
                                self.movelist.insert(m, Some(BOARD.as_ref().unwrap().see_of(m)));
                            }
                        }
                    }

                    return self.begin();
                },
                GenType::VCFCHILD => {
                    if BOARD.as_ref().unwrap().query(BOARD.as_ref().unwrap().oppo_to_move, Material::B4) > 0 {
                        if BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, BOARD.as_ref().unwrap().defend_b4(), Material::F4) + BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, BOARD.as_ref().unwrap().defend_b4(), Material::B4) > 0 {
                            self.movelist.insert(BOARD.as_ref().unwrap().defend_b4(), None);
                        }
                    } else {

                        for i in N4 {
                            let m: Move = BOARD.as_ref().unwrap().last_move(2) + i;
                            if BOARD.as_ref().unwrap().is_empty(m) && BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, m, Material::B4) > 0
                                && (BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, m, Material::B4) >= 2
                                    || BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, m, Material::F3) > 0
                                    || BOARD.as_ref().unwrap().query_us_inc(BOARD.as_ref().unwrap().side_to_move, m, Material::B3) > 0
                                    || BOARD.as_ref().unwrap().query_vcf(BOARD.as_ref().unwrap().side_to_move, m) > 0) {
                                self.movelist.insert(m, Some(BOARD.as_ref().unwrap().see_of(m)));
                            }
                        }
                    }

                    return self.begin();
                },
            }
        }
    }

    unsafe fn score_of(&self, m: Move) -> Score {
        let mut score: Score = BOARD.as_ref().unwrap().see_of(m);
        let mut dist: [i32; 4] = [0; 4];

        for i in 0..4 {
            dist[i] = if BOARD.as_ref().unwrap().piece_cnt >= i + 1 {_distance_between(m, BOARD.as_ref().unwrap().last_move(i + 1))} else {0};
        }
    
        if dist[0] > 4 && dist[1] > 4 && dist[2] > 4 && dist[3] > 4 {
            score /= 4;
        } else if dist[0] > 4 && dist[1] > 4 {
            score /= 2;
        }

        score += if m == self.killers[0] || m == self.killers[1] || m == self.counter_move {BONUS_REFUTATION} else {0};
    
        return score;
    }

    pub fn next_move(&mut self) -> ExtMove {
        let mut ret: ExtMove = ExtMove {m: MOVE_NONE, s: SCORE_NONE};

        loop {
            match self.stage {
                Stage::MainTT | Stage::VcfTT => {
                    self.generate(GenType::TTMOVE);
                    self.picked += 1;
                    self.stage = self.stage + 1;

                    ret = self.begin();
                    break;
                }
                Stage::MainINIT => {
                    self.generate(GenType::MAIN);
                    self.stage = self.stage + 1;
                }
                Stage::VcfINIT => {
                    if self.root_node {
                        self.generate(GenType::VCFROOT);
                    } else {
                        self.generate(GenType::VCFCHILD);
                    }
                    self.stage = self.stage + 1;
                }
                Stage::MainPICK | Stage::VcfPICK => {
                    if self.current() == self.end() {
                        self.stage = self.stage + 1;
                        continue;
                    }
                    let bestIt = self.movelist.max(self.picked, self.size() as usize);
                    ret = self.movelist.begin(Some(bestIt));
                    self.movelist.swap(self.picked, bestIt);
                    self.picked += 1;
                    
                    break;
                }
                _ => {
                    break;
                }
            }
        }
        ret
    }
}

use array_init::array_init;

#[derive(Copy, Clone, PartialEq)]
pub enum Color {
    Black,
    White,
    Hide,
    None,
}

#[derive(EnumIter, EnumCountMacro, Clone, Copy, PartialEq)]
pub enum Direction {
    DRank,
    DADiag,
    DFile,
    DMDiag
}

#[derive(Clone, Copy)]
pub struct DirectionCount {
    DRank: i32,
    DADiag: i32,
    DFile: i32,
    DMDiag: i32,
}

impl DirectionCount {
    fn new() -> DirectionCount {
        Self {
            DRank: 0,
            DADiag: 0,
            DFile: 0,
            DMDiag: 0,
        }
    }
}

impl Index<Direction> for DirectionCount {
    type Output = i32;

    fn index(&self, direction: Direction) -> &Self::Output {
        match direction {
            Direction::DRank => &self.DRank,
            Direction::DADiag => &self.DADiag,
            Direction::DFile => &self.DFile,
            Direction::DMDiag => &self.DMDiag,
        }
    }
}

impl IndexMut<Direction> for DirectionCount {
    fn index_mut(&mut self, direction: Direction) -> &mut Self::Output {
        match direction {
            Direction::DRank => &mut self.DRank,
            Direction::DADiag => &mut self.DADiag,
            Direction::DFile => &mut self.DFile,
            Direction::DMDiag => &mut self.DMDiag,
        }
    }
}


#[derive(Clone, Copy, PartialEq)]
pub struct F3Pack {
    color: Color,
    m: Move,
    direction: Direction,
    f4a: [Move; 2],
    f3d: [Move; 3],
    vind: i32,
    gen: i32,
}

const D: [Move; 4] = [1, (1 << BOARD_SIDE_BIT) - 1, (1 << BOARD_SIDE_BIT), (1 << BOARD_SIDE_BIT) + 1];
const N2: [Move; 16] = [
    -D[3] - D[3],
    -D[2] - D[2],
    -D[1] - D[1],
    -D[3],
    -D[2],
    -D[1],
    -D[0] - D[0],
    -D[0],
    D[0],
    D[0] + D[0],
    D[1],
    D[2],
    D[3],
    D[1] + D[1],
    D[2] + D[2],
    D[3] + D[3],
];

const N3: [Move; 32] = [
    -D[3] - D[3] - D[3],
    -D[2] - D[2] - D[2],
    -D[1] - D[1] - D[1],
    -D[3] - D[3],
    -D[3] - D[2],
    -D[2] - D[2],
    -D[2] - D[1],
    -D[1] - D[1],
    -D[3] - D[0],
    -D[3],
    -D[2],
    -D[1],
    -D[1] + D[0],
    -D[0] - D[0] - D[0],
    -D[0] - D[0],
    -D[0],
    D[0],
    D[0] + D[0],
    D[0] + D[0] + D[0],
    D[1] - D[0],
    D[1],
    D[2],
    D[3],
    D[3] + D[0],
    D[1] + D[1],
    D[1] + D[2],
    D[2] + D[2],
    D[3] + D[2],
    D[3] + D[3],
    D[1] + D[1] + D[1],
    D[2] + D[2] + D[2],
    D[3] + D[3] + D[3],
];

const N4: [Move; 32] = [
    -D[3] - D[3] - D[3] - D[3],
    -D[2] - D[2] - D[2] - D[2],
    -D[1] - D[1] - D[1] - D[1],
    -D[3] - D[3] - D[3],
    -D[2] - D[2] - D[2],
    -D[1] - D[1] - D[1],
    -D[3] - D[3],
    -D[2] - D[2],
    -D[1] - D[1],
    -D[3],
    -D[2],
    -D[1],
    -D[0] - D[0] - D[0] - D[0],
    -D[0] - D[0] - D[0],
    -D[0] - D[0],
    -D[0],
    D[0],
    D[0] + D[0],
    D[0] + D[0] + D[0],
    D[0] + D[0] + D[0] + D[0],
    D[1],
    D[2],
    D[3],
    D[1] + D[1],
    D[2] + D[2],
    D[3] + D[3],
    D[1] + D[1] + D[1],
    D[2] + D[2] + D[2],
    D[3] + D[3] + D[3],
    D[1] + D[1] + D[1] + D[1],
    D[2] + D[2] + D[2] + D[2],
    D[3] + D[3] + D[3] + D[3],
];

impl F3Pack {
    const f4a_size: i32 = 2;
    const f3d_size: i32 = 3;
    fn new(c: Color, m: Move, d: Direction, ind: i32) -> F3Pack {
        Self {
            color: c,
            m,
            direction: d,
            f4a: [MOVE_NONE; 2],
            f3d: [MOVE_NONE; 3],
            vind: ind,
            gen: 0,
        }
    }

    fn valid(&self) -> bool {
        if self.f4a[0] == MOVE_NONE || self.f3d[0] == MOVE_NONE {false} else {true}
    }

    unsafe fn update_renju(&mut self) {
        let mut ind1: i32 = 0;
        let mut ind2: i32 = 0;

        self.f4a.iter_mut().for_each(|x| *x = MOVE_NONE);
        self.f3d.iter_mut().for_each(|x| *x = MOVE_NONE);

        for i in -4..5 {
            let m = self.m + D[self.direction as usize] * i;
            if BOARD.as_ref().unwrap().is_empty(m) {
                if ind1 < F3Pack::f4a_size && BOARD.as_ref().unwrap().query_us_inc(self.color, m, Material::C6) == 0 && BOARD.as_ref().unwrap().query_us_inc(self.color, m, Material::F4) == 1 && BOARD.as_ref().unwrap().query_us_inc(self.color, m, Material::B4) == 0 && BOARD.as_ref().unwrap().query_us_inc(self.color, m, Material::F3) <= 1 {
                    self.f4a[ind1 as usize] = m;
                    ind1 += 1;
                }

                if ind2 < F3Pack::f3d_size && BOARD.as_ref().unwrap().query_opp_dec(if self.color == Color::White {Color::Black} else {Color::White}, m, Material::F3) > 0 {
                    self.f3d[ind2 as usize] = m;
                    ind2 += 1;
                }
            }
        }
    }

    unsafe fn update_free(&mut self) {
        let mut ind1: i32 = 0;
        let mut ind2: i32 = 0;

        self.f4a.iter_mut().for_each(|x| *x = MOVE_NONE);
        self.f3d.iter_mut().for_each(|x| *x = MOVE_NONE);

        for i in -4..5 {
            let m = self.m + D[self.direction as usize] * i;
            if BOARD.as_ref().unwrap().is_empty(m) {
                if ind1 < F3Pack::f4a_size && BOARD.as_ref().unwrap().query_us_inc(self.color, m, Material::F4) > 0 {
                    self.f4a[ind1 as usize] = m;
                    ind1 += 1;
                }

                if ind2 < F3Pack::f3d_size && BOARD.as_ref().unwrap().query_opp_dec(if self.color == Color::White {Color::Black} else {Color::White}, m, Material::F3) > 0 {
                    self.f3d[ind2 as usize] = m;
                    ind2 += 1;
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Interval {
    begin_p: i32,
    end_p: i32,
}

impl Interval {
    fn init(&mut self, bg: i32, ed: i32) {
        self.begin_p = bg;
        self.end_p = ed;
    }

    fn begin(&self) -> i32 {
        self.begin_p
    }

    fn end(&self) -> i32 {
        self.end_p
    }

    fn length(&self) -> i32 {
        self.end_p - self.begin_p
    }

    fn set_begin(&mut self, bg: i32) {
        self.begin_p = bg;
    }

    fn set_end(&mut self, ed: i32) {
        self.end_p = ed;
    }
}

type F3Packs = Vec<F3Pack>;

pub struct Board {
    _board: [Color; MOVE_CAPACITY as usize],
    zobrists: [[ZobristKey; MOVE_CAPACITY as usize]; 2],
    mat: [[MaterialCount; 2]; STACK_SIZE as usize],
    mat_inc: [MaterialCount; 2],
    f3_formed_cnt: [i32; 2],
    piece_list: [Move; MOVE_SIZE as usize],
    updated_move_list: [bool; STACK_SIZE as usize],
    updated_interval: [bool; STACK_SIZE as usize],
    m_list_stack: [MoveList<Move>; STACK_SIZE as usize],
    see: [[[u32; BOARD_SIDE as usize]; VECTOR_SIZE as usize]; 2],
    index_on_table: [DirectionCount; MOVE_CAPACITY as usize],
    index_of_table: [DirectionCount; MOVE_CAPACITY as usize],
    b4d_stack: [Move; STACK_SIZE as usize],
    f3_stack: [F3Packs; STACK_SIZE as usize],
    score: [[Score; 2]; STACK_SIZE as usize],
    see_table: [Score; 16384],
    interval: [[[Interval; BOARD_SIZE as usize]; VECTOR_SIZE as usize]; 2],
    see_stack: [[[[u32; BOARD_SIDE as usize]; 4]; 2]; STACK_SIZE as usize],
    vector_board: [[u32; VECTOR_SIZE as usize]; 2],
    pub side_to_move: Color,
    pub oppo_to_move: Color,
    pub key: u64,
    pub piece_cnt: usize,
}

pub fn reset_pv(pv: &mut Pv) {
    (*pv)[0] = MOVE_NONE;
}

pub unsafe fn is_empty(pv: Option<&Pv>) -> bool {
    pv.unwrap_unchecked()[0] == MOVE_NONE
}

impl Board {
    pub fn new() -> Board {
        let mut prng = PRNG::new(1070372);

        let mut bd: Board = Self {
            _board: array_init(|i| if is_ok_move(i as Move) {Color::Hide} else {Color::None} ),
            key: 0,
            zobrists: array_init(|_| array_init(|_| prng.rand64())),
            mat: array_init(|_| array_init(|_| MaterialCount::new())),
            mat_inc: array_init(|_| MaterialCount::new()),
            f3_formed_cnt: array_init(|_| 0),
            piece_list: array_init(|_| MOVE_NONE),
            updated_move_list: array_init(|_| false),
            updated_interval: array_init(|_| false),
            m_list_stack: array_init(|_| MoveList::<Move>::new()),
            side_to_move: Color::Black,
            oppo_to_move: Color::White,
            see: array_init(|_| array_init(|_| array_init(|_| 0))),
            index_on_table: array_init(|_| DirectionCount::new()),
            index_of_table: array_init(|_| DirectionCount::new()),
            b4d_stack: array_init(|_| MOVE_NONE),
            f3_stack: array_init(|_| Vec::new()),
            score: array_init(|_| array_init(|_| 0)),
            see_table: array_init(|_| 0),
            interval: array_init(|_| array_init(|_| array_init(|_| Interval { begin_p: 0, end_p: 0 }))),
            see_stack: array_init(|_| array_init(|_| array_init(|_| array_init(|_| 0)))),
            vector_board: array_init(|_| array_init(|_| 0)),
            piece_cnt: 0,
        };

        bd.init();
        bd.reset();

        bd
    }

    fn init(&mut self) {
        for d in Direction::iter() {
            for m in 0..MOVE_CAPACITY as Move {
                if is_ok_move(m) {
                    self.index_of_table[m as usize][d] = index_of_helper(m, d);
                    self.index_on_table[m as usize][d] = index_on_helper(m, d);
                }
            }
        }

        for i in 0..16384 as u32 {
            for m in [Material::B4, Material::F3, Material::B3, Material::F2, Material::B2, Material::F1, Material::B1] {
                if (i & ((1 as u32) << ((m as u32) - 3))) != 0 {
                    self.see_table[i as usize] += SEE_HELPER[m as usize];
                }
            }

            for m in [Material::B4, Material::F3, Material::B3, Material::F2, Material::B2, Material::F1, Material::B1] {
                if m == Material::B4 || m == Material::F3 {
                    continue;
                }

                if (i & ((1 as u32) << ((m as u32) + 4))) != 0 {
                    self.see_table[i as usize] -= SEE_HELPER[m as usize];
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.piece_cnt = 0;
        self.side_to_move = Color::Black;
        self.oppo_to_move = Color::White;
        self.key = 0;

        for __material in self.mat.iter_mut() {
            for ___material in __material.iter_mut() {
                for mat in Material::iter() {
                    ___material[mat] = 0;
                }
            }
        }

        for _score in self.score.iter_mut() {
            for __score in _score.iter_mut() {
                *__score = 0;
            }
        }

        for _see in self.see.iter_mut() {
            for __see in _see.iter_mut() {
                for ___see in __see.iter_mut() {
                    *___see = 0;
                }
            }
        }

        for _vector_board in self.vector_board.iter_mut() {
            for __vector_board in _vector_board.iter_mut() {
                *__vector_board = 0;
            }
        }

        for _updated_interval in self.updated_interval.iter_mut() {
            *_updated_interval = false;
        }

        for _updated_move_list in self.updated_move_list.iter_mut() {
            *_updated_move_list = false;
        }

        self.m_list_stack[self.piece_cnt].reset();

        for m in 0..MOVE_CAPACITY as Move {
            self._board[m as usize] = if is_ok_move(m) {Color::Hide} else {Color::None};
        }

        for p in 0..2 as usize {
            for m in 0..MOVE_CAPACITY as Move {
                if is_ok_move(m) {
                    for d in Direction::iter() {
                        self.interval[p][self.index_of(m, d) as usize][self.index_on(m, d) as usize].set_begin(0);
                        self.interval[p][self.index_of(m, d) as usize][self.index_on(m, d) as usize].set_end(_distance_between(_start_of(m, d), _end_of(m, d)) + 1);
                    }
                }
            }
        }

        for p in 0..2 {
            for d in Direction::iter() {
                for m in 0..MOVE_CAPACITY as Move {
                    if is_ok_move(m) {
                        self.line_update_inc(p, m, oppo_direction(d),self.interval[p as usize][self.index_of(m, oppo_direction(d)) as usize][self.index_on(m, oppo_direction(d)) as usize]);
                    }
                }           
            }
        }

        self.updated_interval[self.piece_cnt] = true;
        self.updated_move_list[self.piece_cnt] = true;
    }

    fn get_color(&self, m: &Move) -> Color {
        self._board[*m as usize]
    }

    fn switch_side_to_move(&mut self) {
        match self.side_to_move {
            Color::White => {
                self.side_to_move = Color::Black;
                self.oppo_to_move = Color::White;
            },
            Color::Black => {
                self.side_to_move = Color::White;
                self.oppo_to_move = Color::Black;
            },
            _ => {}
        }
    }

    // fn key_after(&self, m: Move) -> ZobristKey {
    //     self.key ^ self.zobrists[self.side_to_move as usize][m as usize]
    // }

    fn update_interval(&mut self, m: Move) {
        let mut iof: i32;
        let mut ion: i32;

        for d in Direction::iter() {
            iof = self.index_of(m, d);
            ion = self.index_on(m, d);

            for i in self.interval[self.oppo_to_move as usize][iof as usize][ion as usize].begin()..ion {
                self.interval[self.oppo_to_move as usize][iof as usize][i as usize].set_end(ion);
            }

            for i in ion+1..self.interval[self.oppo_to_move as usize][iof as usize][ion as usize].end() {
                self.interval[self.oppo_to_move as usize][iof as usize][i as usize].set_begin(ion + 1);
            }
        }
    }


    fn line_update_inc(&mut self, p: i32, m: Move, d: Direction, itv: Interval) {
        let iof: i32 = self.index_of(m, d);
        let ion: i32 = self.index_on(m, d);

        if itv.length() < 5 {
            for i in itv.begin()..itv.end() {
                self.see[p as usize][iof as usize][i as usize] = 0;
            }
            return;
        }

        let qvb_r: usize = self.query_vector_board(p, iof, itv) as usize;
        let arr: [u32; 16] = if p == 1 {PATTERN_F[qvb_r + (1 << itv.length()) - 1]} else {PATTERN_S[qvb_r + (1 << itv.length()) - 1]};

        let mut pack: F3Pack = F3Pack::new(if p == 0 {Color::Black} else {Color::White}, m, d, iof);
        let mut form_f3: bool = false;
        let mut ind1: i32 = 0;
        let mut ind2: i32 = 0;
        let mut ptr: usize = 0;
        let mut ele: u32 = arr[ptr as usize];
        let mut mat: u32;
        mat = ele & 0xf;
        ptr += 1;

        while mat != MATERIAL_NONE {
            ele >>= 4;
            self.mat_inc[p as usize][mat] += 1;
            self.score[self.piece_cnt][p as usize] += SCORE_HELPER[mat as usize];

            if mat == Material::F3 as u32 {
                form_f3 = true;
            }

            mat = ele & 0xf;
        }

        if form_f3 && self.f3_stack[self.piece_cnt].iter().position(|x| *x == pack) == None {
            for i in itv.begin()..itv.end() {
                if (arr[ptr] & ((1 as u32) << 2)) != 0 && ind1 < F3Pack::f4a_size {
                    pack.f4a[ind1 as usize] = m + D[d as usize] * (i - ion) as Move;
                    ind1 += 1;
                }

                if (arr[ptr] & ((1 as u32) << 25)) != 0 && ind2 < F3Pack::f3d_size {
                    pack.f3d[ind2 as usize] = m + D[d as usize] * (i - ion) as Move;
                    ind2 += 1;
                }

                if (arr[ptr] & ((1 as u32) << 24)) != 0 {
                    self.b4d_stack[self.piece_cnt] = m + D[d as usize] * (i - ion) as Move;
                }

                self.see[p as usize][iof as usize][i as usize] = arr[ptr];
                ptr += 1;
            }

            self.f3_stack[self.piece_cnt].push(pack);
        } else {
            for i in itv.begin()..itv.end() {
                if (arr[ptr] & ((1 as u32) << 24)) != 0 {
                    self.b4d_stack[self.piece_cnt] = m + D[d as usize] * (i - ion) as Move;
                }

                self.see[p as usize][iof as usize][i as usize] = arr[ptr];
                ptr += 1;
            }
        }
    }

    fn line_update_dec(&mut self, p: i32, m: Move, d: Direction, itv: Interval) {
        if itv.length() < 5 {
            return;
        }

        let tmp: usize = self.query_vector_board(p, self.index_of(m, d), itv) as usize;

        if tmp == 0 {
            return;
        }

        let mut ele: u32;
        let mut mat: u32;

        if p == 1 {
            ele = PATTERN_F[tmp + (1 << itv.length()) - 1][0];
        } else {
            ele = PATTERN_S[tmp + (1 << itv.length()) - 1][0];
        }

        mat = ele & 0xf;

        while mat != MATERIAL_NONE {
            ele >>= 4;
            self.mat_inc[p as usize][mat] -= 1;
            self.score[self.piece_cnt][p as usize] -= SCORE_HELPER[mat as usize];
            mat = ele & 0xf;
        }
    }

    unsafe fn f3packs_update(&mut self) {
        let mut f3_cnt: [i32; 2] = [0, 0];
        let mut it: usize = 0;
        while it != self.f3_stack[self.piece_cnt].len() {
            let mut f3p: F3Pack = self.f3_stack[self.piece_cnt][it];

            if f3p.color == Color::Black {
                f3p.update_renju();
            } else {
                f3p.update_free();
            }

            self.f3_stack[self.piece_cnt][it] = f3p;

            if !self.f3_stack[self.piece_cnt][it].valid() {
                self.f3_stack[self.piece_cnt].remove(it);
            } else {
                let itp: usize = self.f3_stack[self.piece_cnt][it].color as usize;
                f3_cnt[itp] += 1;
                if self.f3_stack[self.piece_cnt][it].gen <= 0 {
                    self.f3_formed_cnt[itp] += 1;
                }
                self.f3_stack[self.piece_cnt][it].gen += 1;
                it += 1;
            }
        }

        for p in 0..2 {
            self.mat[self.piece_cnt][p as usize][Material::F3] = f3_cnt[p as usize];
        }

        for p in 0..2 {
            self.mat_inc[p as usize][Material::F3] = self.mat[self.piece_cnt][p as usize][Material::F3] - self.mat[self.piece_cnt - 1][p as usize][Material::F3];
        }
    }

    unsafe fn update_material_see(&mut self, m: Move) {
        for i in 0..self.mat_inc.len() {
            for mat in Material::iter() {
                self.mat_inc[i][mat] = 0;
            }
        }

        for i in 0..self.f3_formed_cnt.len() {
            self.f3_formed_cnt[i] = 0;
        }

        self.mat[self.piece_cnt] = self.mat[self.piece_cnt - 1];
        self.score[self.piece_cnt] = self.score[self.piece_cnt - 1];
        self.f3_stack[self.piece_cnt] = self.f3_stack[self.piece_cnt - 1].clone();
        self.b4d_stack[self.piece_cnt] = self.b4d_stack[self.piece_cnt - 1];

        for p in 0..2 {
            for d in Direction::iter() {
                self.see_stack[self.piece_cnt - 1][p as usize][d as usize] = self.see[p as usize][self.index_of(m, d) as usize];
            }
        }

        for p in 0..2 {
            for d in Direction::iter() {
                self.line_update_dec(p, m, d, self.interval[p as usize][self.index_of(m, d) as usize][self.index_on(m, d) as usize]);
            }
        }

        self.update_vector_board(m);
        let mut iof: i32;
        let mut ion: i32;
        let mut tmpitv: Interval = Interval { begin_p: 0, end_p: 0 };

        for d in Direction::iter() {
            iof = self.index_of(m, d);
            ion = self.index_on(m, d);

            self.line_update_inc(self.side_to_move as i32, m, d, self.interval[self.side_to_move as usize][iof as usize][ion as usize]);

            tmpitv.init(self.interval[self.oppo_to_move as usize][iof as usize][ion as usize].begin(), ion);
            self.line_update_inc(self.oppo_to_move as i32, m, d, tmpitv);

            tmpitv.init(ion + 1, self.interval[self.oppo_to_move as usize][iof as usize][ion as usize].end());
            self.line_update_inc(self.oppo_to_move as i32, m, d, tmpitv);
        }

        for p in 0..2 {
            for i in 0..10 {
                self.mat[self.piece_cnt][p][i] += self.mat_inc[p][i];
            }
        }

        self.f3packs_update();
    }

    pub unsafe fn do_move(&mut self, m: Move) {
        if self.piece_cnt > 0 && !self.updated_interval[self.piece_cnt] {
            self.switch_side_to_move();
            self.update_interval(self.last_move(1));
            self.switch_side_to_move();
            self.updated_interval[self.piece_cnt] = true;
        }
        if self.piece_cnt > 0 && !self.updated_move_list[self.piece_cnt] {
            self.update_movelist(self.last_move(1));
            self.updated_move_list[self.piece_cnt] = true;
        }
        self._board[m as usize] = self.side_to_move;
        self.piece_list[self.piece_cnt] = m;
        self.piece_cnt += 1;
        self.update_material_see(m);
        self.key ^= self.zobrists[self.side_to_move as usize][m as usize];
        self.switch_side_to_move();
        self.updated_interval[self.piece_cnt] = false;
        self.updated_move_list[self.piece_cnt] = false;
    }

    fn restore_see(&mut self, m: Move) {
        for p in 0..2 {
            for d in Direction::iter() {
                self.see[p as usize][self.index_of(m, d) as usize] = self.see_stack[self.piece_cnt][p as usize][d as usize];
            }
        }
    }
    
    fn query_vector_board(&self, p: i32, vind: i32, itv: Interval) -> i32 {
        get_bits(self.vector_board[p as usize][vind as usize], itv.begin() as u32, itv.end() as u32) as i32
    }


    fn update_vector_board(&mut self, m: Move) {
        for d in Direction::iter() {
            let ion: i32 = self.index_on(m, d);
            let iof: i32 = self.index_of(m, d);
            set_bit(&mut self.vector_board[self.side_to_move as usize][iof as usize], ion);
        }
    }

    fn restore_vector_board(&mut self, m: Move) {
        for d in Direction::iter() {
            let ind: i32 = self.index_on(m, d);
            reset_bit(&mut self.vector_board[self.side_to_move as usize][self.index_of(m, d) as usize], ind);
        }
    }

    fn restore_interval(&mut self, m: Move) {
        let mut iof: i32;
        let mut ion: i32;

        for d in Direction::iter() {
            iof = self.index_of(m, d);
            ion = self.index_on(m, d);

            for i in self.interval[self.oppo_to_move as usize][iof as usize][ion as usize].begin()..self.interval[self.oppo_to_move as usize][iof as usize][ion as usize].end() {
                self.interval[self.oppo_to_move as usize][iof as usize][i as usize] = self.interval[self.oppo_to_move as usize][iof as usize][ion as usize];
            }
        }
    }

    pub fn undo_move(&mut self) {
        self.switch_side_to_move();

        self.piece_cnt -= 1;
        let last_move: Move = self.piece_list[self.piece_cnt];
        self._board[last_move as usize] = Color::Hide;
        self.restore_see(last_move);
        self.restore_vector_board(last_move);

        if self.updated_interval[self.piece_cnt + 1] {
            self.restore_interval(last_move);
        }

        for i in 0..self.mat_inc.len() {
            for mat in Material::iter() {
                self.mat_inc[i][mat] = 0;
            }
        }

        for i in 0..self.f3_formed_cnt.len() {
            self.f3_formed_cnt[i] = 0;
        }

        self.key ^= self.zobrists[self.side_to_move as usize][last_move as usize];
    }

    pub fn query(&self, color: Color, material: Material) -> i32 {
        return self.mat[self.piece_cnt][color as usize][material];
    }

    fn query_inc(&self, color: Color, material: Material) -> i32 {
        return self.mat_inc[color as usize][material];
    }

    fn index_of(&self, m: Move, d: Direction) -> i32 {
        self.index_of_table[m as usize][d]
    }

    fn index_on(&self, m: Move, d: Direction) -> i32 {
        self.index_on_table[m as usize][d]
    }

    fn query_see(&self, color: Color, vind: i32, sind: i32, mask: u32) -> bool {
        (self.see[if color == Color::Black {0} else {1}][vind as usize][sind as usize] & mask) != 0
    }

    fn see_of(&self, m: Move) -> Score {
        let mut ret: Score = 0;
        
        for d in Direction::iter() {
            ret += self.see_table[get_bits(self.see[self.side_to_move as usize][self.index_of(m, d) as usize][self.index_on(m, d) as usize], 3, 17) as usize] - self.see_table[get_bits(self.see[self.oppo_to_move as usize][self.index_of(m, d) as usize][self.index_on(m, d) as usize], 17, 31) as usize];
        }
    
        return ret;
    }

    fn query_us_inc(&self, color: Color, m: Move, material: Material) -> i32 {
        let mut ret: i32 = 0;

        for d in Direction::iter() {
            ret += if self.query_see(color, self.index_of(m, d), self.index_on(m, d), (1 as u32) << material as u32) {1} else {0};
        }
    
        return ret;
    }

    fn query_vcf(&self, color: Color, m: Move) -> i32 {
        let mut ret: i32 = 0;
    
        for d in Direction::iter() {
            ret += if self.query_see(color, self.index_of(m, d), self.index_on(m, d), (1 as u32) << 31) {1} else {0};
        }
    
        return ret;
    }

    fn query_opp_dec(&self, color: Color, m: Move, material: Material) -> i32 {
        let mut ret: i32 = 0;

        for d in Direction::iter() {
            ret += if self.query_see(if color == Color::White {Color::Black} else {Color::White}, self.index_of(m, d), self.index_on(m, d), (1 as u32) << (material as i32 + 21)) {1} else {0};
        }

        return ret;
    }

    pub fn check_wld_already(&self) -> Color {
        if self.query(Color::Black, Material::C5) > 0 {
            return Color::Black;
        }

        if self.query(Color::White, Material::C5) > 0 {
            return Color::White;
        }

        if self.piece_cnt as i32 >= MOVE_SIZE {
            return Color::Hide;
        }

        if self.query(Color::Black, Material::C6) > 0 || self.query_inc(Color::Black, Material::F4) + self.query_inc(Color::Black, Material::B4) >= 2 || self.f3_formed_cnt[0 as usize] >= 2 {
            return Color::White;
        }

        return Color::None;
    }

    pub fn check_wld(&self, offset: &mut Score) -> Color {
        let color = self.check_wld_already();
        
        if color != Color::None {
            *offset = 0;
            return color;
        }
    
        // side to move has F4 or B4
        if self.query(self.side_to_move, Material::F4) > 0 || self.query(self.side_to_move, Material::B4) > 0 {
            *offset = 1;
            return self.side_to_move;
        }
    
        // oppo to move has F4 or several B4
        if self.query(self.oppo_to_move, Material::F4) > 0 || self.query(self.oppo_to_move, Material::B4) >= 2 {
            *offset = 2;
            return self.oppo_to_move;
        }
    
        // side to move has F3 and both do not have B4
        if self.query(self.side_to_move, Material::F3) > 0 && self.query(self.oppo_to_move, Material::B4) == 0 {
            *offset = 3;
            return self.side_to_move;
        }
    
        return Color::None;

    }

    pub fn last_move(&self, n: usize) -> Move {
        self.piece_list[self.piece_cnt - n]
    }

    pub fn evaluate(&self) -> Score {
        if self.piece_cnt > 0 {
            (self.score[self.piece_cnt][self.side_to_move as usize] - self.score[self.piece_cnt][self.oppo_to_move as usize]
                + self.score[self.piece_cnt - 1][self.side_to_move as usize] - self.score[self.piece_cnt - 1][self.oppo_to_move as usize]) / 2
        } else {
            0
        }
    }
    
    pub fn is_quiet(&self, m: Option<Move>) -> bool {
        match m {
            Some(m) => {
                if self.query_us_inc(self.side_to_move, m, Material::B4) + self.query_us_inc(self.side_to_move, m, Material::F3) != 0{
                    false
                } else {
                    true
                }
            },
            None => {
                if self.query(Color::Black, Material::B4) + self.query(Color::Black, Material::F3)
                    + self.query(Color::White, Material::B4) + self.query(Color::White, Material::F3) != 0 {
                    false
                } else {
                    true
                }
            }
        }
    }

    fn update_movelist(&mut self, m: Move) {
        self.m_list_stack[self.piece_cnt] = self.m_list_stack[self.piece_cnt - 1];
        self.m_list_stack[self.piece_cnt].remove(m);

        for i in N2.iter() {
            if self.is_empty(m + *i) {
                self.m_list_stack[self.piece_cnt].insert(m + *i);
            }
        }
    }

    pub fn is_empty(&self, m: Move) -> bool {
        self._board[m as usize] == Color::Hide
    }

    pub unsafe fn is_foul(&mut self, m: Move) -> bool {
        let mut ret: bool = false;
        let need_to_switch = self.side_to_move != Color::Black;

        if need_to_switch {
            self.switch_side_to_move();
        }
        
        self.do_move(m);
        ret = self.query(Color::Black, Material::C6) > 0 || self.query_inc(Color::Black, Material::F4) + self.query_inc(Color::Black, Material::B4) >= 2 || self.f3_formed_cnt[Color::Black as usize] >= 2;
        self.undo_move();

        if need_to_switch {
            self.switch_side_to_move();
        }

        return ret;
    }

    pub fn defend_b4(&self) -> Move {
        self.b4d_stack[self.piece_cnt]
    }
}

use std::cmp;

pub fn _rank_of(m: Move) -> i32 {
    (m as i32 >> BOARD_SIDE_BIT) - BOARD_BOUNDARY
}

pub fn _file_of(m: Move) -> i32 {
    (m as i32 & ((1 << BOARD_SIDE_BIT) - 1)) - BOARD_BOUNDARY
}

fn _mdiag_of(m: Move) -> i32 {
    let s = _start_of(m, Direction::DMDiag);
    let r = _rank_of(s);

    if r == 0 {
        _file_of(s)
    } else {
        r + BOARD_SIDE - 1
    }
}

fn _adiag_of(m: Move) -> i32 {
    let s = _start_of(m, Direction::DADiag);
    let r = _rank_of(s);

    if r == 0 {
        _file_of(s)
    } else {
        r + BOARD_SIDE - 1
    }
}

fn _mdiag_index_on(m: Move) -> i32 {
    let mut i = 1;

    while is_ok_move(m - D[Direction::DMDiag as usize] * i) {
        i += 1;
    }

    (i - 1) as i32
}

fn _adiag_index_on(m: Move) -> i32 {
    let mut i = 1;

    while is_ok_move(m - D[Direction::DADiag as usize] * i) {
        i += 1;
    }

    (i - 1) as i32
}

fn _start_of(m: Move, d: Direction) -> Move {
    let mut i = 1;
    while is_ok_move(m - D[d as usize] * i) {
        i += 1;
    }

    m - D[d as usize] * (i - 1)
}

fn _end_of(m: Move, d: Direction) -> Move {
    let mut i = 1;
    while is_ok_move(m + D[d as usize] * i) {
        i += 1;
    }

    m + D[d as usize] * (i - 1)
}

pub fn _make_move(r: &i32, f: &i32) -> Move {
    (((r + BOARD_BOUNDARY) << BOARD_SIDE_BIT) + f + BOARD_BOUNDARY) as Move
}

fn _distance_between(m1: Move, m2: Move) -> i32 {
    cmp::max((_rank_of(m1) - _rank_of(m2)).abs(), (_file_of(m1) - _file_of(m2)).abs())
}

pub struct PRNG {
    s: u64
}

impl PRNG {
    fn new(seed: u64) -> PRNG {
        Self {
            s: seed
        }
    }

    fn rand64(&mut self) -> ZobristKey {
        self.s ^= self.s >> 12;
        self.s ^= self.s << 25;
        self.s ^= self.s >> 27;
        
        self.s * (2685821657736338717 as u64)
    }
}