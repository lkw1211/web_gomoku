<template>
	<div class="gomokuAI">
		<Header-comp title="Gomoku AI" @timelimitChange="timelimitChange"></Header-comp>
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
import { reactive, onBeforeUnmount } from 'vue';
import HeaderComp from '@/components/HeaderComp.vue';

export default {
  name: 'HomeView',
  components: {
	HeaderComp
  },
  setup(props) {
	let myWorker = new Worker("../worker.js");
	try {
		myWorker.postMessage('initialize');
		window.onbeforeunload = event => {
			myWorker.terminate();
		};
		onBeforeUnmount(() => {
			myWorker.terminate();
		});
	} catch (err) {
		alert(`${err}`);
	}

	function _think_and_move(moves, tl) {
		let args = arguments;
		return new Promise((resolve, reject) => {
			myWorker.onmessage = message => resolve(message.data);
			myWorker.postMessage(JSON.stringify({
				func_name: 'think_and_move',
				args,
			}));
		});
	};

	function _make_move(rank, file) {
		let args = arguments;
		return new Promise((resolve, reject) => {
			myWorker.onmessage = message => resolve(message.data);
			myWorker.postMessage(JSON.stringify({
				func_name: 'make_move',
				args,
			}));
		});
	};

	function _rank_of(move) {
		let args = arguments;
		return new Promise((resolve, reject) => {
			myWorker.onmessage = message => resolve(message.data);
			myWorker.postMessage(JSON.stringify({
				func_name: 'rank_of',
				args,
			}));
		});
	}

	function _file_of(move) {
		let args = arguments;
		return new Promise((resolve, reject) => {
			myWorker.onmessage = message => resolve(message.data);
			myWorker.postMessage(JSON.stringify({
				func_name: 'file_of',
				args,
			}));
		});
	}

	function _foul_moves(position) {
		let args = arguments;
		return new Promise((resolve, reject) => {
			myWorker.onmessage = message => resolve(message.data);
			myWorker.postMessage(JSON.stringify({
				func_name: 'foul_moves',
				args,
			}));
		});
	}

	function _check_wld_already(position) {
		let args = arguments;
		return new Promise((resolve, reject) => {
			myWorker.onmessage = message => resolve(message.data);
			myWorker.postMessage(JSON.stringify({
				func_name: 'check_wld_already',
				args,
			}));
		});
	}

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
		time: 20,
		soloPlay: false,
	};

	const state = reactive(JSON.parse(JSON.stringify(initialState)));

	function resetClick() {
		if (state.turn != state.player && state.winState == 0) {
			alert("It cannot be reset during the AI's turn.");
			return;
		}
		Object.assign(state, JSON.parse(JSON.stringify(initialState)));
	}

	async function undoClick() {
		const p_len = state.position.length;
		if ((state.turn != state.player && state.winState == 0) || p_len < 2) {
			alert("It cannot be undone during the AI's turn.");
			return;
		}
		const lastMove = [await _rank_of(state.position[p_len - 1]), await _file_of(state.position[p_len - 1])];
		const secondlastMove = [await _rank_of(state.position[p_len - 2]), await _file_of(state.position[p_len - 2])];
		
		state.color[lastMove[0]][lastMove[1]] = 'hide';
		state.color[secondlastMove[0]][secondlastMove[1]] = 'hide';
		state.target[lastMove[0]][lastMove[1]] = 'hide';
		state.position = state.position.slice(0, -2);

		if (state.position.length > 0) {
			const lastMove = state.position.slice(-1)[0];

			state.target[await _rank_of(lastMove)][await _file_of(lastMove)] = 'last';
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

			// 금수 표시
			let foul_moves = await _foul_moves(state.position);
			
			foul_moves.forEach(async move => {
				state.color[await _rank_of(move)][await _file_of(move)] = 'ban';
			});
		}
		state.winState = 0;
	};

	function selectColor(color) {
		state.player = color;

		if (color == 'white') {
			targetClick((state.play_rows - 1)/2, (state.play_rows - 1)/2, true);
			putClick(true);
		}
		state.colorSelectVisible = false;
	}

	function targetClick(i, j, byAI=state.soloPlay) {
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

	async function requestAINewPosition(moves) {
		try{
			let target = await _think_and_move(moves, state.time);
			targetClick(await _rank_of(target), await _file_of(target), true);
			putClick(true);
		} catch(err) {
			console.log(err);
		}
	}

	async function putClick(byAI=state.soloPlay) {
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

			state.current_target = [];

			state.color[i][j] = state.turn;
			state.position.push(await _make_move(i, j));

			for (let a = 0; a < state.target.length; a++) {
				for (let b = 0; b < state.target.length; b++) {
					if (state.target[a][b] == 'last' && (a != i || b != j)) {
						state.target[a][b] = 'hide';
					}
				}
			}
			state.target[i][j] = 'last';

			// 게임 종료 판단
			state.winState = await _check_wld_already(state.position);


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

				state.turn = 'white';
			} else {
				// 금수 표시
				let foul_moves = await _foul_moves(state.position);

				for (let foul_move of foul_moves) {
					let rank = await _rank_of(foul_move);
					let file = await _file_of(foul_move);

					state.color[rank][file] = 'ban';
				}

				// await Promise.all((await _foul_moves(state.position)).map(async ban_move => {
				// 	state.color[await _rank_of(ban_move)][await _file_of(ban_move)] = 'ban';
				// }));

				state.turn = 'black';
			}

			if (state.turn != state.player && !byAI) {
				requestAINewPosition(state.position);
			}
		}
	}

	function timelimitChange(tl) {
		state.time = tl;
	}

    return { 
		state,
		resetClick,
		undoClick,
		selectColor,
		targetClick,
		moveCheck,
		moveColor,
		requestAINewPosition,
		putClick,
		timelimitChange,
    }
  }
}
</script>

<style scoped lang="stylus">
* {
	box-sizing: border-box;
}

.gomokuAI{
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