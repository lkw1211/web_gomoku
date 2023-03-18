<template>
	<div class="gomoku">
		<div class="title">Gomoku AI</div>
		<div id="board">
			<div id="boundary">
				<div class="point" v-for="(item, index) in state.point" :key="index" :style="item"></div>
				<div class="row" v-for="i in state.board_rows" :key="i">
					<div class="square border-black" v-for="j in state.board_rows" :key="i*100+j">
					</div>
				</div>
			</div>
			<div id="play">
				<div class="row" v-for="i in state.play_rows" :key="i">
					<div v-for="j in state.play_rows" :key="i*100+j" class="square" :style="'border: none; padding: 1px'">
						<div :class="`target ${state.target[i-1][j-1]}`" @click="targetClick(i-1, j-1)"></div>
						<div :class="`${state.color[i-1][j-1]} stone center`"></div>
					</div>
				</div>
			</div>
		</div>
		<el-row justify="center">
			<div class="deck">
				<el-row>
					<el-col :span="8">
						<el-button class="undo-button" round @click="undoClick()">UNDO</el-button>
					</el-col>
					<el-col :span="8">
						<el-button class="put-button" round @click="putClick()">PUT</el-button>
					</el-col>
					<el-col :span="8">
						<el-button class="reset-button" round @click="resetClick()">RESET</el-button>
					</el-col>
				</el-row>
			</div>
		</el-row>
	</div>
	<el-dialog
		v-model="state.colorSelectVisible"
		title="Select Place"
		class="color-select-dialog"
		:close-on-click-modal="false"
		:close-on-press-escape="false"
	>
		<el-row>
			<el-col :span="12">
				<div class="color-select square-2x center" @click="selectColor('black')">
					<div class="stone-2x center black"></div>
				</div>
			</el-col>
			<el-col :span="12">
				<div class="color-select square-2x center" @click="selectColor('white')">
					<div class="stone-2x center white"></div>
				</div>
			</el-col>		
		</el-row>
		<br>
		<br>
	</el-dialog>
</template>

<script type="module">
import router from '@/router';
import { reactive, computed, onMounted } from 'vue'

