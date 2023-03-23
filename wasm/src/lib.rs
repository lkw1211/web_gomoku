use wasm_bindgen::prelude::*;
use std::cmp;

// mod pool;

static mut FUTILITY_MOVE_COUNT: [[i32; DEPTH_NUM as usize]; 2] = [[0; DEPTH_NUM as usize]; 2];
static mut REDUCTION: [[[Depth; MOVE_SIZE as usize]; DEPTH_NUM as usize]; 2] = [[[0; MOVE_SIZE as usize]; DEPTH_NUM as usize]; 2];

mod types;
use types::*;

#[wasm_bindgen(start)]
unsafe fn start() {
    FUTILITY_MOVE_COUNT[0][0] = -2;
    FUTILITY_MOVE_COUNT[1][0] = -12;

    for i in 1..DEPTH_NUM as usize {
        FUTILITY_MOVE_COUNT[0][i] = cmp::min(((5 as f64) * (i as f64).sqrt() - (2 as f64)).round() as i32, 8);
        FUTILITY_MOVE_COUNT[1][i] = cmp::min(((15 as f64) * (i as f64).sqrt() - (12 as f64)).round() as i32, 32);
        for mc in 1..MOVE_SIZE as usize {
            let r = (i as f64).ln() * (mc as f64).ln() / 2.0;
            REDUCTION[0][i][mc] = r.round() as Depth;
            REDUCTION[1][i][mc] = cmp::max(REDUCTION[0][i][mc] - 1, 0 as Depth);
        }
    }
    BOARD = Some(Board::new());
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    unsafe fn log(s: &str);
    #[no_mangle]
    #[used]
    static performance:web_sys::Performance;
}

static mut PERF_START: f64 = 0.0;
static mut TIME_LIMIT: i32 = 30;

#[wasm_bindgen]
pub unsafe fn think_and_move(ms: JsValue, tl: i32) -> Result<Move, JsValue> {
    PERF_START = performance.now();
    TIME_LIMIT = tl;
    let moves: Vec<Move> = serde_wasm_bindgen::from_value(ms)?;

    reset_search();
    BOARD.as_mut().unwrap().reset();
    
    for m in moves.iter() {
        BOARD.as_mut().unwrap().do_move(*m);
    }

    let mut skip_search: bool = false;
    let mut rem: RootExtMove = RootExtMove::new();
    {
        let mut mg: MoveGen = MoveGen::new(());
        let mut em: ExtMove;
        let mut offset: Score = -1;

        // unsafe {
        //     TT.new_search();
        // }

        if !skip_search && moves.len() == 0 {
            rem.score = 0;
            rem.depth = 1 as Depth;
            update_pv2(Some(&mut rem.pv), make_move(7, 7));
            skip_search = true;
        }

        if !skip_search && BOARD.as_ref().unwrap().check_wld(&mut offset) != Color::None {
            em = mg.generate(GenType::WLD);
            rem.score = em.s;
            rem.depth = offset as Depth;
            update_pv2(Some(&mut rem.pv), em.m);
            skip_search = true;
        }

        if !skip_search {
            em = mg.generate(GenType::MAIN);
            if mg.size() == 1 {
                rem.score = 0;
                rem.depth = 1 as Depth;
                update_pv2(Some(&mut rem.pv), em.m);
                skip_search = true;
            }
        }
    }

    if skip_search {
        ROOT_BESTS.push(rem);
    } else {
        search();
    }

    let best_move = ROOT_BESTS.last().unwrap_unchecked().pv[0];

    Ok(best_move)
}

static mut itd: i32 = 0;

unsafe fn reset_alpha_beta() {
    PLY = 0;
    if SEARCH_STACK[0].pv.is_some() {
        reset_pv(SEARCH_STACK[0].pv.as_mut().unwrap_unchecked());
    } else {
        SEARCH_STACK[0].pv = Some([0; STACK_SIZE as usize]);
    }
}

unsafe fn reset_search() {
    PLY_MAX  = 0;
    ROOT_BESTS.clear();
    reset_alpha_beta();
}

pub unsafe fn search() -> Move {
    let mut rem: RootExtMove = RootExtMove::new();
    let mut valid_result: bool = true;
    let mut break_search: bool = false;

    for it_depth in DEPTH_ITERATIVE_MIN..DEPTH_ITERATIVE_MAX+1 {
        itd = it_depth as i32;
        reset_alpha_beta();
        let score = alpha_beta(NodeType::PV, it_depth, -INFINITY_SCORE, INFINITY_SCORE, false);
        rem.set(score, it_depth, SEARCH_STACK[0].pv.clone());

        if terminated() && is_empty(SEARCH_STACK[0].pv.as_ref()) {
            valid_result = false;
        }

        if terminated() || (rem.score.abs() > WIN_SCORE_THRESHOLD && it_depth as Score >= WIN_SCORE - rem.score.abs()) {
            break_search = true;
        }

        if valid_result {
            ROOT_BESTS.push(rem);
        }

        if break_search {
            break;
        }
    }
    ROOT_BESTS.last().unwrap_unchecked().pv[0]
}

#[wasm_bindgen]
pub fn make_move(r: i32, f: i32) -> Move {
    _make_move(&r, &f)
}

#[wasm_bindgen]
pub fn rank_of(m: Move) -> i32 {
    _rank_of(m)
}

#[wasm_bindgen]
pub fn file_of(m: Move) -> i32 {
    _file_of(m)
}

#[wasm_bindgen]
pub unsafe fn foul_moves(ms: JsValue) -> Result<JsValue, JsValue> {
    let moves: Vec<Move> = serde_wasm_bindgen::from_value(ms)?;
    let mut foul_moves: Vec<Move> = Vec::new();

    reset_search();
    BOARD.as_mut().unwrap().reset();
    
    for m in moves.iter() {
        BOARD.as_mut().unwrap().do_move(*m);
    }

    for m in 0..MOVE_CAPACITY as Move {
        if BOARD.as_ref().unwrap().is_empty(m) {
            if BOARD.as_mut().unwrap().is_foul(m) {
                foul_moves.push(m);
            }
        }
    }

    return Ok(serde_wasm_bindgen::to_value(&foul_moves)?);
}

#[wasm_bindgen]
pub unsafe fn check_wld_already(ms: JsValue) -> Result<i32, JsValue> {
    let moves: Vec<Move> = serde_wasm_bindgen::from_value(ms)?;
    let mut foul_moves: Vec<Move> = Vec::new();

    reset_search();
    BOARD.as_mut().unwrap().reset();
    
    for m in moves.iter() {
        BOARD.as_mut().unwrap().do_move(*m);
    }

    return Ok(match BOARD.as_ref().unwrap().check_wld_already() {
        Color::Black => 1,
        Color::White => 2,
        _ => 0
    });
}

unsafe fn terminated() -> bool {
    let elasped = performance.now() - PERF_START;
    let limit = (TIME_LIMIT * 1000 - 50) as f64;

    return elasped >= limit;
}