export default {
  name: 'HomeView',
  components: {
  },
  setup(props) {
	let color = [];
	let target = [];
	let current_target = [];
	let play_rows = 15;
	let board_rows = 14;

	for (let i = 0; i < play_rows; i++) {
		let colorRow = [];
		let targetRow = [];

		for (let j = 0; j < play_rows; j++) {
			colorRow.push('hide');
			targetRow.push('hide');
		}
		color.push(colorRow);
		target.push(targetRow);
	}

	const initialState = {
		colorSelectVisible: true,
		board_rows,
		play_rows,
		point: [
			{
				top: `50%`,
				left: `50%`,
			},
			{
				top: `${300/14}%`,
				left: `${300/14}%`,
			},
			{
				top: `${1100/14}%`,
				left: `${1100/14}%`,
			},
			{
				top: `${300/14}%`,
				left: `${1100/14}%`,
			},
			{
				top: `${1100/14}%`,
				left: `${300/14}%`,
			}
		],
		color,
		position: [],
		current_target,
		target,
		player: 'black',
		turn: 'black',
		winState: 0,
		blackMinMaxPos: [-1, -1, -1, -1],
		time: 20,
	};

	const state = reactive(JSON.parse(JSON.stringify(initialState)));
	const BOARD_BOUNDARY = 4;
	const BOARD_SIDE_BIT = 5;

	function make_move(r, f) {
		return ((r + BOARD_BOUNDARY) << BOARD_SIDE_BIT) + f + BOARD_BOUNDARY
	}

	function rank_of(m) {
		return (m >> BOARD_SIDE_BIT) - BOARD_BOUNDARY
	}

	function file_of(m) {
		return (m & ((1 << BOARD_SIDE_BIT) - 1)) - BOARD_BOUNDARY
	}

	function resetClick() {
		Object.assign(state, JSON.parse(JSON.stringify(initialState)));
	}

	function undoClick() {
		const undoMoves = state.position.slice(-2);

		undoMoves.forEach((move) => {
			state.color[rank_of(move)][file_of(move)] = 'hide';
		});
		const oldLastMove = undoMoves.slice(-1)[0];
		state.target[rank_of(oldLastMove)][file_of(oldLastMove)] = 'hide';

		state.position = state.position.slice(0, -2);

		if (state.position.length > 0) {
			const lastMove = state.position.slice(-1)[0];

			state.target[rank_of(lastMove)][file_of(lastMove)] = 'last';
		}

		if (state.turn == 'black') {
			// 금수 초기화
			for (let i = 0; i < state.color.length; i++) {
				for (let j = 0; j < state.color.length; j++) {
					if (state.color[i][j] == 'ban') {
						state.color[i][j] = 'hide';
					}
				}
			}

			// 금수 탐색
			let banList = [];
			for (let i = 0; i < state.color.length; i++) {
				for (let j = 0; j < state.color.length; j++) {
					if (isBanSearchTarget(i, j) && state.color[i][j] == 'hide') {
						if (checkBan(i, j)) {
							banList.push([i, j]);
						}
					}
				}
			}

			// 금수 표시
			banList.forEach(banPosition => {
				state.color[banPosition[0]][banPosition[1]] = 'ban';
			});
		}
	};

	function selectColor(color) {
		state.player = color;

		if (color == 'white') {
			targetClick((state.play_rows - 1)/2, (state.play_rows - 1)/2, true);
			putClick(true);
		}
		state.colorSelectVisible = false;
	}

	function _checkWin(color) {
		for (let i = 0; i < state.play_rows - 4; i++) {
			for (let j = 0; j < state.play_rows; j++) {
				if (state.color[i][j] == color && state.color[i+1][j] == color && state.color[i+2][j] == color && state.color[i+3][j] == color && state.color[i+4][j] == color) {
					return true;
				}
			}
		}

		for (let j = 0; j < state.play_rows - 4; j++) {
			for (let i = 0; i < state.play_rows; i++) {
				if (state.color[i][j] == color && state.color[i][j+1] == color && state.color[i][j+2] == color && state.color[i][j+3] == color && state.color[i][j+4] == color) {
					return true;
				}
			}
		}

		for (let i = 0; i < state.play_rows - 4; i++) {
			for (let j = 0; j < state.play_rows - 4; j++) {
				if (state.color[i][j] == color && state.color[i+1][j+1] == color && state.color[i+2][j+2] == color && state.color[i+3][j+3] == color && state.color[i+4][j+4] == color) {
					return true;
				}
			}
		}

		for (let i = 0; i < state.play_rows - 4; i++) {
			for (let j = 0; j < state.play_rows - 4; j++) {
				if (state.color[state.play_rows - i - 1][j] == color && state.color[state.play_rows - i - 2][j+1] == color && state.color[state.play_rows - i - 3][j+2] == color && state.color[state.play_rows - i - 4][j+3] == color && state.color[state.play_rows - i - 5][j+4] == color) {
					return true;
				}
			}
		}

		return false;
	};

	function checkWin() {
		if (_checkWin('black')) {
			return 1;
		}

		if (_checkWin('white')) {
			return 2;
		}

		return 0;
	}

	function targetClick(i, j, byAI=false) {
		if (state.player == 'none') {
			state.colorSelectVisible = true;
			return;
		}
		
		if (state.color[i][j] != 'hide' || state.winState != 0 || (state.turn != state.player && !byAI)) return;

		for (let a = 0; a < state.target.length; a++) {
			for (let b = 0; b < state.target.length; b++) {
				if (state.target[a][b] == 'select' && (a != i || b != j)) {
					state.target[a][b] = 'hide';
				}
			}
		}

		state.target[i][j] = 'select';
		state.current_target = [i, j];
	}

	function moveCheck(i, j, direction, moveCount) {
		let moveI = i + direction[0] * moveCount;
		let moveJ = j + direction[1] * moveCount;

		if (moveI < 0 || moveI >= state.color.length) return false;
		if (moveJ < 0 || moveJ >= state.color.length) return false;

		return true;
	}

	function moveColor(i, j, direction, moveCount) {
		if (!moveCheck(i, j, direction, moveCount)) return 'error';
		let moveI = i + moveCount * direction[0];
		let moveJ = j + moveCount * direction[1];
		return state.color[moveI][moveJ];
	}

	// moveCounts 0번이 1칸쪽 1번이 2칸쪽
	function checkEndPoint1(i, j, direction, moveCounts) {
		if (moveColor(i, j, direction, moveCounts[0]) == 'black'
			|| (moveColor(i, j, direction, moveCounts[1]) == 'black' && moveColor(i, j, direction, moveCounts[0]) != 'hide')) {
			return false;
		}

		return true;
	}

	function checkEndPoint2(i, j, direction, moveCounts) {
		if (moveColor(i, j, direction, moveCounts[0]) == 'black'
			|| moveColor(i, j, direction, moveCounts[1]) == 'black') {
			return false;
		}

		return true;
	}

	// 3x3 금수
	function check3Ban(i, j, direction) {

		// XTBBXX
		if (moveColor(i, j, direction, -1) == 'hide'
			&& moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'black'
			&& moveColor(i, j, direction, 3) == 'hide'
			&& moveColor(i, j, direction, 4) == 'hide'
			&& checkEndPoint1(i, j, direction, [-2, 5])) return 1;

		//XBTBXX
		if (moveColor(i, j, direction, -2) == 'hide'
			&& moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'hide'
			&& moveColor(i, j, direction, 3) == 'hide'
			&& checkEndPoint1(i, j, direction, [-3, 4])) return 1;

		//XBBTXX
		if (moveColor(i, j, direction, -3) == 'hide'
			&& moveColor(i, j, direction, -2) == 'black'
			&& moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'hide'
			&& moveColor(i, j, direction, 2) == 'hide'
			&& checkEndPoint1(i, j, direction, [-4, 3])) return 1;

		// XXTBBX
		if (moveColor(i, j, direction, -2) == 'hide'
			&& moveColor(i, j, direction, -1) == 'hide'
			&& moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'black'
			&& moveColor(i, j, direction, 3) == 'hide'
			&& checkEndPoint1(i, j, direction, [4, -3])) return 1;

		// XXBTBX
		if (moveColor(i, j, direction, -3) == 'hide'
			&& moveColor(i, j, direction, -2) == 'hide'
			&& moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'hide'
			&& checkEndPoint1(i, j, direction, [3, -4])) return 1;

		// XXBBTX
		if (moveColor(i, j, direction, -4) == 'hide'
			&& moveColor(i, j, direction, -3) == 'hide'
			&& moveColor(i, j, direction, -2) == 'black'
			&& moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'hide'
			&& checkEndPoint1(i, j, direction, [2, -5])) return 1;
		
		// XTXBBX
		if (moveColor(i, j, direction, -1) == 'hide'
			&& moveColor(i, j, direction, 1) == 'hide'
			&& moveColor(i, j, direction, 2) == 'black'
			&& moveColor(i, j, direction, 3) == 'black'
			&& moveColor(i, j, direction, 4) == 'hide'
			&& checkEndPoint2(i, j, direction, [-2, 5])) return 1;
			
		// XBXTBX
		if (moveColor(i, j, direction, -3) == 'hide'
			&& moveColor(i, j, direction, -2) == 'black'
			&& moveColor(i, j, direction, -1) == 'hide'
			&& moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'hide'
			&& checkEndPoint2(i, j, direction, [-4, 3])) return 1;

		// XBXBTX
		if (moveColor(i, j, direction, -4) == 'hide'
			&& moveColor(i, j, direction, -3) == 'black'
			&& moveColor(i, j, direction, -2) == 'hide'
			&& moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'hide'
			&& checkEndPoint2(i, j, direction, [-5, 2])) return 1;

		// XTBXBX
		if (moveColor(i, j, direction, -1) == 'hide'
			&& moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'hide'
			&& moveColor(i, j, direction, 3) == 'black'
			&& moveColor(i, j, direction, 4) == 'hide'
			&& checkEndPoint2(i, j, direction, [-2, 5])) return 1;

		// XBTXBX
		if (moveColor(i, j, direction, -2) == 'hide'
			&& moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'hide'
			&& moveColor(i, j, direction, 2) == 'black'
			&& moveColor(i, j, direction, 3) == 'hide'
			&& checkEndPoint2(i, j, direction, [-3, 4])) return 1;

		// XBBXTX
		if (moveColor(i, j, direction, -4) == 'hide'
			&& moveColor(i, j, direction, -3) == 'black'
			&& moveColor(i, j, direction, -2) == 'black'
			&& moveColor(i, j, direction, -1) == 'hide'
			&& moveColor(i, j, direction, 1) == 'hide'
			&& checkEndPoint2(i, j, direction, [-5, 2])) return 1;

		return 0;
	};

	// 10 .... 23
	function checkEndPoint3(i, j, direction, moveCounts) {
		if (moveColor(i, j, direction, moveCounts[0]) == 'black'
			|| moveColor(i, j, direction, moveCounts[2]) == 'black'
			|| (
				(moveColor(i, j, direction, moveCounts[0]) == 'white' || moveColor(i, j, direction, moveCounts[0]) == 'error'
				|| (moveColor(i, j, direction, moveCounts[0]) == 'hide' && moveColor(i, j, direction, moveCounts[1]) == 'black')) 
				&&
				(moveColor(i, j, direction, moveCounts[2]) == 'white' || moveColor(i, j, direction, moveCounts[2]) == 'error'
				|| (moveColor(i, j, direction, moveCounts[2]) == 'hide' && moveColor(i, j, direction, moveCounts[3]) == 'black'))
				)
			) {
			return false;
		}

		return true;
	}

	// 0 .... 1
	function checkEndPoint4(i, j, direction, moveCounts) {
		if (moveColor(i, j, direction, moveCounts[0]) == 'black'
			|| moveColor(i, j, direction, moveCounts[1]) == 'black'
			) {
			return false;
		}

		return true;
	}

	// 4x4 금수
	function check4Ban(i, j, direction) {
		// TBBB
		if (moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'black'
			&& moveColor(i, j, direction, 3) == 'black'
			&& checkEndPoint3(i, j, direction, [-1, -2, 4, 5])) return 1;
		
		// BTBB
		if (moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'black'
			&& checkEndPoint3(i, j, direction, [-2, -3, 3, 4])) return 1;
		
		// BBTB
		if (moveColor(i, j, direction, -2) == 'black'
			&& moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'black'
			&& checkEndPoint3(i, j, direction, [-3, -4, 2, 3])) return 1;
		
		// BBBT
		if (moveColor(i, j, direction, -3) == 'black'
			&& moveColor(i, j, direction, -2) == 'black'
			&& moveColor(i, j, direction, -1) == 'black'
			&& checkEndPoint3(i, j, direction, [-4, -5, 1, 2])) return 1;
		
		// TBBXB
		if (moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'black'
			&& moveColor(i, j, direction, 3) == 'hide'
			&& moveColor(i, j, direction, 4) == 'black'
			&& checkEndPoint4(i, j, direction, [-1, 5])) return 1;

		// BTBXB
		if (moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'hide'
			&& moveColor(i, j, direction, 3) == 'black'
			&& checkEndPoint4(i, j, direction, [-2, 4])) return 1;

		// BBTXB
		if (moveColor(i, j, direction, -2) == 'black'
			&& moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'hide'
			&& moveColor(i, j, direction, 2) == 'black'
			&& checkEndPoint4(i, j, direction, [-3, 3])) return 1;

		// BBBXT
		if (moveColor(i, j, direction, -3) == 'black'
			&& moveColor(i, j, direction, -2) == 'black'
			&& moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'hide'
			&& checkEndPoint4(i, j, direction, [-4, 2])) return 1;
		
		// TBXBB
		if (moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'hide'
			&& moveColor(i, j, direction, 3) == 'black'
			&& moveColor(i, j, direction, 4) == 'black'
			&& checkEndPoint4(i, j, direction, [-1, 5])) return 1;

		// BTXBB
		if (moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'hide'
			&& moveColor(i, j, direction, 2) == 'black'
			&& moveColor(i, j, direction, 3) == 'black'
			&& checkEndPoint4(i, j, direction, [-2, 4])) return 1;

		// BBXTB
		if (moveColor(i, j, direction, -3) == 'black'
			&& moveColor(i, j, direction, -2) == 'black'
			&& moveColor(i, j, direction, -1) == 'hide'
			&& moveColor(i, j, direction, 1) == 'black'
			&& checkEndPoint4(i, j, direction, [-4, 2])) return 1;

		// BBXBT
		if (moveColor(i, j, direction, -4) == 'black'
			&& moveColor(i, j, direction, -3) == 'black'
			&& moveColor(i, j, direction, -2) == 'hide'
			&& moveColor(i, j, direction, -1) == 'black'
			&& checkEndPoint4(i, j, direction, [-5, 1])) return 1;

		return 0;
	};

	// 장목 금수
	function check6Ban(i, j, direction) {

		// BTBBBB
		if (moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'black'
			&& moveColor(i, j, direction, 3) == 'black'
			&& moveColor(i, j, direction, 4) == 'black') return 1;

		// BBTBBB
		if (moveColor(i, j, direction, -2) == 'black'
			&& moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'black'
			&& moveColor(i, j, direction, 3) == 'black') return 1;

		// BBBTBB
		if (moveColor(i, j, direction, -3) == 'black'
			&& moveColor(i, j, direction, -2) == 'black'
			&& moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'black'
			&& moveColor(i, j, direction, 2) == 'black') return 1;

		// BBBBTB
		if (moveColor(i, j, direction, -4) == 'black'
			&& moveColor(i, j, direction, -3) == 'black'
			&& moveColor(i, j, direction, -2) == 'black'
			&& moveColor(i, j, direction, -1) == 'black'
			&& moveColor(i, j, direction, 1) == 'black') return 1;

		return 0;
	};

	function checkBan(i, j) {
		let BanCount3;
		let BanCount4;
		let BanCount6;

		BanCount3 = check3Ban(i, j, [0, 1]) + check3Ban(i, j, [1, 0]) + check3Ban(i, j, [1, 1]) + check3Ban(i, j, [-1, 1]);
		BanCount4 = check4Ban(i, j, [0, 1]) + check4Ban(i, j, [1, 0]) + check4Ban(i, j, [1, 1]) + check4Ban(i, j, [-1, 1]);
		BanCount6 = check6Ban(i, j, [0, 1]) + check6Ban(i, j, [1, 0]) + check6Ban(i, j, [1, 1]) + check6Ban(i, j, [-1, 1]);

		if (BanCount3 >= 2 || BanCount4 >= 2 || BanCount6 >= 1) {
			return true;
		}

		return false;
	};

	function isBanSearchTarget(i, j) {
		let limit = 2;
		let iMinLimit = state.blackMinMaxPos[0] - limit;
		let iMaxLimit = state.blackMinMaxPos[1] + limit;
		let jMinLimit = state.blackMinMaxPos[2] - limit;
		let jMaxLimit = state.blackMinMaxPos[3] + limit;

		if (i >= iMinLimit && i <= iMaxLimit && j >= jMinLimit && j <= jMaxLimit) {
			return true;
		}

		return false;
	};

	async function requestAINewPosition(moves) {
		try{
			const target = await _think_and_move(moves, state.time);
			targetClick(rank_of(target), file_of(target), true);
			putClick(true);
		} catch(err) {
			alert(err);
		}
	}

	async function putClick(byAI=false) {
		if (state.player == 'none') {
			state.colorSelectVisible = true;
			return;
		}

		if (state.winState != 0 || (state.turn != state.player && !byAI)) {
			return;
		}

		if (state.current_target.length > 0) {
			let i = state.current_target[0];
			let j = state.current_target[1];

			state.color[i][j] = state.turn;
			state.position.push(make_move(i, j));

			for (let a = 0; a < state.target.length; a++) {
				for (let b = 0; b < state.target.length; b++) {
					if (state.target[a][b] == 'last' && (a != i || b != j)) {
						state.target[a][b] = 'hide';
					}
				}
			}
			state.target[i][j] = 'last';

			// 게임 종료 판단
			state.winState = checkWin();

			if (state.winState > 0) {
				setTimeout(() => {
					if (state.winState == 1) {
						alert('Black win!');
					} else if (state.winState == 2) {
						alert('White win!');
					}
				}, 200);
			}

			if (state.turn == 'black') {
				// 금수 초기화
				for (let i = 0; i < state.color.length; i++) {
					for (let j = 0; j < state.color.length; j++) {
						if (state.color[i][j] == 'ban') {
							state.color[i][j] = 'hide';
						}
					}
				}

				// 흑돌 위치 최대 최소 표시
				if (state.blackMinMaxPos[0] == -1) {
					state.blackMinMaxPos = [i, i, j, j];
				} else {
					let iMin = Math.min(i, state.blackMinMaxPos[0]);
					let iMax = Math.max(i, state.blackMinMaxPos[1]);
					let jMin = Math.min(j, state.blackMinMaxPos[2]);
					let jMax = Math.max(j, state.blackMinMaxPos[3]);

					state.blackMinMaxPos = [iMin, iMax, jMin, jMax];
				}

				state.turn = 'white';
			} else {
				// 금수 탐색
				let banList = [];
				for (let i = 0; i < state.color.length; i++) {
					for (let j = 0; j < state.color.length; j++) {
						if (isBanSearchTarget(i, j) && state.color[i][j] == 'hide') {
							if (checkBan(i, j)) {
								banList.push([i, j]);
							}
						}
					}
				}

				// 금수 표시
				banList.forEach(banPosition => {
					state.color[banPosition[0]][banPosition[1]] = 'ban';
				});

				state.turn = 'black';
			}

			if (state.turn != state.player && !byAI) {
				await requestAINewPosition(state.position);
			}
		}
	}

    return { 
		state,
		resetClick,
		undoClick,
		selectColor,
		_checkWin,
		checkWin,
		targetClick,
		moveCheck,
		moveColor,
		checkEndPoint1,
		checkEndPoint2,
		check3Ban,
		checkEndPoint3,
		checkEndPoint4,
		check4Ban,
		check6Ban,
		checkBan,
		isBanSearchTarget,
		requestAINewPosition,
		putClick,
    }
  }
}
</script>