unsafe fn alpha_beta(nt: NodeType, depth: Depth, mut alpha: Score, mut beta: Score, cautious: bool) -> Score {
    if terminated() {
        return if (PLY as u32 & (1 as u32)) != 0 {WIN_SCORE} else {-WIN_SCORE};
    }

    if PLY_MAX < PLY {
        PLY_MAX = PLY;
    }

    let pv_node: bool = nt == NodeType::PV;
    let root_node: bool = PLY == 0;
    let mut offset: Score = -1;
    let wld: Color = BOARD.as_ref().unwrap().check_wld(&mut offset);
    if offset != -1 {
        if wld == BOARD.as_ref().unwrap().side_to_move {
            return WIN_SCORE - PLY as Score - offset;
        } else if wld == BOARD.as_ref().unwrap().oppo_to_move {
            return -WIN_SCORE + PLY as Score + offset;
        } else if wld == Color::Hide {
            return 0;
        }
    }

    if !root_node {
        alpha = cmp::max(-WIN_SCORE + PLY as Score, alpha);
        beta = cmp::min(WIN_SCORE - PLY as Score - 1, beta);
        if alpha >= beta {
            return alpha;
        }
    }

    let mut static_score: Score = BOARD.as_ref().unwrap().evaluate();
    let mut score: Score = static_score;

    if depth <= 0 || PLY >= DEPTH_MAX {
        if static_score < beta && BOARD.as_ref().unwrap().query(BOARD.as_ref().unwrap().side_to_move, types::Material::B3) > 0 {
            score = vcf(nt, cmp::min(PLY * 2, DEPTH_MAX), true);

            if score > WIN_SCORE_THRESHOLD {
                return score;
            }
        }
        return static_score;
    }

    let mut best_move: Move = MOVE_NONE;
    let mut tt_move: Move = MOVE_NONE;
    let mut best_score: Score = -INFINITY_SCORE;
    let mut tt_score: Score = SCORE_NONE;
    let key: ZobristKey = BOARD.as_ref().unwrap().key;
    let mut tt_entry: &mut TTEntry = &mut Default::default();
    let mut tt_hit: bool = false;
    let mut move_count: i32 = 0;
    let defend_b4: bool = BOARD.as_ref().unwrap().query(BOARD.as_ref().unwrap().oppo_to_move, types::Material::B4) > 0;
    let quiet_node: bool = BOARD.as_ref().unwrap().is_quiet(None);
    let mut extend: bool = false;
    let mut do_full_depth_search: bool = false;
    let mut child_pv: Pv = [MOVE_NONE; STACK_SIZE as usize]; 
    let mut new_depth: Depth;
    reset_pv(&mut child_pv);

    SEARCH_STACK[PLY as usize + 2].killers[0] = MOVE_NONE;
    SEARCH_STACK[PLY as usize + 2].killers[1] = MOVE_NONE;

    // tt_entry = TT.probe(&key, &mut tt_hit);

    if root_node && ROOT_BESTS.len() != 0 {
        tt_move = ROOT_BESTS.last().unwrap_unchecked().pv[0];
        tt_score = ROOT_BESTS.last().unwrap_unchecked().score;
    } else if tt_hit {
        tt_move = tt_entry.move16();
        tt_score = score_from_tt(tt_entry.score());
    } else {
        tt_move = MOVE_NONE;
        score = SCORE_NONE;
    };

    if !pv_node && tt_hit && tt_score > WIN_SCORE_THRESHOLD && (tt_entry.bound() as u8 & Bound::BoundLower as u8) != 0 {
        // Update history
        if tt_score >= beta {
            update_history(tt_move);
        }
        return tt_score;
    }

    if !pv_node && tt_hit && tt_entry.depth() >= depth && tt_score != SCORE_NONE && if tt_score >= beta { (tt_entry.bound() as u8 & Bound::BoundLower as u8) != 0 } else { (tt_entry.bound() as u8 & Bound::BoundUpper as u8) != 0 }  {
        if tt_score >= beta {
            update_history(tt_move);
        }
        return tt_score;
    }

    'outer: {
        if defend_b4 {
            break 'outer;
        }

        if tt_hit && (tt_entry.bound() as u8 & (if tt_score > static_score {Bound::BoundLower} else {Bound::BoundUpper}) as u8) != 0 {
            static_score = tt_score;
        }

        if !root_node && depth < 5 && static_score + futility_margin(depth) <= alpha {
            return alpha_beta(nt, 0, alpha, beta, cautious);
        }

        if !root_node && depth < 7 && static_score - futility_margin(depth) >= beta && static_score < WIN_SCORE_THRESHOLD { // Do not return not verified wins
            return static_score;
        }

        if depth >= 7 && tt_move == MOVE_NONE {
            alpha_beta(nt, depth / 2, alpha, beta, cautious);
            // tt_entry = TT.probe(&key, &mut tt_hit);
            tt_move  = if tt_hit { tt_entry.move16() } else { MOVE_NONE };
            tt_score = if tt_hit { score_from_tt(tt_entry.score()) } else { SCORE_NONE };
        }
    }

    let cm: Move = if BOARD.as_ref().unwrap().piece_cnt >= 1 { COUNTER_MOVES[BOARD.as_ref().unwrap().last_move(1) as usize] } else { MOVE_NONE };
    let karr1: Move = SEARCH_STACK[PLY as usize].killers[0];
    let karr2: Move = SEARCH_STACK[PLY as usize].killers[1];

    let mut mg: MoveGen = MoveGen::new((Stage::MainTT, tt_move, false, PLY, karr1, karr2, cm));
    let mut em: ExtMove;

    loop {
        em = mg.next_move();
        if em.m == MOVE_NONE {
            break;
        }

        move_count += 1;
        if !cautious && PLY >= 2 {
            if move_count > FUTILITY_MOVE_COUNT[if quiet_node {1} else {0}][depth as usize] {
                break;
            }
        } else {
            if move_count > FUTILITY_MOVE_COUNT[if quiet_node {1} else {0}][depth as usize] && em.s < SEE_THRESHOLD {
                break;
            }
        }

        // prepetch

        new_depth = depth - 1;

        if defend_b4 {
            extend = true;
        }

        if extend {
            new_depth = cmp::min(new_depth + 1, DEPTH_MAX);
        } else {
            new_depth = cmp::min(new_depth, DEPTH_MAX);
        }

        PLY += 1;
        SEARCH_STACK[PLY as usize].pv = Some(child_pv);
        BOARD.as_mut().unwrap().do_move(em.m);

        if depth >= 3 && move_count > 1 {
            let r: Depth = REDUCTION[if pv_node {1} else {0}][depth as usize][move_count as usize];
            let d: Depth = if new_depth - r >= 1 && r >= 0 {new_depth - r} else if new_depth - r < 1 {1} else {new_depth};
            score = -alpha_beta(NodeType::NonPV, d, -alpha - 1, -alpha, cautious);
            do_full_depth_search = score > alpha && d != new_depth;
        } else {
            do_full_depth_search = !pv_node || move_count > 1;
        }

        if do_full_depth_search {
            score = -alpha_beta(NodeType::NonPV, new_depth, -alpha - 1, -alpha, cautious);
        }

        if pv_node && (move_count == 1 || (score > alpha && (root_node || score < beta))) {
            score = -alpha_beta(NodeType::PV, new_depth, -beta, -alpha, cautious);
        }

        if pv_node && PLY >= 2 && !cautious && score > WIN_SCORE_THRESHOLD {
            let s: Score = -alpha_beta(NodeType::PV, new_depth, -WIN_SCORE_THRESHOLD, -WIN_SCORE_THRESHOLD + 1, true);

            if s < WIN_SCORE_THRESHOLD {
                score = -alpha_beta(NodeType::PV, new_depth, -beta, -alpha, true);
            }
        }

        BOARD.as_mut().unwrap().undo_move();
        PLY -= 1;
        if terminated()  {
            return best_score;
        }

        if score > best_score {
            best_score = score;

            if score > alpha {
                best_move = em.m;

                if pv_node && (!root_node || score < beta) {
                    let cpv = SEARCH_STACK[PLY as usize + 1].pv;
                    update_pv3(SEARCH_STACK[PLY as usize].pv.as_mut(), em.m, cpv);
                }

                if pv_node && score < beta {
                    alpha = score;
                } else {
                    break;
                }
            }
        }
    }

    if best_move != MOVE_NONE {
        update_history(best_move);
    }

    // let bound: Bound = if best_score >= beta {Bound::BoundLower} else if pv_node && best_move != MOVE_NONE {Bound::BoundExact} else {Bound::BoundUpper};
    // tt_entry.save(key, best_move, score_to_tt(best_score), bound, false, depth);

    best_score
}