<style scoped lang="stylus">
* {
	box-sizing: border-box;
}

.gomoku{
	text-align:center;
	align: center
}

.target {
	position: absolute;
	z-index: 11;
	width: 100%;
	height: 100%;
}

.color-select-dialog {
	font-size: 2em;
	font-weight: bold;
	min-width: 30vmin
}

.color-select:hover {
	border: 1px solid green;
}

.select {
	background:
		linear-gradient(to right, green 0.4vmin, transparent 0.4vmin) 0 0,
		linear-gradient(to right, green 0.4vmin, transparent 0.4vmin) 0 100%,
		linear-gradient(to left, green 0.4vmin, transparent 0.4vmin) 100% 0,
		linear-gradient(to left, green 0.4vmin, transparent 0.4vmin) 100% 100%,
		linear-gradient(to bottom, green 0.4vmin, transparent 0.4vmin) 0 0,
		linear-gradient(to bottom, green 0.4vmin, transparent 0.4vmin) 100% 0,
		linear-gradient(to top, green 0.4vmin, transparent 0.4vmin) 0 100%,
		linear-gradient(to top, green 0.4vmin, transparent 0.4vmin) 100% 100%;

	background-repeat: no-repeat;
	background-size: 25% 25%;
}

@media (min-aspect-ratio: 4/5) {	
	.last {
		width: calc((82vmin - 50px) / 60);
		height: @width;
		border-radius: calc((82vmin - 50px) / 120);
		background-color: #00CC00;
		z-index: 13;
		position: absolute;
		top: 50%;
		left: @top;
		transform: translate(-50%, -50%);
	}
	
	.title {
	  display: block;
	  font-size: 5vmin;
	  margin-top: 1vmin;
	  margin-bottom: 1vmin;
	  margin-left: 0;
	  margin-right: 0;
	  font-weight: bold;
	}
	
	#board {
		position: relative;
		display: inline-flex;
		background: linear-gradient(-135deg, darkorange, sandybrown);
		flex-direction: column;
	}
	
	#boundary{
		margin: calc((82vmin - 50px) / 30 - 1px);
		border: 1px solid black;
		position: relative;
	}
	
	.point{
		position: absolute;
		background: #000;
		width: calc((82vmin - 50px) / 60);
		height: @width;
		border-radius: calc((82vmin - 50px) / 120);
		transform: translate(calc(-1 * (82vmin - 50px) / 120), calc(-1 * (82vmin - 50px) / 120))
	}
	
	.row{
		display: flex;
	}
	
	.border-black {
		border: 1px solid #000;
	}
	.square {
		position: relative;
		width: calc((82vmin - 50px) / 15);
		height: @width;
		box-sizing: border-box;
	}
	
	.square-2x {
		position: relative;
		width: calc((82vmin - 50px) / 15 * 2);
		height: @width;
		box-sizing: border-box;
	}
	
	#play{
		position: absolute;
		flex-direction: column;
		width: 100%;
		height: @width;
		z-index: 10;
	}
	
	.center {
		top: 50%;
		left: @top;
		transform: translate(-50%, -50%);
	}
	
	.stone {
		width: calc((82vmin - 50px) / 45 * 2);
		height: @width;
		z-index: 10;
		position: absolute;
		border-radius: @width;
	}
	
	.stone-2x {
		width: calc((82vmin - 50px) / 45 * 4);
		height: @width;
		z-index: 10;
		position: absolute;
		border-radius: @width;
	}
	
	.white{
		background: #fff;
		box-shadow: 0 0 10px rgba(0, 0, 0, .5) inset;
	}
	
	.black{
		background: #000;
		box-shadow: 0 0 20px rgba(240, 240, 240, .5) inset;
	}
	
	.ban {
		background-size: cover;
		background-image: url('@/assets/x.png');
	}
	
	.deck {
		background: #444;
		width: calc(82vmin - 50px);
		margin-top: 20px;
		padding: 5px;
		box-sizing: border-box;
	}
	
	.undo-button {
		height: 8vmin;
		width: @height;
		font-size: 2vmin;
		background-color: #008800;
		color: white;
		border: 4px solid #006600;
	}
	
	.put-button {
		height: 8vmin;
		width: @height;
		font-size: 2vmin;
		background-color: #222;
		color: white;
		border: 4px solid #333;
	}
	
	.reset-button {
		height: 8vmin;
		width: @height;
		font-size: 2vmin;
		background-color: #880000;
		color: white;
		border: 4px solid #660000;
	}
}