unsafe fn update_history(tt_move: Move) {
    SEARCH_STACK[PLY as usize].update_killers(tt_move);
    if BOARD.as_ref().unwrap().piece_cnt >= 1 {
        COUNTER_MOVES[BOARD.as_mut().unwrap().last_move(1) as usize] = tt_move;
    }
}

unsafe fn futility_margin(d: Depth) -> Score {
    45 as Score * d as Score 
}

unsafe fn score_to_tt(s: Score) -> Score {
    return if s > WIN_SCORE_THRESHOLD {s + PLY as Score} else if s < -WIN_SCORE_THRESHOLD {s - PLY as Score} else {s};
}

unsafe fn score_from_tt(s: Score) -> Score {
    return if s > WIN_SCORE_THRESHOLD {s - PLY as Score} else if s < -WIN_SCORE_THRESHOLD {s + PLY as Score} else {s};
}

unsafe fn vcf(nt: NodeType, depth: Depth, root_node: bool) -> Score {
    let pv_node: bool = nt == NodeType::PV;
    let mut offset: Score = -1;

    if !root_node {
        PLY_MAX = cmp::max(PLY_MAX, PLY);

        let color = BOARD.as_ref().unwrap().check_wld(&mut offset);
        if offset != -1 {
            if color == BOARD.as_ref().unwrap().side_to_move {
                return WIN_SCORE - PLY as Score - offset;
            } else if color == BOARD.as_ref().unwrap().oppo_to_move {
                return -WIN_SCORE + PLY as Score + offset;
            } else if color == Color::Hide {
                return 0; // draw
            }
        }
    }

    if depth <= 0 || PLY >= DEPTH_MAX {
        return 0;
    }

    let mut b4d: Move;
    let mut score: Score;
    let mut best_score: Score;
    let mut child_pv: Pv = [MOVE_NONE; STACK_SIZE as usize];
    let mut move_count: i32;
    best_score = -INFINITY_SCORE;
    move_count = 0;
    reset_pv(&mut child_pv);

    let mut mg: MoveGen = MoveGen::new((Stage::VcfTT, MOVE_NONE, root_node));
    let mut em: ExtMove;

    loop {
        em = mg.next_move();
        if em.m == MOVE_NONE {
            break;
        }
        move_count += 1;
        if !root_node && move_count > 2 {
            break;
        }
        PLY += 2;
        SEARCH_STACK[PLY as usize].pv = Some(child_pv);
        BOARD.as_mut().unwrap().do_move(em.m);
        let color: Color = BOARD.as_ref().unwrap().check_wld(&mut offset);
        if color != Color::None {
            BOARD.as_mut().unwrap().undo_move();
            PLY -= 2;
            if color == BOARD.as_ref().unwrap().side_to_move {
                best_score = WIN_SCORE - PLY as Score - offset;
                if pv_node {
                    update_pv2(SEARCH_STACK[PLY as usize].pv.as_mut(), em.m);
                }
                break;
            } else {
                move_count -= 1;
                continue;
            }
        }
        b4d = BOARD.as_ref().unwrap().defend_b4();

        BOARD.as_mut().unwrap().do_move(b4d);

        score = vcf(nt, depth - 2, false);
        
        BOARD.as_mut().unwrap().undo_move();
        BOARD.as_mut().unwrap().undo_move();

        PLY -= 2;
        if score > WIN_SCORE_THRESHOLD {
            best_score = score;
            if pv_node {
                let cpv = SEARCH_STACK[PLY as usize + 2].pv;
                update_pv4(SEARCH_STACK[PLY as usize].pv.as_mut(), em.m, cpv, b4d);
            }
            break;
        }
    }

    best_score
}