@media	(max-aspect-ratio: 799/1000) {
	.last {
		width: calc((100vw - 5px) / 60);
		height: @width;
		border-radius: calc((100vw - 5px) / 120);
		background-color: #00CC00;
		z-index: 13;
		position: absolute;
		top: 50%;
		left: @top;
		transform: translate(-50%, -50%);
	}
	
	.title {
	  display: block;
	  font-size: 5vw;
	  margin-top: 1vw;
	  margin-bottom: 1vw;
	  margin-left: 0;
	  margin-right: 0;
	  font-weight: bold;
	}
	
	#board {
		position: relative;
		display: inline-flex;
		background: linear-gradient(-135deg, darkorange, sandybrown);
		flex-direction: column;
	}
	
	#boundary{
		margin: calc((100vw - 5px) / 30 - 1px);
		border: 1px solid black;
		position: relative;
	}
	
	.point{
		position: absolute;
		background: #000;
		width: calc((100vw - 5px) / 60);
		height: @width;
		border-radius: calc((100vw - 5px) / 120);
		transform: translate(calc(-1 * (100vw - 5px) / 120), calc(-1 * (100vw - 5px) / 120))
	}
	
	.row{
		display: flex;
	}
	
	.border-black {
		border: 1px solid #000;
	}
	.square {
		position: relative;
		width: calc((100vw - 5px) / 15);
		height: @width;
		box-sizing: border-box;
	}
	
	.square-2x {
		position: relative;
		width: calc((100vw - 5px) / 15 * 2);
		height: @width;
		box-sizing: border-box;
	}
	
	#play{
		position: absolute;
		flex-direction: column;
		width: 100%;
		height: @width;
		z-index: 10;
	}
	
	.center {
		top: 50%;
		left: @top;
		transform: translate(-50%, -50%);
	}
	
	.stone {
		width: calc((100vw - 5px) / 45 * 2);
		height: @width;
		z-index: 10;
		position: absolute;
		border-radius: @width;
	}
	
	.stone-2x {
		width: calc((100vw - 5px) / 45 * 4);
		height: @width;
		z-index: 10;
		position: absolute;
		border-radius: @width;
	}
	
	.white{
		background: #fff;
		box-shadow: 0 0 10px rgba(0, 0, 0, .5) inset;
	}
	
	.black{
		background: #000;
		box-shadow: 0 0 20px rgba(240, 240, 240, .5) inset;
	}
	
	.ban {
		background-size: cover;
		background-image: url('@/assets/x.png');
	}
	
	.deck {
		background: #444;
		width: calc(100vw - 5px);
		margin-top: 20px;
		padding: 5px;
		box-sizing: border-box;
	}
	
	.undo-button {
		height: 8vw;
		width: @height;
		font-size: 2vw;
		background-color: #008800;
		color: white;
		border: 4px solid #006600;
	}
	
	.put-button {
		height: 8vw;
		width: @height;
		font-size: 2vw;
		background-color: #222;
		color: white;
		border: 4px solid #333;
	}
	
	.reset-button {
		height: 8vw;
		width: @height;
		font-size: 2vw;
		background-color: #880000;
		color: white;
		border: 4px solid #660000;
	}
}

</style>