unsafe fn update_pv2(pv: Option<&mut Pv>, m0: Move) {
    let mut pv_ptr = pv.unwrap_unchecked();
    pv_ptr[0] = m0;
    pv_ptr[1] = MOVE_NONE;
}

unsafe fn update_pv3(pv: Option<&mut Pv>, m0: Move, child_pv: Option<Pv>) {
    let mut i: usize = 0;
    let mut pv_ptr = pv.unwrap_unchecked();
    let child_pv_ptr = child_pv.unwrap_unchecked();
    pv_ptr[0] = m0;

    while child_pv_ptr[i] != MOVE_NONE {
        i += 1;
        pv_ptr[i] = child_pv_ptr[i - 1];
    }

    pv_ptr[i + 1] = MOVE_NONE;
}


unsafe fn update_pv4(pv: Option<&mut Pv>, m0: Move, child_pv: Option<Pv>, m1: Move) {
    let mut i: usize = 0;
    let mut pv_ptr = pv.unwrap_unchecked();
    let child_pv_ptr = child_pv.unwrap_unchecked();
    pv_ptr[0] = m0;
    pv_ptr[1] = m1;

    while child_pv_ptr[i] != MOVE_NONE {
        i += 1;
        pv_ptr[i + 1] = child_pv_ptr[i - 1];
    }

    pv_ptr[i + 2] = MOVE_NONE;
}